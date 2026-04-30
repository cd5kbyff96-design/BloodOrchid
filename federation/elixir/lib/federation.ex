defmodule VailIrisFederation do
  @moduledoc """
  Vail Iris Federation Layer — Phase 2C stub.

  Implements the HotStuff-derived BFT quorum coordinator for SimulationState
  snapshot consensus across federated nodes.

  Architecture invariants this module enforces:
    I1  No direct kernel access. All state arrives as SnapshotProposal
        contract envelopes from the Rust boundary layer.
    I2  OCaml pre-admission gate runs BEFORE any proposal enters the
        quorum pipeline. Proposals with gate_status="invalid" are
        hard-rejected in `pre_admit/1` and never reach the vote phase.
        This mirrors the architecture: "OCaml filter runs upstream of
        consensus, not downstream."
    I3  Proposals missing an invariant_token (proves gate bypass) are
        hard-rejected. There is no override path.
    I4  Quorum commit is deterministic for identical proposal + vote
        sequences on the same node state.
    I5  No kernel logic, no invariant logic, no frontend logic here.
        All interop is contract-based.

  Phase 2C scope:
    - In-memory validator registry (single local node, threshold = 1)
    - Synchronous GenServer (no distributed Elixir cluster yet)
    - BLS signatures are placeholder bytes (production: threshold BLS in Phase 6)
    - Speculative execution path declared but not activated

  Production path (Phase 6):
    - Distributed Elixir cluster with ETS-backed validator registry + TTL rotation
    - Full BLS threshold signature aggregation via Rust consensus/ crate
    - Asynchronous batch proposals with Phoenix.PubSub fan-out
    - Speculative commit for low-risk updates (loss_delta < threshold)
  """

  use GenServer
  require Logger

  # ─── Public API ─────────────────────────────────────────────────────────────

  @doc "Start the federation quorum coordinator."
  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @doc """
  Propose a Rust-boundary-validated SimulationState snapshot for consensus.

  The proposal map MUST include:
    - :simulation_id    — non-empty string
    - :step_index       — non-negative integer
    - :state_bytes      — binary (serialized SimulationState)
    - :state_hash       — 16-char FNV-64 hex string
    - :gate_status      — "valid" | "uncertain"  (never "invalid")
    - :invariant_token  — non-empty binary (OCaml gate token)

  Returns {:ok, proposal_id} | {:error, reason}
  """
  @spec propose_snapshot(map()) :: {:ok, String.t()} | {:error, term()}
  def propose_snapshot(proposal) when is_map(proposal) do
    GenServer.call(__MODULE__, {:propose, proposal})
  end

  @doc """
  Cast a vote on an active proposal.

  Returns {:ok, :committed | :pending} | {:error, :unknown_proposal}
  """
  @spec cast_vote(String.t(), String.t(), boolean(), String.t() | nil) ::
          {:ok, :committed | :pending} | {:error, :unknown_proposal}
  def cast_vote(proposal_id, node_id, accept, reason \\ nil) do
    GenServer.call(__MODULE__, {:vote, proposal_id, node_id, accept, reason})
  end

  @doc "Return all committed snapshots ordered by step_index (ascending)."
  @spec committed_snapshots() :: {:ok, [map()]}
  def committed_snapshots do
    GenServer.call(__MODULE__, :list_committed)
  end

  @doc "Return the latest committed snapshot, or {:ok, nil} if none."
  @spec latest_snapshot() :: {:ok, map() | nil}
  def latest_snapshot do
    GenServer.call(__MODULE__, :latest_snapshot)
  end

  @doc "Return pending proposal IDs."
  @spec pending_proposals() :: {:ok, [String.t()]}
  def pending_proposals do
    GenServer.call(__MODULE__, :list_pending)
  end

  # ─── GenServer callbacks ─────────────────────────────────────────────────────

  @impl true
  def init(opts) do
    threshold = Keyword.get(opts, :quorum_threshold, 1)
    validators = Keyword.get(opts, :validators, MapSet.new(["local-node-0"]))

    state = %{
      pending:           %{},        # proposal_id => proposal_map
      committed:         [],         # list of committed proposals (ordered)
      validators:        validators, # MapSet of node IDs
      quorum_threshold:  threshold,  # minimum accepting votes to commit
    }

    Logger.info(
      "[Federation] Quorum coordinator started " <>
        "(phase=2C, threshold=#{threshold}, validators=#{MapSet.size(validators)})"
    )

    {:ok, state}
  end

  @impl true
  def handle_call({:propose, proposal}, _from, state) do
    case pre_admit(proposal) do
      {:ok, admission_status} ->
        proposal_id = resolve_proposal_id(proposal)

        enriched =
          proposal
          |> Map.put(:proposal_id, proposal_id)
          |> Map.put(:votes, %{})
          |> Map.put(:admission_status, admission_status)
          |> Map.put(:proposed_at_mono, System.monotonic_time(:millisecond))

        pending = Map.put(state.pending, proposal_id, enriched)

        log_level = if admission_status == :uncertain, do: :warning, else: :info
        Logger.log(
          log_level,
          "[Federation] Proposal admitted (#{admission_status}): " <>
            "id=#{proposal_id} sim=#{proposal[:simulation_id]} step=#{proposal[:step_index]}"
        )

        {:reply, {:ok, proposal_id}, %{state | pending: pending}}

      {:error, reason} ->
        Logger.warning(
          "[Federation] Proposal rejected by pre-admission gate: #{inspect(reason)}"
        )
        {:reply, {:error, {:pre_admission_rejected, reason}}, state}
    end
  end

  @impl true
  def handle_call({:vote, proposal_id, node_id, accept, reason}, _from, state) do
    case Map.get(state.pending, proposal_id) do
      nil ->
        {:reply, {:error, :unknown_proposal}, state}

      proposal ->
        vote = %{
          node_id:  node_id,
          accept:   accept,
          reason:   reason,
          voted_at: System.monotonic_time(:millisecond),
        }

        updated_proposal = put_in(proposal, [:votes, node_id], vote)
        pending          = Map.put(state.pending, proposal_id, updated_proposal)
        state            = %{state | pending: pending}

        accept_count =
          updated_proposal[:votes]
          |> Map.values()
          |> Enum.count(& &1[:accept])

        if accept_count >= state.quorum_threshold do
          {committed_proposal, remaining_pending} = Map.pop(state.pending, proposal_id)

          committed_proposal =
            committed_proposal
            |> Map.put(:committed_at_mono, System.monotonic_time(:millisecond))
            |> Map.put(:commit_proof, build_commit_proof(committed_proposal, accept_count))

          committed = state.committed ++ [committed_proposal]

          Logger.info(
            "[Federation] Snapshot committed: " <>
              "id=#{proposal_id} sim=#{committed_proposal[:simulation_id]} " <>
              "step=#{committed_proposal[:step_index]} votes=#{accept_count}/#{state.quorum_threshold}"
          )

          {:reply, {:ok, :committed}, %{state | pending: remaining_pending, committed: committed}}
        else
          {:reply, {:ok, :pending}, state}
        end
    end
  end

  @impl true
  def handle_call(:list_committed, _from, state) do
    summaries =
      Enum.map(
        state.committed,
        &Map.take(&1, [:proposal_id, :simulation_id, :step_index, :state_hash, :gate_status, :committed_at_mono])
      )
    {:reply, {:ok, summaries}, state}
  end

  @impl true
  def handle_call(:latest_snapshot, _from, state) do
    {:reply, {:ok, List.last(state.committed)}, state}
  end

  @impl true
  def handle_call(:list_pending, _from, state) do
    {:reply, {:ok, Map.keys(state.pending)}, state}
  end

  # ─── Private helpers ─────────────────────────────────────────────────────────

  # pre_admit/1 — pre-admission filter (OCaml gate.ml stub).
  #
  # Production path: Rust boundary calls into OCaml via C-ABI embedding.
  # This stub enforces the same rejection semantics locally.
  #
  # Rejection rules (hard — no override path):
  #   1. gate_status="invalid" — OCaml gate already rejected this
  #   2. Missing invariant_token — proves gate bypass attempt
  #   3. Missing simulation_id
  #   4. Missing state_hash
  #
  # Returns: {:ok, :valid | :uncertain} | {:error, reason_string}
  defp pre_admit(proposal) do
    cond do
      proposal[:gate_status] == "invalid" ->
        {:error, "gate_status=invalid; OCaml gate rejected before federation admission"}

      is_nil(proposal[:invariant_token]) or proposal[:invariant_token] == "" or
          proposal[:invariant_token] == <<>> ->
        {:error, "invariant_token missing or empty; gate bypass is forbidden"}

      is_nil(proposal[:simulation_id]) or proposal[:simulation_id] == "" ->
        {:error, "simulation_id must be present and non-empty"}

      is_nil(proposal[:state_hash]) or proposal[:state_hash] == "" ->
        {:error, "state_hash must be present; required for commit proof"}

      is_nil(proposal[:step_index]) ->
        {:error, "step_index must be present"}

      proposal[:gate_status] == "uncertain" ->
        {:ok, :uncertain}

      true ->
        {:ok, :valid}
    end
  end

  # Resolve or generate a proposal_id for an incoming proposal.
  defp resolve_proposal_id(%{proposal_id: id}) when is_binary(id) and byte_size(id) > 0, do: id

  defp resolve_proposal_id(proposal) do
    input =
      "#{proposal[:simulation_id]}\0#{proposal[:step_index]}\0#{System.unique_integer([:monotonic])}"

    :crypto.hash(:sha256, input)
    |> Base.encode16(case: :lower)
    |> binary_part(0, 16)
  end

  # Build a stub CommitProof map (placeholder for BLS aggregate sig in Phase 6).
  defp build_commit_proof(proposal, votes_received) do
    %{
      proposal_id:         proposal[:proposal_id],
      simulation_id:       proposal[:simulation_id],
      step_index:          proposal[:step_index],
      state_hash:          proposal[:state_hash],
      aggregate_signature: <<>>,   # BLS aggregate sig placeholder
      quorum_size:         1,      # stub: single-node quorum
      votes_received:      votes_received,
    }
  end
end

# ─── OTP Application ──────────────────────────────────────────────────────────

defmodule VailIrisFederation.Application do
  @moduledoc false
  use Application

  @impl true
  def start(_type, _args) do
    children = [
      VailIrisFederation,
    ]
    opts = [strategy: :one_for_one, name: VailIrisFederation.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
