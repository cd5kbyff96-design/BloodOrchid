# invariants/ocaml/causal.ml
(* Causal validator for simulation state transitions *)

type violation = {
  reason: string;
  severity: string;
  context: string option;
}

type causal_result = {
  valid: bool;
  violations: violation list;
  confidence: float;
}

(* Check that state transitions preserve causal ordering *)
let validate_causal_order (state_history: bytes list) : causal_result =
  let open Bytes in
  let violations = ref [] in
  
  (* Check each consecutive pair of states *)
  let rec check_pairs states =
    match states with
    | [] | [_] -> ()
    | a :: b :: rest ->
      (* Extract step indices and verify monotonicity *)
      let step_a = get_int32 a 4 in
      let step_b = get_int32 b 4 in
      if step_b <= step_a then (
        violations := {reason = "Step index decreased"; severity = "critical"; context = None} :: !violations
      );
      check_pairs (b :: rest)
  in
  
  check_pairs state_history;
  
  {
    valid = List.length !violations = 0;
    violations = !violations;
    confidence = 1.0 -. float_of_int (List.length !violations) /. 10.0;
  }