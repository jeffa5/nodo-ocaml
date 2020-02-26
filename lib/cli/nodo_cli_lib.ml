open Cmdliner

module Cli (S : Nodo.Storage) (F : Nodo.Format) = struct
  module Show = Show.Make (S) (F) (Config.V)
  module Edit = Edit.Make (S) (F)
  module Remove = Remove.Make (S) (F)
  module Format = Format.Make (S) (F)
  module Sync = Sync.Make (S)

  let target_arg =
    let doc = Arg.info ~docv:"TARGET" ~doc:"The target nodo or project." [] in
    Arg.(value & pos 0 string "" doc)

  let default_cmd =
    let doc = "A note and todo manager." in
    ( Term.(const Lwt_main.run $ (const Show.exec $ const ""))
    , Term.info ~doc "nodo" )

  let show_cmd =
    let doc = "Show the project tree or nodo." in
    ( Term.(const Lwt_main.run $ (const Show.exec $ target_arg))
    , Term.info ~doc "show" )

  let edit_cmd =
    let doc = "Edit a nodo." in
    let create_arg =
      let doc = "Create the nodo if it does not exist." in
      Arg.(value & flag (info ~doc ["c"]))
    in
    let editor_arg =
      let doc =
        "The editor to launch when editing. The command will be run as "
        ^ Arg.doc_quote "$(docv) file"
      in
      let env = Arg.env_var "NODO_EDITOR" in
      Arg.(value & opt string "vim" & info ~env ~docv:"EDITOR" ~doc ["e"])
    in
    ( Term.(
        const Lwt_main.run
        $ (const Edit.exec $ create_arg $ target_arg $ editor_arg))
    , Term.info ~doc "edit" )

  let remove_cmd =
    let doc = "Remove a nodo." in
    let force_arg =
      let doc = "Use force, for removing projects." in
      Arg.(value & flag (info ~doc ["f"]))
    in
    ( Term.(const Lwt_main.run $ (const Remove.exec $ target_arg $ force_arg))
    , Term.info ~doc "remove" )

  let format_cmd =
    let doc = "Format a project or nodo." in
    ( Term.(const Lwt_main.run $ (const Format.exec $ target_arg))
    , Term.info ~doc "format" )

  let sync_cmd =
    let doc = "Sync the nodo storage." in
    ( Term.(const Lwt_main.run $ (const Sync.exec $ const ()))
    , Term.info ~doc "sync" )

  let commands = [show_cmd; edit_cmd; remove_cmd; format_cmd; sync_cmd]

  let exec ~formats ~storage =
    ignore formats ;
    ignore storage ;
    Term.(exit @@ eval_choice default_cmd commands)
end
