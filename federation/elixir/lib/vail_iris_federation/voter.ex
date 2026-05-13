# federation/elixir/lib/vail_iris_federation/voter.ex
defmodule VailIrisFederation.Voter do
  @moduledoc """
  Voter behavior for federation consensus.
  
  Implements the voting logic for accepting or rejecting
  snapshot proposals in the HotStuff-derived BFT quorum.
  """

  @type vote_result :: :accept | :reject | :abstain

  @doc """
  Cast a vote on a proposal.
  
  Returns {:ok, :committed | :pending} or {:error, reason}
  """
  @spec cast_vote(String.t(), String.t(), boolean(), String.t() | nil) ::
          {:ok, :committed | :pending} | {:error, :unknown_proposal}
  def cast_vote(proposal_id, node_id, accept, reason \\ nil) do
    VailIrisFederation.cast_vote(proposal_id, node_id, accept, reason)
  end

  @doc """
  Validate a proposal before voting.
  """
  @spec validate_proposal(VailIrisFederation.SnapshotProposal.t()) :: :ok | {:error, String.t()}
  def validate_proposal(proposal) do
    cond do
      byte_size(proposal.state_bytes) == 0 ->
        {:error, "state_bytes is empty"}

      byte_size(proposal.state_hash) != 16 ->
        {:error, "state_hash must be 16 characters"}

      proposal.gate_status == :invalid ->
        {:error, "gate_status is invalid"}

      byte_size(proposal.invariant_token) == 0 ->
        {:error, "invariant_token is missing"}

      true ->
        :ok
    end
  end
end