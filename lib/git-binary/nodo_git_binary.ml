open Stdlib.Result
open Lwt.Syntax

let exec_to_result ~cwd c =
  let* () = Lwt_io.printl ("+ " ^ c) in
  let open Lwt_process in
  let cwd = String.concat "/" cwd in
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

module type Config = sig
  val dir : string list ref

  val remote : string ref

  val author : string ref
end

module Make (C : Config) = struct
  include Nodo.Storage_types

  type n = location

  type nodo = [`Nodo of n]

  type p = location

  type project = [`Project of p]

  type t = [nodo | project]

  let build_path p =
    let path = String.concat "/" p in
    if FilePath.is_relative path then !C.dir @ p else p

  let read (`Nodo p) =
    let path = String.concat "/" p in
    let+ s = Lwt_io.lines_of_file path |> Lwt_stream.to_list in
    String.concat "\n" s |> ok

  let write (`Nodo p) content =
    let path = String.concat "/" p in
    let* () =
      String.split_on_char '\n' content
      |> Lwt_stream.of_list |> Lwt_io.lines_to_file path
    in
    let open Lwt_result.Syntax in
    let* () = exec_to_result ~cwd:!C.dir ("git add " ^ path) in
    exec_to_result ~cwd:!C.dir
      ("git commit -m 'Updated " ^ path ^ "' --author='" ^ !C.author ^ "'")

  let list (`Project p) =
    let path = String.concat "/" p in
    let* l =
      Lwt_unix.files_of_directory path
      |> Lwt_stream.filter_map_s (fun f ->
             let path = path ^ "/" ^ f in
             let+ stat = Lwt_unix.stat path in
             match stat.st_kind with
             | Lwt_unix.S_REG ->
                 Some (`Nodo (p @ [f]))
             | S_DIR -> (
               match f with
               | ".git" | "." | ".." ->
                   None
               | _ ->
                   Some (`Project (p @ [f])) )
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
    let p = build_path target in
    let path = String.concat "/" p in
    let* exists = Lwt_unix.file_exists path in
    if exists then
      if Sys.is_directory path then Lwt.return_some (`Project p)
      else Lwt.return_some (`Nodo p)
    else Lwt.return_none

  let create l =
    let path = build_path l in
    let nodo = `Nodo path in
    write nodo "" |> Lwt_result.map (fun _ -> nodo)

  let remove = function
    | `Nodo n ->
        let path = String.concat "/" n in
        let () = FileUtil.rm [path] in
        let open Lwt_result.Syntax in
        let* () = exec_to_result ~cwd:!C.dir ("git add " ^ path) in
        exec_to_result ~cwd:!C.dir
          ("git commit -m 'Removed " ^ path ^ "' --author='" ^ !C.author ^ "'")
    | `Project p ->
        let path = String.concat "/" p in
        let () = FileUtil.rm ~recurse:true [path] in
        let open Lwt_result.Syntax in
        let* () = exec_to_result ~cwd:!C.dir ("git add " ^ path) in
        exec_to_result ~cwd:!C.dir
          ("git commit -m 'Removed " ^ path ^ "' --author='" ^ !C.author ^ "'")

  let location = function `Nodo n -> n | `Project p -> p

  let name t =
    let parts = (match t with `Nodo n -> n | `Project p -> p) |> List.rev in
    match parts with [] -> "" | r :: _ -> r

  let with_extension l e =
    match List.rev l with [] -> l | x :: xs -> List.rev ((x ^ "." ^ e) :: xs)

  let sync () =
    let open Lwt_result.Syntax in
    let* () = exec_to_result ~cwd:!C.dir "git checkout master" in
    let* () = exec_to_result ~cwd:!C.dir ("git pull " ^ !C.remote) in
    exec_to_result ~cwd:!C.dir ("git push " ^ !C.remote)
end
