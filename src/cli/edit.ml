module S (Storage : Nodo_core.Storage) (Format : Nodo_core.Format) = struct
  let edit (`Nodo nodo) =
    let _ = Sys.command @@ "vim " ^ nodo in
    ()

  let exec create target =
    match Storage.classify target with
    | None -> if create then edit (Storage.create target)
    | Some target -> (
        match target with
        | `Nodo _ as n -> edit n
        | `Project _ -> print_endline "Unable to edit a project" )
end
