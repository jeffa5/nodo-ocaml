type t = {hidden_dirs: string list; storage: Storage.config; format_ext: string}
[@@deriving make, show]

let cmdliner_term =
  let build hidden_dirs storage format_ext =
    {hidden_dirs; storage; format_ext}
  in
  let open Cmdliner in
  let docs = Manpage.s_common_options in
  let hidden_dirs =
    let doc = "The list of dirs to ignore" in
    Arg.(
      value
      & opt (list string) ["archive"; "temp"]
      & info ["hidden-dirs"] ~docs ~doc)
  in
  let format_ext =
    let doc = "Extension of the desired format" in
    Arg.(value & opt string "md" & info ["format"] ~docs ~doc)
  in
  Term.(const build $ hidden_dirs $ Storage.cmdliner_term $ format_ext)
