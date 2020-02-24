let ( let* ) = Lwt.bind

module Make (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let remove_nodo n =
    let* r = Storage.remove n in
    match r with Ok () -> Lwt.return_unit | Error s -> Lwt_io.printl s

  let remove_project ~force p =
    if force then
      let* r = Storage.remove p |> Lwt_result.map_err print_endline in
      match r with Ok () -> Lwt.return_unit | Error () -> Lwt.return_unit
    else Lwt_io.printl "Refusing to remove a project without force"

  let exec target force =
    match target with
    | "" ->
        Lwt_io.printl "TARGET cannot be empty"
    | _ -> (
        let target = Astring.String.cuts ~sep:"/" target in
        let* t = Storage.classify target in
        match t with
        | None -> (
            let target =
              Storage.with_extension target (List.hd Format.extensions)
            in
            let* t = Storage.classify target in
            match t with
            | None ->
                Lwt_io.printl "target not found"
            | Some (`Nodo _ as n) ->
                remove_nodo n
            | Some (`Project _ as p) ->
                remove_project ~force p )
        | Some (`Nodo _ as n) ->
            remove_nodo n
        | Some (`Project _ as p) ->
            remove_project ~force p )
end
