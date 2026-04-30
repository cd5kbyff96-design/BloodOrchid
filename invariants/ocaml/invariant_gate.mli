type request = { simulation_state : bytes }

type response = {
  accepted : bool;
  violations : string list;
}

val evaluate : request -> response
