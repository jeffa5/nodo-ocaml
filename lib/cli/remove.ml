module Make (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let exec target force =
    let target = Astring.String.cuts ~sep:"/" target in
    match Storage.classify target with
    | None ->
        print_endline "target not found"
    | Some (`Nodo _ as n) ->
        Storage.remove n
    | Some (`Project _ as p) ->
        if force then Storage.remove p
        else print_endline "Refusing to remove a project without force"
end
