module Make (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let edit (nodo : Storage.nodo) =
    (* TODO: need to get storage to render the nodo in the format to a temp file, then vim that file then when they exit need to read the file again and write back to the location through storage and format (should do auto formatting for us) *)
    let path = String.concat "/" (Storage.location nodo) in
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
    | Some (`Nodo _ as n) ->
        edit n
    | Some (`Project _) ->
        print_endline "Unable to edit a project"
end
