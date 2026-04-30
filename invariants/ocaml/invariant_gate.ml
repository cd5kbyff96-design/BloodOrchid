type request = { simulation_state : bytes }

type response = {
  accepted : bool;
  violations : string list;
}

(* Lightweight deterministic stub:
   - accepts plausibly valid SimulationState payload bytes
   - rejects malformed/empty payloads *)
let evaluate (req : request) : response =
  let n = Bytes.length req.simulation_state in
  if n = 0 then
    { accepted = false; violations = [ "simulation_state is empty" ] }
  else
    let b0 = Char.code (Bytes.get req.simulation_state 0) in
    if b0 <> 0x0A then
      { accepted = false; violations = [ "malformed simulation_state payload" ] }
    else if n >= 2 && Char.code (Bytes.get req.simulation_state 1) = 0 then
      { accepted = false; violations = [ "empty simulation_id in payload" ] }
    else
      { accepted = true; violations = [] }
