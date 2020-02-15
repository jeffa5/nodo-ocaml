module Make (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let read_file f =
    let chan = open_in f and lines = ref [] in
    let content =
      ( try
          while true do
            lines := input_line chan :: !lines
          done ;
          !lines
        with End_of_file -> close_in chan ; List.rev !lines )
      |> String.concat "\n"
    in
    close_in chan ; content

  let edit nodo =
    let f, o =
      Filename.open_temp_file "nodo-" ("." ^ List.hd Format.extensions)
    in
    Storage.read nodo |> Format.parse |> Format.render |> output_string o ;
    close_out_noerr o ;
    let _ = Sys.command @@ "vim " ^ f in
    let content = read_file f in
    Format.parse content |> Format.render |> Storage.write nodo

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
