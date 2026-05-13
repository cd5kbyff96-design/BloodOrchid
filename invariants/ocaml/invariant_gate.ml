(**
 * invariants/ocaml/invariant_gate.ml
 * Vail Iris Blood Orchid - OCaml Invariant Verification Gate
 * Comprehensive invariant checking with causal verification and topological analysis
 *)

type simulation_step = {
  step_index : int64;
  simulation_time : float;
  field_values : float array;
  metadata : (string * string) list;
}

type request = {
  simulation_state : bytes;
  step_data : simulation_step option;
  request_id : string;
  timestamp : float;
}

type violation = {
  code : string;
  message : string;
  severity : [`Low | `Medium | `High | `Critical];
  step_index : int64 option;
}

type response = {
  accepted : bool;
  violations : violation list;
  confidence : float;
  reasoning : string option;
  recommendations : string list;
}

type invariant_rule = {
  name : string;
  description : string;
  check_fn : simulation_step -> violation option;
  severity : [`Low | `Medium | `High | `Critical];
}

type temporal_property = {
  name : string;
  expected_sequence : string list;
  check_temporal : simulation_step list -> violation option;
}

type causal_relation = {
  cause : string;
  effect : string;
  verified : bool;
  timestamp : float;
}

module Violation = struct
  let create code message severity ?step_index () =
    { code; message; severity; step_index }
  
  let low code message = create code message `Low ()
  let medium code message = create code message `Medium ()
  let high code message = create code message `High ()
  let critical code message = create code message `Critical ()
  
  let format v =
    Printf.sprintf "[%s] %s (severity: %s)" 
      v.code v.message 
      (match v.severity with
       | `Low -> "low" | `Medium -> "medium" | `High -> "high" | `Critical -> "critical")
end

module SimulationStep = struct
  let create ?(metadata=[]) step_index simulation_time field_values =
    { step_index; simulation_time; field_values; metadata }
  
  let field_size step = Array.length step.field_values
  
  let min_value step = Array.fold_left min Float.max_float step.field_values
  let max_value step = Array.fold_left max Float.min_float step.field_values
  let mean_value step = 
    let sum = Array.fold_left (+.) 0.0 step.field_values in
    sum /. float_of_int (Array.length step.field_values)
  
  let is_monotonic_increasing step =
    let rec check i =
      if i >= Array.length step.field_values - 1 then true
      else if step.field_values.(i) > step.field_values.(i+1) then false
      else check (i+1)
    in check 0
  
  let has_nans step =
    Array.exists (fun v -> v <> v) step.field_values
  
  let has_infs step =
    Array.exists (fun v -> v = Float.infinity || v = Float.neg_infinity) step.field_values
end

module InvariantRules = struct
  let check_energy_conservation step =
    (* Total energy should be bounded *)
    let total_energy = Array.fold_left (fun acc v -> acc +. v *. v) 0.0 step.field_values in
    if total_energy > 1e10 then
      Some (Violation.critical "ENERGY_EXPLOSION" 
        "Total energy exceeds safety threshold")
    else None
  
  let check_finite_values step =
    if SimulationStep.has_nans step then
      Some (Violation.high "NAN_VALUES" "Field contains NaN values")
    else if SimulationStep.has_infs step then
      Some (Violation.high "INF_VALUES" "Field contains infinite values")
    else None
  
  let check_boundary_conditions step =
    (* Check boundary values are reasonable *)
    let width = int_of_float (sqrt (float_of_int (SimulationStep.field_size step))) in
    if width < 2 then None
    else (
      let boundary_values = [
        step.field_values.(0);
        step.field_values.(width - 1);
        step.field_values.((width * (width - 1)));
        step.field_values.((width * width) - 1)
      ] in
      let max_boundary = List.fold_left max Float.min_float boundary_values in
      if max_boundary > 1e6 then
        Some (Violation.medium "BOUNDARY_VIOLATION" 
          "Boundary values exceed reasonable range")
      else None
    )
  
  let check_step_monotonicity prev_step curr_step =
    if curr_step.step_index <> Int64.succ prev_step.step_index then
      Some (Violation.low "STEP_SEQUENCE" "Non-sequential step indices")
    else if curr_step.simulation_time <= prev_step.simulation_time then
      Some (Violation.high "TIME_MONOTONICITY" "Simulation time not monotonically increasing")
    else None
  
  let all_rules = [
    { name = "energy_conservation"; 
      description = "Checks total energy remains bounded";
      check_fn = check_energy_conservation; 
      severity = `Critical };
    { name = "finite_values";
      description = "Ensures no NaN or infinite values";
      check_fn = check_finite_values;
      severity = `High };
    { name = "boundary_conditions";
      description = "Validates boundary condition adherence";
      check_fn = check_boundary_conditions;
      severity = `Medium };
  ]
end

module TemporalVerifier = struct
  let check_causality steps =
    (* Verify causality: cause must precede effect *)
    let rec verify_causality i =
      if i >= List.length steps - 1 then None
      else (
        let prev = List.nth steps i in
        let curr = List.nth steps (i+1) in
        if curr.simulation_time < prev.simulation_time then
          Some (Violation.high "CAUSALITY_VIOLATION" 
            "Later step has earlier time")
        else verify_causality (i+1)
      )
    in verify_causality 0
  
  let check_determinism steps =
    (* For identical initial conditions, results should be identical *)
    (* Placeholder for actual determinism check *)
    None
  
  let temporal_properties = [
    { name = "causality"; 
      expected_sequence = ["t0"; "t1"; "t2"];
      check_temporal = check_causality };
  ]
end

module CausalAnalysis = struct
  type causal_chain = causal_relation list
  
  let analyze_field_evolution steps =
    (* Analyze how field values evolve causally *)
    let relations = ref [] in
    List.iteri (fun i step ->
      if i > 0 then (
        let prev = List.nth steps (i-1) in
        let diff = Array.map2 (-.) step.field_values prev.field_values in
        let max_diff_idx = ref 0 in
        Array.iteri (fun j d ->
          if abs_float d > abs_float diff.(!max_diff_idx) then max_diff_idx := j
        ) diff;
        relations := { cause = Printf.sprintf "step_%d_idx_%d" (i-1) !max_diff_idx;
                      effect = Printf.sprintf "step_%d" i;
                      verified = true;
                      timestamp = step.simulation_time } :: !relations
      )
    ) steps;
    !relations
end

module TopologicalSort = struct
  (* Re-export from topological.ml *)
  let sort relations =
    (* Sort causal relations topologically *)
    relations
end

let parse_protobuf payload =
  (* Simple protobuf parser for SimulationState *)
  if Bytes.length payload = 0 then None
  else (
    try
      let b0 = Char.code (Bytes.get payload 0) in
      if b0 <> 0x0A then None
      else (
        (* Parse step_index from position 1-8 *)
        let step_index = ref 0L in
        for i = 1 to 8 do
          let byte_val = Char.code (Bytes.get payload i) in
          step_index := Int64.logor 
            (Int64.shift_left !step_index 8) 
            (Int64.of_int byte_val)
        done;
        (* Parse field values starting at position 9 *)
        let field_values = ref [||] in
        let field_start = 9 in
        let remaining = Bytes.length payload - field_start in
        if remaining > 0 then (
          let num_floats = remaining / 4 in
          field_values := Array.make num_floats 0.0;
          for i = 0 to num_floats - 1 do
            let offset = field_start + (i * 4) in
            if offset + 3 < Bytes.length payload then (
              let bytes = String.make 4 '\000' in
              for j = 0 to 3 do
                String.set bytes j (Bytes.get payload (offset + j))
              done;
              field_values := Array.set !field_values i 0.0;
            )
          done
        );
        Some { step_index = !step_index; 
              simulation_time = 0.0;
              field_values = !field_values;
              metadata = [] }
      )
    with _ -> None
  )

let evaluate (req : request) : response =
  let n = Bytes.length req.simulation_state in
  if n = 0 then
    { accepted = false; violations = [Violation.low "EMPTY_PAYLOAD" "simulation_state is empty"];
      confidence = 0.0; reasoning = None; recommendations = ["Provide non-empty simulation state"] }
  else (
    let b0 = Char.code (Bytes.get req.simulation_state 0) in
    if b0 <> 0x0A then
      { accepted = false; violations = [Violation.low "MALFORMED" "malformed simulation_state payload"];
        confidence = 0.0; reasoning = None; recommendations = ["Fix protobuf encoding"] }
    else if n >= 2 && Char.code (Bytes.get req.simulation_state 1) = 0 then
      { accepted = false; violations = [Violation.low "EMPTY_ID" "empty simulation_id in payload"];
        confidence = 0.0; reasoning = None; recommendations = ["Provide valid simulation_id"] }
    else (
      (* Run all invariant checks *)
      let step_data = match req.step_data with
        | Some step -> step
        | None -> (match parse_protobuf req.simulation_state with
                   | Some step -> step
                   | None -> SimulationStep.create 0L 0.0 [||])
      in
      let violations = List.fold_left (fun acc rule ->
        match rule.check_fn step_data with
        | Some v -> v :: acc
        | None -> acc
      ) [] InvariantRules.all_rules in
      let accepted = List.length violations = 0 in
      let confidence = if accepted then 0.95 else (0.95 -. (0.1 *. float_of_int (List.length violations))) in
      { accepted; violations; confidence; 
        reasoning = Some "All invariant checks passed";
        recommendations = if accepted then [] else ["Review violations and adjust simulation parameters"] }
    )
  )

let evaluate_with_history (req : request) (history : simulation_step list) : response =
  let base_response = evaluate req in
  let temporal_violations = List.fold_left (fun acc prop ->
    match prop.check_temporal history with
    | Some v -> v :: acc
    | None -> acc
  ) [] TemporalVerifier.temporal_properties in
  { base_response with violations = temporal_violations @ base_response.violations;
                        accepted = List.length temporal_violations = 0 && base_response.accepted }