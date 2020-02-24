module Result = Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

module Make (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let read_file f =
    let chan = open_in f and lines = ref [] in
    ( try
        while true do
          lines := input_line chan :: !lines
        done ;
        !lines
      with End_of_file -> close_in chan ; List.rev !lines )
    |> String.concat "\n"

  let edit nodo =
    let* content =
      Lwt_io.with_temp_file (fun (f, o) ->
          let* r = Storage.read nodo in
          let+ () =
            match r with
            | Ok c ->
                Format.parse c |> Format.render |> Lwt_io.write o
            | Error e ->
                Lwt_io.printl e
          in
          let _ = Sys.command @@ "vim " ^ f in
          read_file f)
    in
    Format.parse content |> Format.render |> Storage.write nodo

  let exec create target =
    let open Astring in
    match target with
    | "" ->
        Lwt_io.printl "TARGET cannot be empty"
    | _ -> (
        let target = String.cuts ~sep:"/" target in
        let* t = Storage.classify target in
        match t with
        | None -> (
            let target =
              Storage.with_extension target (List.hd Format.extensions)
            in
            let* t = Storage.classify target in
            match t with
            | None ->
                if create then
                  let* t = Storage.create target in
                  match t with
                  | Ok f -> (
                      let* e = edit f in
                      match e with
                      | Ok () ->
                          Lwt.return_unit
                      | Error e ->
                          Lwt_io.printl e )
                  | Error e ->
                      Lwt_io.printl e
                else Lwt_io.printl "target not found"
            | Some (`Nodo _ as n) -> (
                let* e = edit n in
                match e with
                | Ok () ->
                    Lwt.return_unit
                | Error e ->
                    Lwt_io.printl e )
            | Some (`Project _) ->
                Lwt_io.printl "Unable to edit a project" )
        | Some (`Nodo _ as n) -> (
            let* e = edit n in
            match e with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
        | Some (`Project _) ->
            Lwt_io.printl "Unable to edit a project" )
end
