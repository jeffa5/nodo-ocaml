module Result = Stdlib.Result

let ( let* ) = Lwt.bind

let exec (conf : Storage.config) =
  let* r = Storage.sync conf in
  match r with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e
