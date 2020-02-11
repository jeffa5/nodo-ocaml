module S (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let exec target =
    match Storage.classify target with
    | None -> print_endline "target not found"
    | Some target -> Storage.remove target
end
