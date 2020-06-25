module Result = Stdlib.Result

let ( let* ) = Lwt.bind

type config = {global: Config.t}

let cmdliner_term =
  let build global = {global} in
  let open Cmdliner in
  Term.(const build $ Config.cmdliner_term)

module Make (C : sig
  val t : config
end) =
struct
  module Storage = Storage.Make (struct
    let t = C.t.global.storage
  end)

  let exec () =
    let* r = Storage.sync () in
    match r with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e
end
