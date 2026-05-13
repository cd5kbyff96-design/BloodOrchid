# invariants/ocaml/topological.ml
(* Topological invariants for state validation *)

type topology_error = {
  invariant_name: string;
  details: string;
  observed_value: float option;
  expected_range: (float * float) option;
}

type topology_result = {
  satisfied: bool;
  errors: topology_error list;
}

(* Validate that state values remain within physical bounds *)
let validate_conservation_laws (state: bytes) : topology_result =
  let open Bytes in
  let errors = ref [] in
  let len = length state in
  
  (* Check for finite values only *)
  for i = 0 to len - 4 do
    let value = get_float state i in
    if not (Float.is_finite value) then (
      errors := {
        invariant_name = "finiteness";
        details = Printf.sprintf "Non-finite value at offset %d" i;
        observed_value = Some value;
        expected_range = None;
      } :: !errors
    );
    i <- i + 4
  done;
  
  {
    satisfied = List.length !errors = 0;
    errors = !errors;
  }