module Result = Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

type config = {global: Config.t; create: bool; editor: string; target: string}
[@@deriving show]

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
  Term.(
    const build $ Config.cmdliner_term $ create $ editor
    $ Common.required_target_arg)

module Make (C : sig
  val t : config
end) =
struct
  module Storage = Storage.Make (struct
    let t = C.t.global.storage
  end)

  let edit nodo =
    let path = Storage.path nodo in
    let* content =
      let command = Printf.sprintf "%s %s" C.t.editor path in
      Logs.debug (fun f -> f "Executing edit command: %s" command) ;
      let _ = Sys.command command in
      Storage.read (`Nodo (Storage.location nodo))
    in
    match content with
    | Error e ->
        let* () = Lwt_io.printl e in
        Lwt.return_ok ()
    | Ok content -> (
        Logs.debug (fun f -> f "Read Content: %s" content) ;
        Logs.debug (fun f ->
            f "Finding format from extension: %s" C.t.global.format_ext) ;
        match Format.find_format_from_extension C.t.global.format_ext with
        | None ->
            Lwt.return_error "No format found"
        | Some (module F) ->
            let parsed = F.parse content in
            Logs.debug (fun f -> f "Parsed content: %a" Nodo.S.pp parsed) ;
            let rendered = F.render parsed in
            Logs.debug (fun f -> f "Rendered content: %s" rendered) ;
            rendered |> Storage.write nodo )

  let create_edit () =
    let target = Storage.with_extension ~ext:C.t.global.format_ext C.t.target in
    let* t = Storage.create target in
    match t with
    | Ok n -> (
        let* e = edit n in
        match e with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
    | Error e ->
        Lwt_io.printl e

  let exec () =
    let open Astring in
    let* t = Storage.classify C.t.target in
    match t with
    | None -> (
        let target =
          Storage.with_extension C.t.target ~ext:C.t.global.format_ext
        in
        let* t = Storage.classify target in
        match t with
        | None -> (
            if C.t.create then create_edit ()
            else
              let* () =
                Lwt_io.print
                  "Target not found, would you like to create it? [Y/n]: "
              in
              let* line = Lwt_io.read_line_opt Lwt_io.stdin in
              match line with
              | None ->
                  Lwt.return_unit
              | Some line -> (
                match String.Ascii.lowercase line with
                | "n" | "no" ->
                    Lwt.return_unit
                | _ ->
                    create_edit () ) )
        | Some (`Nodo _ as n) -> (
            let* e = edit n in
            match e with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
        | Some (`Project _) ->
            Lwt_io.printl "Unable to edit a project" )
    | Some (`Nodo _ as n) -> (
        let* e = edit n in
        match e with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
    | Some (`Project _) ->
        Lwt_io.printl "Unable to edit a project"
end
