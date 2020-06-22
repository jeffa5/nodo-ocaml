type location = string

type config = {dir: string; author: string; remote: string} [@@deriving make]

let cmdliner_term =
  let build dir author remote = {dir; author; remote} in
  let open Cmdliner in
  let docs = Manpage.s_common_options in
  let dir =
    let home =
      match Sys.getenv_opt "HOME" with Some h -> h | None -> "/tmp"
    in
    let doc = "The base dir to use for storage" in
    Arg.(value & opt string (home ^ "/.nodo") & info ["dir"] ~docs ~doc)
  in
  let author =
    let doc = "The author name to use for commits" in
    Arg.(value & opt string "Nodo" & info ["author"] ~docs ~doc)
  in
  let remote =
    let doc = "The remote url to sync with" in
    Arg.(required & opt (some string) None & info ["remote"] ~docs ~doc)
  in
  Term.(const build $ dir $ author $ remote)

open Stdlib.Result
open Lwt.Syntax

let exec_to_result ~cwd c =
  let* () = Lwt_io.printl ("+ " ^ c) in
  let open Lwt_process in
  let command = shell c in
  let* s = (open_process_none ~stdout:`Keep ~stderr:`Keep ~cwd command)#close in
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

type n = location

type nodo = [`Nodo of n]

type p = location

type project = [`Project of p]

type t = [nodo | project]

let build_path (config : config) path =
  if FilePath.is_relative path then config.dir ^ "/" ^ path else path

let read config (`Nodo p) =
  let path = build_path config p in
  let+ s = Lwt_io.lines_of_file path |> Lwt_stream.to_list in
  String.concat "\n" s |> ok

let write config (`Nodo p) content =
  let path = build_path config p in
  let* () =
    String.split_on_char '\n' content
    |> Lwt_stream.of_list |> Lwt_io.lines_to_file path
  in
  let open Lwt_result.Syntax in
  let* () = exec_to_result ~cwd:config.dir ("git add " ^ path) in
  exec_to_result ~cwd:config.dir
    ("git commit -m 'Updated " ^ path ^ "' --author='" ^ config.author ^ "'")

let list config (`Project p) =
  let path = build_path config p in
  let* l =
    Lwt_unix.files_of_directory path
    |> Lwt_stream.filter_map_s (fun f ->
           let path = path ^ "/" ^ f in
           let+ stat = Lwt_unix.stat path in
           match stat.st_kind with
           | Lwt_unix.S_REG ->
               Some (`Nodo (p ^ "/" ^ f))
           | S_DIR -> (
             match f with
             | ".git" | "." | ".." ->
                 None
             | _ ->
                 Some (`Project (p ^ "/" ^ f)) )
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

let classify config target =
  let path = build_path config target in
  let* exists = Lwt_unix.file_exists path in
  if exists then
    if Sys.is_directory path then Lwt.return_some (`Project target)
    else Lwt.return_some (`Nodo target)
  else Lwt.return_none

let create config l =
  let nodo = `Nodo l in
  write config nodo "" |> Lwt_result.map (fun _ -> nodo)

let remove config t =
  match t with
  | `Nodo n ->
      let path = build_path config n in
      let () = FileUtil.rm [path] in
      let open Lwt_result.Syntax in
      let* () = exec_to_result ~cwd:config.dir ("git add " ^ path) in
      exec_to_result ~cwd:config.dir
        ("git commit -m 'Removed " ^ path ^ "' --author='" ^ config.author ^ "'")
  | `Project p ->
      let path = build_path config p in
      let () = FileUtil.rm ~recurse:true [path] in
      let open Lwt_result.Syntax in
      let* () = exec_to_result ~cwd:config.dir ("git add " ^ path) in
      exec_to_result ~cwd:config.dir
        ("git commit -m 'Removed " ^ path ^ "' --author='" ^ config.author ^ "'")

let location = function `Nodo n -> n | `Project p -> p

let name t = match t with `Nodo n -> n | `Project p -> p |> Filename.basename

let with_extension l e = l ^ "." ^ e

let sync (config : config) =
  let open Lwt_result.Syntax in
  let* () = exec_to_result ~cwd:config.dir "git checkout master" in
  let* () = exec_to_result ~cwd:config.dir ("git pull " ^ config.remote) in
  exec_to_result ~cwd:config.dir ("git push " ^ config.remote)
