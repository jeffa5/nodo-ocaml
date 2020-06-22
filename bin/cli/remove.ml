let ( let* ) = Lwt.bind

type config = {global: Config.t; target: string; force: bool}

let cmdliner_term =
  let build global force target = {global; force; target} in
  let open Cmdliner in
  let force =
    let doc = "Use force, for removing projects." in
    Arg.(value & flag (info ~doc ["f"]))
  in
  let target =
    let doc = "The target to show" in
    Arg.(required & pos 0 (some string) None & info [] ~doc)
  in
  Term.(const build $ Config.cmdliner_term $ force $ target)

let remove_nodo (gconf : Config.t) n =
  let* r = Storage.remove gconf.storage n in
  match r with Ok () -> Lwt.return_unit | Error s -> Lwt_io.printl s

let remove_project conf p =
  if conf.force then
    let* r =
      Storage.remove conf.global.storage p |> Lwt_result.map_err print_endline
    in
    match r with Ok () -> Lwt.return_unit | Error () -> Lwt.return_unit
  else Lwt_io.printl "Refusing to remove a project without force"

let exec conf =
  match conf.target with
  | "" ->
      Lwt_io.printl "TARGET cannot be empty"
  | _ -> (
      let* t = Storage.classify conf.global.storage conf.target in
      match t with
      | None -> (
          let target =
            Storage.with_extension conf.target (List.hd Format.formats)
          in
          let* t = Storage.classify conf.global.storage target in
          match t with
          | None ->
              Lwt_io.printl "target not found"
          | Some (`Nodo _ as n) ->
              remove_nodo conf.global n
          | Some (`Project _ as p) ->
              remove_project conf p )
      | Some (`Nodo _ as n) ->
          remove_nodo conf.global n
      | Some (`Project _ as p) ->
          remove_project conf p )
