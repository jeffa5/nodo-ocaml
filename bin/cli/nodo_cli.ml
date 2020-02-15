module Prefix : Nodo_filesystem.Prefix_type = struct
  let prefix = ["/home"; "andrew"; ".nodo"]
end

module Fs = Nodo_filesystem.Make (Prefix)
module Cli = Nodo_cli_lib.Cli (Fs) (Nodo_markdown)

let () = Cli.exec ~formats:[] ~storage:[]
