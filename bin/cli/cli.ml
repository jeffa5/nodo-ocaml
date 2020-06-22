open Cmdliner

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

let () =
  let run lwt =
    let l () r = r in
    Term.(const Lwt_main.run $ (const l $ setup_log $ lwt))
  in
  let default_cmd =
    let doc = "A note and todo manager." in
    (run Term.(const Show.exec $ Show.cmdliner_term), Term.info ~doc "nodo")
  in
  let show_cmd =
    let doc = "Show the project tree or nodo." in
    (run Term.(const Show.exec $ Show.cmdliner_term), Term.info ~doc "show")
  in
  let edit_cmd =
    let doc = "Edit a nodo." in
    (run Term.(const Edit.exec $ Edit.cmdliner_term), Term.info ~doc "edit")
  in
  let remove_cmd =
    let doc = "Remove a nodo." in
    ( run Term.(const Remove.exec $ Remove.cmdliner_term)
    , Term.info ~doc "remove" )
  in
  let sync_cmd =
    let doc = "Sync the nodo storage." in
    (run Term.(const Sync.exec $ Storage.cmdliner_term), Term.info ~doc "sync")
  in
  let completion_cmd =
    let doc = "Generate completion scripts and options." in
    ( run
        Term.(
          const Completion.exec $ Storage.cmdliner_term
          $ Completion.cmdliner_term)
    , Term.info ~doc "completion" )
  in
  let commands = [completion_cmd; show_cmd; edit_cmd; remove_cmd; sync_cmd] in
  Term.(exit @@ eval_choice default_cmd commands)
