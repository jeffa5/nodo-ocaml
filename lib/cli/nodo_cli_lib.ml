open Cmdliner

let target_arg =
  let doc = Arg.info ~docv:"TARGET" ~doc:"The target to use" [] in
  Arg.(value & pos 0 string "" doc)

module Prefix : Nodo_filesystem.Prefix_type = struct
  let prefix = "/home/andrew/.nodo"
end

module Fs = Nodo_filesystem.Make (Prefix)
module Show = Show.S (Fs) (Nodo_markdown) (Config.V)
module Edit = Edit.S (Fs) (Nodo_markdown)
module Remove = Remove.S (Fs) (Nodo_markdown)

let show_cmd = (Term.(const Show.exec $ target_arg), Term.info "show")

let edit_cmd =
  let create_arg = Arg.(value & flag (info [ "c" ])) in
  (Term.(const Edit.exec $ create_arg $ target_arg), Term.info "edit")

let remove_cmd = (Term.(const Remove.exec $ target_arg), Term.info "remove")

let commands = [ show_cmd; edit_cmd; remove_cmd ]

let exec ~formats ~storage =
  ignore formats;
  ignore storage;
  Term.(exit @@ eval_choice show_cmd commands)
