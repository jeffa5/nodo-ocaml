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

  let edit nodo editor =
    let name = Storage.name nodo in
    let* content =
      Lwt_io.with_temp_file ~prefix:"nodo-" ~suffix:("-" ^ name) (fun (f, o) ->
          let* r = Storage.read nodo in
          let* () =
            match r with
            | Ok c ->
                Format.parse c |> Format.render |> Lwt_io.write o
            | Error e ->
                Lwt_io.printl e
          in
          let+ () = Lwt_io.flush o in
          let _ = Sys.command @@ editor ^ " " ^ f in
          read_file f)
    in
    Format.parse content |> Format.render |> Storage.write nodo

  let create_edit target editor =
    let* t = Storage.create target in
    match t with
    | Ok f -> (
        let* e = edit f editor in
        match e with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
    | Error e ->
        Lwt_io.printl e

  let exec create target editor =
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
            | None -> (
                if create then create_edit target editor
                else
                  let* () =
                    Lwt_io.print
                      "Target not found, would you like to create it? [y/N]: "
                  in
                  let* line = Lwt_io.read_line_opt Lwt_io.stdin in
                  match line with
                  | None ->
                      Lwt.return_unit
                  | Some line -> (
                    match String.Ascii.lowercase line with
                    | "y" | "yes" ->
                        create_edit target editor
                    | _ ->
                        Lwt.return_unit ) )
            | Some (`Nodo _ as n) -> (
                let* e = edit n editor in
                match e with
                | Ok () ->
                    Lwt.return_unit
                | Error e ->
                    Lwt_io.printl e )
            | Some (`Project _) ->
                Lwt_io.printl "Unable to edit a project" )
        | Some (`Nodo _ as n) -> (
            let* e = edit n editor in
            match e with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
        | Some (`Project _) ->
            Lwt_io.printl "Unable to edit a project" )
end
