open Stdlib.Option
open Cmdliner

module Config = struct
  let dir = ref ""

  let remote = ref ""

  let user = ref ""

  let remote_headers = ref None

  let author = user

  let create_headers u p =
    let e = Cohttp.Header.of_list [] in
    Cohttp.Header.add_authorization e (`Basic (u, p)) |> some

  let make_config d r u p =
    dir := d ;
    remote := r ;
    user := u ;
    remote_headers := create_headers u p
end

let config_args =
  let dir_arg =
    let home =
      match Sys.getenv_opt "HOME" with Some h -> h | None -> "/tmp"
    in
    let doc = "Directory to store the nodos." in
    Arg.(value & opt string (home ^ "/.nodo") & info ~docv:"DIR" ~doc ["d"])
  in
  let remote_arg =
    let env = Arg.env_var "NODO_SYNC_REMOTE" in
    let doc = "Remote to sync with." in
    Arg.(value & opt string "" & info ~env ~docv:"REMOTE" ~doc ["r"])
  in
  let user_arg =
    let env = Arg.env_var "NODO_SYNC_USER" in
    let doc = "Username to use for syncing and authoring commits." in
    Arg.(value & opt string "" & info ~env ~docv:"USER" ~doc ["u"])
  in
  let pass_arg =
    let env = Arg.env_var "NODO_SYNC_PASS" in
    let doc = "Password to use for syncing." in
    Arg.(value & opt string "" & info ~env ~docv:"PASS" ~doc ["p"])
  in
  Term.(const Config.make_config $ dir_arg $ remote_arg $ user_arg $ pass_arg)

module Git = Nodo_git_filesystem.Make (Config)
module Cli = Nodo_cli_lib.Cli (Git) (Nodo_markdown)

let () = Cli.exec config_args ~formats:[] ~storage:[]
