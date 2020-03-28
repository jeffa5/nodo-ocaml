module Nodo = Nodo.S

(* type nodo = Nodo.t [@@deriving yojson] *)

let extensions = ["json"]

let parse _content = (({due_date= ""} : Nodo.metadata), [])

let render _nodo = ""
