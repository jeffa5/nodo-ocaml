module Make (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let edit (`Nodo nodo) =
    let path = String.concat "/" nodo in
    let _ = Sys.command @@ "vim " ^ path in
    ()

  let exec create target =
    let extension = List.hd Format.extensions in
    let target =
      if target = "" then Sys.getcwd () ^ "/.nodo." ^ extension else target
    in
    let open Astring in
    let target = String.cuts ~sep:"/" target in
    match Storage.classify target with
    | None ->
        if create then edit (Storage.create target)
    | Some target -> (
      match target with
      | `Nodo _ as n ->
          edit n
      | `Project _ ->
          print_endline "Unable to edit a project" )
end
