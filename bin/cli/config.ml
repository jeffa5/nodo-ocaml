type t = {hidden_dirs: string list; storage: Storage.config} [@@deriving make]

let cmdliner_term =
  let build hidden_dirs storage = {hidden_dirs; storage} in
  let open Cmdliner in
  let docs = Manpage.s_common_options in
  let hidden_dirs =
    let doc = "The list of dirs to ignore" in
    Arg.(
      value
      & opt (list string) ["archive"; "temp"]
      & info ["hidden-dirs"] ~docs ~doc)
  in
  Term.(const build $ hidden_dirs $ Storage.cmdliner_term)
