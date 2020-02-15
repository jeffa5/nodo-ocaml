open Cmdliner

module Cli (S : Nodo.Storage) (F : Nodo.Format) = struct
  module Show = Show.Make (S) (F) (Config.V)
  module Edit = Edit.Make (S) (F)
  module Remove = Remove.Make (S) (F)

  let target_arg =
    let doc = Arg.info ~docv:"TARGET" ~doc:"The target nodo or project" [] in
    Arg.(value & pos 0 string "" doc)

  let default_cmd =
    let doc = "A note and todo manager." in
    (Term.(const Show.exec $ const ""), Term.info ~doc "nodo")

  let show_cmd =
    let doc = "Show the project tree or nodo." in
    (Term.(const Show.exec $ target_arg), Term.info ~doc "show")

  let edit_cmd =
    let doc = "Edit a nodo." in
    let create_arg = Arg.(value & flag (info ["c"])) in
    (Term.(const Edit.exec $ create_arg $ target_arg), Term.info ~doc "edit")

  let remove_cmd =
    let doc = "Remove a nodo." in
    let force_arg =
      let doc = "Use force, for removing projects." in
      Arg.(value & flag (info ~doc ["f"]))
    in
    (Term.(const Remove.exec $ target_arg $ force_arg), Term.info ~doc "remove")

  let commands = [show_cmd; edit_cmd; remove_cmd]

  let exec ~formats ~storage =
    ignore formats ;
    ignore storage ;
    Term.(exit @@ eval_choice default_cmd commands)
end
