open Stdlib.Option

module Config : Nodo_git_filesystem.Config = struct
  let dir =
    match Sys.getenv_opt "NODO_STORE_DIR" with
    | None ->
        print_endline "Must provide nodo store dir" ;
        exit 1
    | Some s ->
        s

  let remote =
    match Sys.getenv_opt "NODO_SYNC_REMOTE" with
    | None ->
        print_endline "Must provide sync remote environment variable" ;
        exit 1
    | Some s ->
        s

  let user =
    match Sys.getenv_opt "NODO_SYNC_USER" with
    | None ->
        print_endline "Must provide user environment variable" ;
        exit 1
    | Some s ->
        s

  let remote_headers =
    let e = Cohttp.Header.of_list [] in
    let pass =
      match Sys.getenv_opt "NODO_SYNC_PASS" with
      | None ->
          print_endline "Must provide pass environment variable" ;
          exit 1
      | Some s ->
          s
    in
    Cohttp.Header.add_authorization e (`Basic (user, pass)) |> some

  let author = user
end

module Git = Nodo_git_filesystem.Make (Config)
module Cli = Nodo_cli_lib.Cli (Git) (Nodo_markdown)

let () =
  Logs.set_reporter (Logs_fmt.reporter ()) ;
  (* Logs.set_level (Some Logs.Debug) ; *)
  Cli.exec ~formats:[] ~storage:[]
