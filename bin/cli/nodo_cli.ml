(* module Prefix : Nodo_filesystem.Prefix_type = struct *)
(*   let prefix = ["/home"; "andrew"; ".nodo"] *)
(* end *)

(* module Fs = Nodo_filesystem.Make (Prefix) *)
(* module Cli = Nodo_cli_lib.Cli (Fs) (Nodo_markdown) *)

module Config : Nodo_git_filesystem.Config = struct
  let dir = "/home/andrew/.nodo-git"

  (* let remote = "git://github.com/jeffat/nodo-test.git" *)
  let remote = ""

  let author = "andrew <dev@jeffas.io>"
end

module Git = Nodo_git_filesystem.Make (Config)
module Cli = Nodo_cli_lib.Cli (Git) (Nodo_markdown)

let () =
  Logs.set_reporter (Logs_fmt.reporter ()) ;
  (* Logs.set_level (Some Logs.Debug) ; *)
  Cli.exec ~formats:[] ~storage:[]
