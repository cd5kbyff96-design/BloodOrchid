# invariants/ocaml/policy.ml
(* Policy checker for operational constraints *)

type policy_violation = {
  rule: string;
  message: string;
  remediation: string option;
}

type policy_result = {
  compliant: bool;
  violations: policy_violation list;
}

(* Check operational policies *)
let check_policies (state: bytes) (metadata: string StringMap.t) : policy_result =
  let violations = ref [] in
  
  (* Check simulation_id is present *)
  (match StringMap.find_opt "simulation_id" metadata with
   | None -> violations := {rule = "simulation_id_required"; message = "simulation_id missing"; remediation = Some "Provide simulation_id"} :: !violations
   | Some id when String.length id = 0 -> violations := {rule = "simulation_id_required"; message = "simulation_id is empty"; remediation = Some "Provide non-empty simulation_id"} :: !violations
   | _ -> ());
  
  (* Check step_index is non-negative *)
  (match StringMap.find_opt "step_index" metadata with
   | None -> violations := {rule = "step_index_required"; message = "step_index missing"; remediation = Some "Provide step_index"} :: !violations
   | Some idx -> 
       (try
          if int_of_string idx < 0 then
            violations := {rule = "step_index_nonnegative"; message = "step_index must be non-negative"; remediation = None} :: !violations
        with Failure _ -> ())
   | _ -> ());
  
  {
    compliant = List.length !violations = 0;
    violations = !violations;
  }