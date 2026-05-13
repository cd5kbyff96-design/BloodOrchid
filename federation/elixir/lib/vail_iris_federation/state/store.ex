# federation/elixir/lib/vail_iris_federation/state/store.ex
defmodule VailIrisFederation.State.Store do
  @moduledoc """
  Distributed state store for federation layer.
  
  Manages committed snapshots with efficient lookup by
  simulation_id and step_index.
  """

  use GenServer

  @doc """
  Start the state store.
  """
  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @impl true
  def init(_opts) do
    {:ok, %{
      snapshots: %{},  # simulation_id => [snapshot]
      by_step: %{},    # {simulation_id, step_index} => snapshot
      latest: %{}      # simulation_id => latest snapshot
    }}
  end

  @doc """
  Store a committed snapshot.
  """
  @spec store(map()) :: :ok
  def store(snapshot) do
    GenServer.call(__MODULE__, {:store, snapshot})
  end

  @doc """
  Get latest snapshot for a simulation.
  """
  @spec get_latest(String.t()) :: {:ok, map() | nil}
  def get_latest(simulation_id) do
    GenServer.call(__MODULE__, {:get_latest, simulation_id})
  end

  @doc """
  Get snapshot by step index.
  """
  @spec get_at(String.t(), non_neg_integer()) :: {:ok, map() | nil}
  def get_at(simulation_id, step_index) do
    GenServer.call(__MODULE__, {:get_at, simulation_id, step_index})
  end

  @impl true
  def handle_call({:store, snapshot}, _from, state) do
    sim_id = snapshot[:simulation_id]
    step = snapshot[:step_index]

    # Update snapshots list
    snapshots = Map.update(state.snapshots, sim_id, [snapshot], fn list ->
      [snapshot | list] |> Enum.sort_by(& &1[:step_index])
    end)

    # Update by_step index
    by_step = Map.put(state.by_step, {sim_id, step}, snapshot)

    # Update latest
    latest = Map.put(state.latest, sim_id, snapshot)

    new_state = %{state | snapshots: snapshots, by_step: by_step, latest: latest}
    {:reply, :ok, new_state}
  end

  @impl true
  def handle_call({:get_latest, sim_id}, _from, state) do
    {:reply, {:ok, Map.get(state.latest, sim_id)}, state}
  end

  @impl true
  def handle_call({:get_at, sim_id, step}, _from, state) do
    {:reply, {:ok, Map.get(state.by_step, {sim_id, step})}, state}
  end
end