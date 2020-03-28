type metadata = [%import: Nodo.S.metadata] [@@deriving yojson]

type text_type = [%import: Nodo.S.text_type] [@@deriving yojson]

type text_item = [%import: Nodo.S.text_item] [@@deriving yojson]

type text = [%import: Nodo.S.text] [@@deriving yojson]

type list_item = [%import: Nodo.S.list_item] [@@deriving yojson]

type list_ = [%import: Nodo.S.list_] [@@deriving yojson]

type block = [%import: Nodo.S.block] [@@deriving yojson]

type nodo = [%import: Nodo.S.t] [@@deriving yojson]

module Nodo = Nodo.S

let extensions = ["json"]

let parse content =
  try
    match nodo_of_yojson @@ Yojson.Safe.from_string content with
    | Ok n ->
        n
    | Error _s ->
        Nodo.make ~metadata:(Nodo.make_metadata ()) ()
  with Yojson.Json_error _s -> Nodo.make ~metadata:(Nodo.make_metadata ()) ()

let render nodo = Yojson.Safe.to_string ~std:true (nodo_to_yojson nodo)

let test_parse t = parse t |> Nodo.show |> print_endline

let test_render n = render n |> print_endline

let%expect_test "Empty text gives empty nodo" =
  test_parse "" ; [%expect {| { metadata = { due_date = "" }; blocks = [] } |}]

let%expect_test "render metadata" =
  test_render (Nodo.make ~metadata:(Nodo.make_metadata ()) ()) ;
  [%expect {| {"metadata":{},"blocks":[]} |}] ;
  test_render (Nodo.make ~metadata:(Nodo.make_metadata ~due_date:"test" ()) ()) ;
  [%expect {|
    {"metadata":{"due_date":"test"},"blocks":[]} |}]
