# invariants/ocaml/utils/helpers.ml
(* Utility functions for invariant checking *)

let hash_bytes (data: bytes) : int64 =
  (* FNV-64 hash *)
  let open Int64 in
  let offset = 0xcbf29ce484222325L in
  let prime = 0x100000001b3L in
  let len = Bytes.length data in
  let rec loop i h =
    if i >= len then h
    else loop (i + 1) (mul (logand (add h (of_int (Char.code (Bytes.get data i)))) 0xFFFFFFFFFFFFFFFFL) prime)
  in
  loop 0 offset

let validate_field_dimensions (width: int) (height: int) : bool =
  width >= 2 && height >= 2

let check_finite_array (arr: float array) : bool =
  Array.for_all Float.is_finite arr