let ( let* ) = Lwt.bind

module Make (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let exec target force =
    let target = Astring.String.cuts ~sep:"/" target in
    let* t = Storage.classify target in
    match t with
    | None ->
        Lwt_io.printl "target not found"
    | Some (`Nodo _ as n) -> (
        let* r = Storage.remove n in
        match r with Ok () -> Lwt.return_unit | Error s -> Lwt_io.printl s )
    | Some (`Project _ as p) ->
        if force then
          let* r = Storage.remove p |> Lwt_result.map_err print_endline in
          match r with Ok () -> Lwt.return_unit | Error () -> Lwt.return_unit
        else Lwt_io.printl "Refusing to remove a project without force"
end
