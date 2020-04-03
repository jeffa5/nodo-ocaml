open Cmdliner

module Cli (S : Nodo.Storage) (F : Nodo.Format) = struct
  module Show = Show.Make (S) (F) (Config.V)
  module Edit = Edit.Make (S) (F)
  module Remove = Remove.Make (S) (F)
  module Sync = Sync.Make (S)
  module Completion = Completion.Make (S) (F) (Config.V)

  let setup_log style_renderer level =
    Fmt_tty.setup_std_outputs ?style_renderer () ;
    Logs.set_level level ;
    Logs.set_reporter (Logs_fmt.reporter ()) ;
    ()

  let setup_log =
    Term.(const setup_log $ Fmt_cli.style_renderer () $ Logs_cli.level ())

  let target_arg =
    let doc = Arg.info ~docv:"TARGET" ~doc:"The target nodo or project." [] in
    Arg.(value & pos 0 string "" doc)

  let exec opts ~formats ~storage =
    ignore formats ;
    ignore storage ;
    let run lwt =
      let l () () r = r in
      Term.(const Lwt_main.run $ (const l $ setup_log $ opts $ lwt))
    in
    let default_cmd =
      let doc = "A note and todo manager." in
      (run Term.(const Show.exec $ const ""), Term.info ~doc "nodo")
    in
    let show_cmd =
      let doc = "Show the project tree or nodo." in
      (run Term.(const Show.exec $ target_arg), Term.info ~doc "show")
    in
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
      ( run Term.(const Edit.exec $ create_arg $ target_arg $ editor_arg)
      , Term.info ~doc "edit" )
    in
    let remove_cmd =
      let doc = "Remove a nodo." in
      let force_arg =
        let doc = "Use force, for removing projects." in
        Arg.(value & flag (info ~doc ["f"]))
      in
      ( run Term.(const Remove.exec $ target_arg $ force_arg)
      , Term.info ~doc "remove" )
    in
    let sync_cmd =
      let doc = "Sync the nodo storage." in
      (run Term.(const Sync.exec $ const ()), Term.info ~doc "sync")
    in
    let completion_cmd =
      let generate_arg =
        let doc =
          "The shell to generate completion scripts for. The value of $(docv) \
           must be "
          ^ Arg.doc_alts ~quoted:true ["zsh"]
          ^ "."
        in
        Arg.(
          value & opt (some string) None (info ~docv:"SHELL" ~doc ["generate"]))
      in
      let all_args = Arg.(value & pos_all string [] (info [])) in
      let doc = "Generate completion scripts and options." in
      ( run Term.(const Completion.exec $ generate_arg $ all_args)
      , Term.info ~doc "completion" )
    in
    let commands = [completion_cmd; show_cmd; edit_cmd; remove_cmd; sync_cmd] in
    Term.(exit @@ eval_choice default_cmd commands)
end
