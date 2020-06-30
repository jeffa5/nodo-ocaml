open Lwt.Syntax

let test_nodo =
  Alcotest.testable Nodo_cli_lib.Storage.pp_nodo Nodo_cli_lib.Storage.equal_nodo

let test_t =
  Alcotest.testable Nodo_cli_lib.Storage.pp Nodo_cli_lib.Storage.equal

let with_storage f =
  Lwt_io.with_temp_dir (fun dir ->
      let conf = Nodo_cli_lib.Storage.make_config ~dir in
      let s =
        ( module Nodo_cli_lib.Storage.Make (struct
          let t = conf
        end) : Nodo_cli_lib.Storage.S )
      in
      f dir s)

let test_name =
  let open Alcotest_lwt in
  let cases =
    [ ("from nodo", "nodo", `Nodo "nodo")
    ; ("from project", "project", `Project "project")
    ; ("from path", "nodo", `Nodo "project/nodo") ]
  in
  ( "name"
  , List.map
      (fun (n, f, t) ->
        test_case n `Quick (fun _ _ ->
            with_storage (fun _ (module S) ->
                Alcotest.(check string) "equal" f (S.name t) |> Lwt.return)))
      cases )

let test_location =
  let open Alcotest_lwt in
  let cases =
    [ ("from nodo", "nodo", `Nodo "nodo")
    ; ("from project", "project", `Project "project")
    ; ("from location", "project/nodo", `Nodo "project/nodo") ]
  in
  ( "location"
  , List.map
      (fun (n, f, t) ->
        test_case n `Quick (fun _ _ ->
            with_storage (fun _ (module S) ->
                Alcotest.(check string) "equal" f (S.location t) |> Lwt.return)))
      cases )

let test_path =
  let open Alcotest_lwt in
  let cases =
    [ ("from nodo", "nodo", `Nodo "nodo")
    ; ("from project", "project", `Project "project")
    ; ("from path", "project/nodo", `Nodo "project/nodo") ]
  in
  ( "path"
  , List.map
      (fun (n, f, t) ->
        test_case n `Quick (fun _ _ ->
            with_storage (fun dir (module S) ->
                Alcotest.(check string)
                  "equal" (Filename.concat dir f) (S.path t)
                |> Lwt.return)))
      cases )

let test_with_extension =
  let open Alcotest_lwt in
  let cases =
    [ ("no extension", "file")
    ; ("empty extension", "file.")
    ; ("other extension", "file.other")
    ; ("same extension", "file.ext") ]
  in
  ( "with_extension"
  , List.map
      (fun (n, f) ->
        test_case n `Quick (fun _ _ ->
            with_storage (fun _ (module S) ->
                Alcotest.(check string)
                  "equal" "file.ext"
                  (S.with_extension ~ext:"ext" f)
                |> Lwt.return)))
      cases )

let test_classify =
  let open Alcotest_lwt in
  ( "classify"
  , [ test_case "missing string" `Quick (fun _ _ ->
          with_storage (fun _ (module S) ->
              let+ o = S.classify "missing" in
              Alcotest.(check (option test_t)) "equal" None o))
    ; test_case "existing file" `Quick (fun _ _ ->
          with_storage (fun temp_dir (module S) ->
              Lwt_io.with_temp_file ~temp_dir (fun (n, _) ->
                  let fname = Filename.basename n in
                  let+ o = S.classify fname in
                  Alcotest.(check (option test_t))
                    "equal"
                    (Some (`Nodo fname))
                    o)))
    ; test_case "existing dir" `Quick (fun _ _ ->
          with_storage (fun temp_dir (module S) ->
              Lwt_io.with_temp_dir ~parent:temp_dir (fun n ->
                  let fname = Filename.basename n in
                  let+ o = S.classify fname in
                  Alcotest.(check (option test_t))
                    "equal"
                    (Some (`Project fname))
                    o))) ] )

let test_list =
  let open Alcotest_lwt in
  ( "list"
  , [ test_case "empty dir" `Quick (fun _ _ ->
          with_storage (fun _ (module S) ->
              let+ r = S.list (`Project "") in
              Alcotest.(check (result (list test_t) string)) "equal" (Ok []) r))
    ; test_case "one file" `Quick (fun _ _ ->
          with_storage (fun temp_dir (module S) ->
              Lwt_io.with_temp_file ~temp_dir (fun (n, _) ->
                  let+ r = S.list (`Project "") in
                  Alcotest.(check (result (list test_t) string))
                    "equal"
                    (Ok [`Nodo (Filename.basename n)])
                    r)))
    ; test_case "one dir" `Quick (fun _ _ ->
          with_storage (fun temp_dir (module S) ->
              Lwt_io.with_temp_dir ~parent:temp_dir (fun n ->
                  let+ r = S.list (`Project "") in
                  Alcotest.(check (result (list test_t) string))
                    "equal"
                    (Ok [`Project (Filename.basename n)])
                    r)))
    ; test_case "one file and one dir" `Quick (fun _ _ ->
          with_storage (fun temp_dir (module S) ->
              Lwt_io.with_temp_file ~temp_dir (fun (f, _) ->
                  Lwt_io.with_temp_dir ~parent:temp_dir (fun d ->
                      let+ r = S.list (`Project "") in
                      Alcotest.(check (result (list test_t) string))
                        "equal"
                        (Ok
                           [ `Project (Filename.basename d)
                           ; `Nodo (Filename.basename f) ])
                        r)))) ] )

let test_create =
  let open Alcotest_lwt in
  ( "create"
  , [ test_case "empty arg" `Quick (fun _ _ ->
          with_storage (fun _ (module S) ->
              let+ r = S.create "" in
              Alcotest.(check (result test_nodo string))
                "equal" (Error "Nodo with empty path cannot exist") r))
    ; test_case "standard file" `Quick (fun _ _ ->
          with_storage (fun _ (module S) ->
              let+ r = S.create "test" in
              Alcotest.(check (result test_nodo string))
                "equal"
                (Ok (`Nodo "test"))
                r)) ] )

let test_write =
  let open Alcotest_lwt in
  ( "write"
  , [ test_case "write empty file" `Quick (fun _ _ ->
          with_storage (fun _ (module S) ->
              let* r = S.write (`Nodo "test") "" in
              Alcotest.(check (result unit string)) "write ok" (Ok ()) r ;
              let+ r = S.read (`Nodo "test") in
              Alcotest.(check (result string string)) "read ok" (Ok "") r))
    ; test_case "write small file" `Quick (fun _ _ ->
          with_storage (fun _ (module S) ->
              let* r = S.write (`Nodo "test") "test" in
              Alcotest.(check (result unit string)) "write ok" (Ok ()) r ;
              let+ r = S.read (`Nodo "test") in
              Alcotest.(check (result string string)) "read ok" (Ok "test") r))
    ] )

let suite () =
  let open Alcotest_lwt in
  run "Storage"
    [ test_name
    ; test_location
    ; test_path
    ; test_with_extension
    ; test_classify
    ; test_list
    ; test_create
    ; test_write ]
