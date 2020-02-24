module Result = Stdlib.Result

let ( let* ) = Lwt.bind

module Make (Storage : Nodo.Storage) = struct
  let exec () =
    let* r = Storage.sync () in
    match r with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e
end
