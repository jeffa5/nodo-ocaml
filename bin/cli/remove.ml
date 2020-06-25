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

module Make (C : sig
  val t : config
end) =
struct
  module Storage = Storage.Make (struct
    let t = C.t.global.storage
  end)

  let remove_nodo n =
    let* r = Storage.remove n in
    match r with Ok () -> Lwt.return_unit | Error s -> Lwt_io.printl s

  let remove_project p =
    if C.t.force then
      let* r = Storage.remove p |> Lwt_result.map_err print_endline in
      match r with Ok () -> Lwt.return_unit | Error () -> Lwt.return_unit
    else Lwt_io.printl "Refusing to remove a project without force"

  let exec () =
    match C.t.target with
    | "" ->
        Lwt_io.printl "TARGET cannot be empty"
    | _ -> (
        let* t = Storage.classify C.t.target in
        match t with
        | None -> (
            let target =
              Storage.with_extension C.t.target (List.hd Format.formats)
            in
            let* t = Storage.classify target in
            match t with
            | None ->
                Lwt_io.printl "target not found"
            | Some (`Nodo _ as n) ->
                remove_nodo n
            | Some (`Project _ as p) ->
                remove_project p )
        | Some (`Nodo _ as n) ->
            remove_nodo n
        | Some (`Project _ as p) ->
            remove_project p )
end