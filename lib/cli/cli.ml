open Cmdliner

let setup_log =
  let setup style_renderer level =
    Fmt_tty.setup_std_outputs ?style_renderer () ;
    Logs.set_level level ;
    Logs.set_reporter (Logs_fmt.reporter ())
  in
  let docs = Manpage.s_common_options in
  Term.(const setup $ Fmt_cli.style_renderer ~docs () $ Logs_cli.level ~docs ())

let main () =
  let run lwt =
    let l () r = r in
    Term.(const Lwt_main.run $ (const l $ setup_log $ lwt))
  in
  let default_cmd =
    let doc = "A note and todo manager." in
    let cmd c =
      Logs.debug (fun f -> f "Using config: %a" Show.pp_config c) ;
      let module E = Show.Make (struct
        let t = c
      end) in
      E.exec ()
    in
    (run Term.(const cmd $ Show.cmdliner_term), Term.info ~doc "nodo")
  in
  let show =
    let doc = "Show the project tree or nodo." in
    let cmd c =
      Logs.debug (fun f -> f "Using config: %a" Show.pp_config c) ;
      let module E = Show.Make (struct
        let t = c
      end) in
      E.exec ()
    in
    (run Term.(const cmd $ Show.cmdliner_term), Term.info ~doc "show")
  in
  let edit =
    let doc = "Edit a nodo." in
    let cmd c =
      Logs.debug (fun f -> f "Using config: %a" Edit.pp_config c) ;
      let module E = Edit.Make (struct
        let t = c
      end) in
      E.exec ()
    in
    (run Term.(const cmd $ Edit.cmdliner_term), Term.info ~doc "edit")
  in
  let remove =
    let doc = "Remove a nodo." in
    let cmd c =
      Logs.debug (fun f -> f "Using config: %a" Remove.pp_config c) ;
      let module E = Remove.Make (struct
        let t = c
      end) in
      E.exec ()
    in
    (run Term.(const cmd $ Remove.cmdliner_term), Term.info ~doc "remove")
  in
  let move =
    let doc = "Move a nodo." in
    let cmd c =
      Logs.debug (fun f -> f "Using config: %a" Move.pp_config c) ;
      let module E = Move.Make (struct
        let t = c
      end) in
      E.exec ()
    in
    (run Term.(const cmd $ Move.cmdliner_term), Term.info ~doc "move")
  in
  let sync =
    let doc = "Sync the nodo storage." in
    let cmd c =
      Logs.debug (fun f -> f "Using config: %a" Sync.pp_config c) ;
      let module E = Sync.Make (struct
        let t = c
      end) in
      E.exec ()
    in
    (run Term.(const cmd $ Sync.cmdliner_term), Term.info ~doc "sync")
  in
  let completion =
    let doc = "Generate completion scripts and options." in
    let cmd c =
      Logs.debug (fun f -> f "Using config: %a" Completion.pp_config c) ;
      let module E = Completion.Make (struct
        let t = c
      end) in
      E.exec ()
    in
    ( run Term.(const cmd $ Completion.cmdliner_term)
    , Term.info ~doc "completion" )
  in
  let commands = [completion; show; edit; remove; move; sync] in
  Term.(exit @@ eval_choice default_cmd commands)
