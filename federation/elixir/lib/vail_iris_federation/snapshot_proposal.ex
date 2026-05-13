# federation/elixir/lib/vail_iris_federation/snapshot_proposal.ex
defmodule VailIrisFederation.SnapshotProposal do
  @moduledoc """
  Snapshot proposal structure for federation consensus.
  
  Represents a simulation state snapshot proposed for
  distributed consensus across federated nodes.
  """

  defstruct [
    :proposal_id,
    :simulation_id,
    :step_index,
    :state_bytes,
    :state_hash,
    :gate_status,
    :invariant_token,
    :proposed_at
  ]

  @type t :: %__MODULE__{
          proposal_id: String.t(),
          simulation_id: String.t(),
          step_index: non_neg_integer(),
          state_bytes: binary(),
          state_hash: String.t(),
          gate_status: :valid | :uncertain,
          invariant_token: binary(),
          proposed_at: non_neg_integer()
        }

  @spec new(map()) :: t()
  def new(attrs) when is_map(attrs) do
    %__MODULE__{
      proposal_id: Map.get(attrs, :proposal_id, generate_id()),
      simulation_id: Map.get(attrs, :simulation_id, ""),
      step_index: Map.get(attrs, :step_index, 0),
      state_bytes: Map.get(attrs, :state_bytes, <<>>),
      state_hash: Map.get(attrs, :state_hash, ""),
      gate_status: Map.get(attrs, :gate_status, :valid),
      invariant_token: Map.get(attrs, :invariant_token, <<>>),
      proposed_at: Map.get(attrs, :proposed_at, System.system_time(:millisecond))
    }
  end

  @spec valid?(t()) :: boolean()
  def valid?(proposal) do
    byte_size(proposal.simulation_id) > 0 and
    byte_size(proposal.state_hash) == 16 and
    byte_size(proposal.invariant_token) > 0 and
    proposal.gate_status in [:valid, :uncertain]
  end

  defp generate_id do
    :crypto.hash(:sha256, :erlang.term_to_binary({System.unique_integer([:monotonic]), self()}))
    |> Base.encode16(case: :lower)
    |> binary_part(0, 16)
  end
end