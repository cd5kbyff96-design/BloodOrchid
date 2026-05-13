# federation/elixir/lib/vail_iris_federation/commit_proof.ex
defmodule VailIrisFederation.CommitProof do
  @moduledoc """
  Commit proof structure for federation consensus.
  
  Contains the cryptographic proof that a snapshot proposal
  achieved quorum consensus.
  """

  defstruct [
    :proposal_id,
    :simulation_id,
    :step_index,
    :state_hash,
    :aggregate_signature,
    :quorum_size,
    :votes_received,
    :committed_at
  ]

  @type t :: %__MODULE__{
          proposal_id: String.t(),
          simulation_id: String.t(),
          step_index: non_neg_integer(),
          state_hash: String.t(),
          aggregate_signature: binary(),
          quorum_size: pos_integer(),
          votes_received: pos_integer(),
          committed_at: non_neg_integer()
        }

  @spec new(map()) :: t()
  def new(attrs) when is_map(attrs) do
    %__MODULE__{
      proposal_id: Map.get(attrs, :proposal_id, ""),
      simulation_id: Map.get(attrs, :simulation_id, ""),
      step_index: Map.get(attrs, :step_index, 0),
      state_hash: Map.get(attrs, :state_hash, ""),
      aggregate_signature: Map.get(attrs, :aggregate_signature, <<>>),
      quorum_size: Map.get(attrs, :quorum_size, 1),
      votes_received: Map.get(attrs, :votes_received, 1),
      committed_at: Map.get(attrs, :committed_at, System.system_time(:millisecond))
    }
  end

  @spec valid?(t()) :: boolean()
  def valid?(proof) do
    byte_size(proof.proposal_id) > 0 and
    byte_size(proof.state_hash) == 16 and
    proof.votes_received >= 1 and
    proof.votes_received <= proof.quorum_size
  end
end