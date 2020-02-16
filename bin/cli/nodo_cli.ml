(* module Prefix : Nodo_filesystem.Prefix_type = struct *)
(* let prefix = ["/home"; "andrew"; ".nodo"] *)
(* end *)

module Remote : Nodo_git_filesystem.Remote = struct
  let remote = "git://github.com/jeffat/nodo-test.git"
end

(* module Fs = Nodo_filesystem.Make (Prefix) *)
module Git = Nodo_git_filesystem (Remote)
module Cli = Nodo_cli_lib.Cli (Git) (Nodo_markdown)

let () = Cli.exec ~formats:[] ~storage:[]
