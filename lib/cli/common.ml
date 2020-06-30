open Cmdliner

let target_arg =
  let doc = Arg.info ~docv:"TARGET" ~doc:"The target nodo or project." [] in
  Arg.(value & pos 0 string "" doc)

let required_target_arg =
  let doc = Arg.info ~docv:"TARGET" ~doc:"The target nodo or project." [] in
  Arg.(required & pos 0 (some string) None doc)
