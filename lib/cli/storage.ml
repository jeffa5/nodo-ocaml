type config = {dir: string} [@@deriving make, show]

let cmdliner_term =
  let build dir = {dir} in
  let open Cmdliner in
  let docs = Manpage.s_common_options in
  let dir =
    let home =
      match Sys.getenv_opt "HOME" with Some h -> h | None -> "/tmp"
    in
    let doc = "The base dir to use for storage" in
    Arg.(
      value
      & opt string (Filename.concat home ".nodo")
      & info ["dir"] ~docs ~doc)
  in
  Term.(const build $ dir)

open Stdlib.Result
open Lwt.Syntax

module type S = sig
  type nodo = [`Nodo of string]

  type project = [`Project of string]

  type t = [nodo | project]

  val list : project -> (t list, string) Lwt_result.t

  val classify : string -> t option Lwt.t

  val create : string -> (nodo, string) Lwt_result.t

  val remove : [< nodo | project] -> (unit, string) Lwt_result.t

  val location : [< nodo | project] -> string

  val name : [< nodo | project] -> string

  val path : [< nodo | project] -> string

  val with_extension : ext:string -> string -> string

  val read : nodo -> (string, string) Lwt_result.t

  val write : nodo -> string -> (unit, string) Lwt_result.t

  val sync : unit -> (unit, string) Lwt_result.t
end

module Make (C : sig
  val t : config
end) : S = struct
  let exec_to_result ~cwd c =
    let* () = Lwt_io.printl ("+ " ^ c) in
    let open Lwt_process in
    let command = shell c in
    let* s =
      (open_process_none ~stdout:`Keep ~stderr:`Keep ~cwd command)#close
    in
    match s with
    | Unix.WEXITED code -> (
      match code with
      | 0 ->
          Lwt.return_ok ()
      | _ ->
          Lwt.return_error
            ( "Failed executing command: " ^ c ^ ", received error code: "
            ^ string_of_int code ) )
    | _ ->
        Lwt.return_error ("Failed executing command: " ^ c)

  type nodo = [`Nodo of string]

  type project = [`Project of string]

  type t = [nodo | project]

  let build_path path = Filename.concat C.t.dir path

  let read (`Nodo p) =
    let path = build_path p in
    let* exists = Lwt_unix.file_exists path in
    if not exists then Lwt.return_ok ""
    else
      let+ s = Lwt_io.lines_of_file path |> Lwt_stream.to_list in
      String.concat "\n" s |> ok

  let write (`Nodo p as n) content =
    let path = build_path p in
    let* prev = read n in
    match prev with
    | Error e ->
        Lwt.return_error (Printf.sprintf "failed to read: %s" e)
    | Ok s ->
        if s = content then Lwt.return_ok ()
        else
          let* () =
            String.split_on_char '\n' content
            |> Lwt_stream.of_list |> Lwt_io.lines_to_file path
          in
          let open Lwt_result.Syntax in
          let* () = exec_to_result ~cwd:C.t.dir ("git add " ^ path) in
          exec_to_result ~cwd:C.t.dir ("git commit -m 'Updated " ^ p ^ "'")

  let list (`Project p) =
    let path = build_path p in
    let* l =
      Lwt_unix.files_of_directory path
      |> Lwt_stream.filter_map_s (fun f ->
             let path = Filename.concat path f in
             let+ stat = Lwt_unix.stat path in
             match stat.st_kind with
             | Lwt_unix.S_REG ->
                 Some (`Nodo (Filename.concat p f))
             | S_DIR -> (
               match f with
               | ".git" | "." | ".." ->
                   None
               | _ ->
                   Some (`Project (Filename.concat p f)) )
             | S_CHR ->
                 print_endline "s_chr" ;
                 assert false
             | S_BLK ->
                 print_endline "s_blk" ;
                 assert false
             | S_FIFO ->
                 print_endline "fifo" ;
                 assert false
             | S_LNK ->
                 print_endline "link" ;
                 assert false
             | S_SOCK ->
                 print_endline "sock" ;
                 assert false)
      |> Lwt_stream.to_list
    in
    Lwt.return_ok l

  let classify target =
    let path = build_path target in
    let* exists = Lwt_unix.file_exists path in
    if exists then
      if Sys.is_directory path then Lwt.return_some (`Project target)
      else Lwt.return_some (`Nodo target)
    else Lwt.return_none

  let create l =
    let nodo = `Nodo l in
    write nodo "" |> Lwt_result.map (fun _ -> nodo)

  let remove t =
    match t with
    | `Nodo n ->
        let path = build_path n in
        let () = FileUtil.rm [path] in
        let open Lwt_result.Syntax in
        let* () = exec_to_result ~cwd:C.t.dir ("git add " ^ path) in
        exec_to_result ~cwd:C.t.dir ("git commit -m 'Removed " ^ path ^ "'")
    | `Project p ->
        let path = build_path p in
        let () = FileUtil.rm ~recurse:true [path] in
        let open Lwt_result.Syntax in
        let* () = exec_to_result ~cwd:C.t.dir ("git add " ^ path) in
        exec_to_result ~cwd:C.t.dir ("git commit -m 'Removed " ^ path ^ "'")

  let location = function `Nodo n -> n | `Project p -> p

  let name t =
    (match t with `Nodo n -> n | `Project p -> p) |> Filename.basename

  let path t = (match t with `Nodo n -> n | `Project p -> p) |> build_path

  let with_extension ~ext l = l ^ "." ^ ext

  let sync () =
    let open Lwt_result.Syntax in
    let* () = exec_to_result ~cwd:C.t.dir "git checkout master" in
    let* () = exec_to_result ~cwd:C.t.dir "git pull" in
    exec_to_result ~cwd:C.t.dir "git push"
end
