let dir = "/root/.nodo"

let with_storage ?(dir = dir) f =
  let conf = Nodo_cli_lib.Storage.make_config ~dir in
  let s =
    ( module Nodo_cli_lib.Storage.Make (struct
      let t = conf
    end) : Nodo_cli_lib.Storage.S )
  in
  f s

let test_name =
  let open Alcotest_lwt in
  ( "name"
  , [ test_case "from nodo" `Quick (fun _ _ ->
          with_storage (fun (module S) ->
              Alcotest.(check string) "equal" "nodo" (S.name (`Nodo "nodo"))
              |> Lwt.return))
    ; test_case "from project" `Quick (fun _ _ ->
          with_storage (fun (module S) ->
              Alcotest.(check string)
                "equal" "project"
                (S.name (`Project "project"))
              |> Lwt.return))
    ; test_case "from path" `Quick (fun _ _ ->
          with_storage (fun (module S) ->
              Alcotest.(check string)
                "equal" "project/nodo"
                (S.name (`Nodo "project/nodo"))
              |> Lwt.return)) ] )

let test_location =
  let open Alcotest_lwt in
  ( "location"
  , [ test_case "from nodo" `Quick (fun _ _ ->
          with_storage (fun (module S) ->
              Alcotest.(check string) "equal" "nodo" (S.location (`Nodo "nodo"))
              |> Lwt.return))
    ; test_case "from project" `Quick (fun _ _ ->
          with_storage (fun (module S) ->
              Alcotest.(check string)
                "equal" "project"
                (S.location (`Project "project"))
              |> Lwt.return))
    ; test_case "from location" `Quick (fun _ _ ->
          with_storage (fun (module S) ->
              Alcotest.(check string)
                "equal" "project/nodo"
                (S.location (`Nodo "project/nodo"))
              |> Lwt.return)) ] )

let test_path =
  let open Alcotest_lwt in
  ( "path"
  , [ test_case "from nodo" `Quick (fun _ _ ->
          with_storage (fun (module S) ->
              Alcotest.(check string)
                "equal"
                (Filename.concat dir "nodo")
                (S.path (`Nodo "nodo"))
              |> Lwt.return))
    ; test_case "from project" `Quick (fun _ _ ->
          with_storage (fun (module S) ->
              Alcotest.(check string)
                "equal"
                (Filename.concat dir "project")
                (S.path (`Project "project"))
              |> Lwt.return))
    ; test_case "from path" `Quick (fun _ _ ->
          with_storage (fun (module S) ->
              Alcotest.(check string)
                "equal"
                (Filename.concat dir "project/nodo")
                (S.path (`Nodo "project/nodo"))
              |> Lwt.return)) ] )

let suite () =
  let open Alcotest_lwt in
  run "Storage" [test_name; test_location; test_path]
