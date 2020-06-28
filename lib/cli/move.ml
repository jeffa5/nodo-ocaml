let ( let* ) = Lwt.bind

let ( let> ) = Lwt_result.bind

let ( let^ ) = Lwt_result.map

type config = {global: Config.t; from: string; to_: string} [@@deriving show]

let cmdliner_term =
  let build global from to_ = {global; from; to_} in
  let open Cmdliner in
  let from =
    let doc = Arg.info ~docv:"FROM" ~doc:"The source nodo to move" [] in
    Arg.(required & pos 0 (some string) None doc)
  in
  let to_ =
    let doc = Arg.info ~docv:"TO" ~doc:"The destination nodo" [] in
    Arg.(required & pos 1 (some string) None doc)
  in
  Term.(const build $ Config.cmdliner_term $ from $ to_)

module Make (C : sig
  val t : config
end) =
struct
  module Storage = Storage.Make (struct
    let t = C.t.global.storage
  end)

  let move from =
    let> content = Storage.read from in
    let> () = Storage.remove from in
    let> n = Storage.create C.t.to_ in
    Storage.write n content

  let exec () =
    let* t = Storage.classify C.t.from in
    match t with
    | None -> (
        let target =
          Storage.with_extension C.t.from ~ext:C.t.global.format_ext
        in
        let* t = Storage.classify target in
        match t with
        | None ->
            Lwt_io.printl "from not found"
        | Some (`Nodo _ as n) -> (
            let* r = move n in
            match r with Error e -> Lwt_io.printl e | Ok () -> Lwt.return_unit )
        | Some (`Project _) ->
            Lwt_io.printl "can't move a project" )
    | Some (`Nodo _ as n) -> (
        let* r = move n in
        match r with Error e -> Lwt_io.printl e | Ok () -> Lwt.return_unit )
    | Some (`Project _) ->
        Lwt_io.printl "can't move a project"
end
