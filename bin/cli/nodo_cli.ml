open Stdlib.Option
open Cmdliner

module Config = struct
  let dir = ref []

  let remote = ref ""

  let author = ref ""

  let make_config d r u =
    let d = String.split_on_char '/' d in
    dir := d ;
    remote := r ;
    author := u
end

let config_args =
  let dir_arg =
    let home =
      match Sys.getenv_opt "HOME" with Some h -> h | None -> "/tmp"
    in
    let doc = "Directory to store the nodos." in
    Arg.(value & opt string (home ^ "/.nodo") & info ~docv:"DIR" ~doc ["dir"])
  in
  let remote_arg =
    let env = Arg.env_var "NODO_SYNC_REMOTE" in
    let doc = "Remote to sync with." in
    Arg.(value & opt string "" & info ~env ~docv:"REMOTE" ~doc ["remote"])
  in
  let author_arg =
    let env = Arg.env_var "NODO_SYNC_AUTHOR" in
    let doc = "Username to use for syncing and authoring commits." in
    Arg.(value & opt string "" & info ~env ~docv:"USER" ~doc ["user"])
  in
  Term.(const Config.make_config $ dir_arg $ remote_arg $ author_arg)

module Git = Nodo_git_binary.Make (Config)
module Cli = Nodo_cli_lib.Cli (Git) (Nodo_markdown)

let () = Cli.exec config_args ~formats:[] ~storage:[]
