module Result = Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

type config = {global: Config.t; create: bool; editor: string; target: string}

let cmdliner_term =
  let build global create editor target = {global; create; editor; target} in
  let open Cmdliner in
  let create =
    let doc = "Create the nodo if it does not exist." in
    Arg.(value & flag (info ~doc ["c"]))
  in
  let editor =
    let doc =
      "The editor to launch when editing. The command will be run as "
      ^ Arg.doc_quote "$(docv) file"
    in
    let env = Arg.env_var "NODO_EDITOR" in
    Arg.(value & opt string "vim" & info ~env ~docv:"EDITOR" ~doc ["e"])
  in
  let target =
    let doc = "The target to show" in
    Arg.(required & pos 0 (some string) None & info [] ~doc)
  in
  Term.(const build $ Config.cmdliner_term $ create $ editor $ target)

let read_file f =
  let chan = open_in f and lines = ref [] in
  ( try
      while true do
        lines := input_line chan :: !lines
      done ;
      !lines
    with End_of_file -> close_in chan ; List.rev !lines )
  |> String.concat "\n"

let edit conf nodo =
  let name = Storage.name nodo in
  let* content =
    Lwt_io.with_temp_file ~prefix:"nodo-" ~suffix:("-" ^ name) (fun (f, o) ->
        let* r = Storage.read conf.global.storage nodo in
        let* () =
          match r with
          | Ok c ->
              Format.parse conf.global c |> Format.render conf.global
              |> Lwt_io.write o
          | Error e ->
              Lwt_io.printl e
        in
        let+ () = Lwt_io.flush o in
        let _ = Sys.command @@ conf.editor ^ " " ^ f in
        read_file f)
  in
  Format.parse content |> Format.render conf.global
  |> Storage.write conf.global.storage nodo

let create_edit (global_conf : Config.t) conf =
  let* t = Storage.create global_conf.storage conf.target in
  match t with
  | Ok n -> (
      let* e = edit conf n in
      match e with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
  | Error e ->
      Lwt_io.printl e

let exec (conf : config) =
  let open Astring in
  match conf.target with
  | "" ->
      Lwt_io.printl "TARGET cannot be empty"
  | _ -> (
      let* t = Storage.classify conf.global.storage conf.target in
      match t with
      | None -> (
          let target =
            Storage.with_extension conf.target (List.hd Format.formats)
          in
          let* t = Storage.classify conf.global.storage target in
          match t with
          | None -> (
              if conf.create then create_edit conf.global conf
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
                      create_edit conf.global conf
                  | _ ->
                      Lwt.return_unit ) )
          | Some (`Nodo _ as n) -> (
              let* e = edit conf n in
              match e with
              | Ok () ->
                  Lwt.return_unit
              | Error e ->
                  Lwt_io.printl e )
          | Some (`Project _) ->
              Lwt_io.printl "Unable to edit a project" )
      | Some (`Nodo _ as n) -> (
          let* e = edit conf n in
          match e with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
      | Some (`Project _) ->
          Lwt_io.printl "Unable to edit a project" )
