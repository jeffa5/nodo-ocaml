let extensions = ["json"]

let parse content =
  try
    match Nodo_core.S.of_yojson @@ Yojson.Safe.from_string content with
    | Ok n ->
        n
    | Error _s ->
        Nodo_core.S.make ~metadata:(Nodo_core.S.make_metadata ()) ()
  with Yojson.Json_error _s ->
    Nodo_core.S.make ~metadata:(Nodo_core.S.make_metadata ()) ()

let render nodo = Yojson.Safe.to_string ~std:true (Nodo_core.S.to_yojson nodo)

let test_parse t = parse t |> Nodo_core.S.show |> print_endline

let test_render n = render n |> print_endline

let%expect_test "Empty text gives empty nodo" =
  test_parse "" ; [%expect {| { metadata = { due_date = "" }; blocks = [] } |}]

let%expect_test "render metadata" =
  test_render (Nodo_core.S.make ~metadata:(Nodo_core.S.make_metadata ()) ()) ;
  [%expect {| {"metadata":{},"blocks":[]} |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ~due_date:"test" ())
       ()) ;
  [%expect {|
    {"metadata":{"due_date":"test"},"blocks":[]} |}]
