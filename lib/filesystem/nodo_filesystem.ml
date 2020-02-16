open Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

module type Prefix_type = sig
  val prefix : string list
end

module Make (Prefix : Prefix_type) = struct
  include Nodo.Storage_types

  type n = location

  type nodo = [`Nodo of n]

  type p = location

  type project = [`Project of p]

  type t = [nodo | project]

  let build_path p =
    let path = String.concat "/" p in
    if FilePath.is_relative path then Prefix.prefix @ p else p

  let read (`Nodo p) =
    let path = String.concat "/" p in
    let+ s = Lwt_io.lines_of_file path |> Lwt_stream.to_list in
    String.concat "\n" s |> ok

  let write (`Nodo p) content =
    let path = String.concat "/" p in
    let+ () =
      String.split_on_char '\n' content
      |> Lwt_stream.of_list |> Lwt_io.lines_to_file path
    in
    Ok ()

  let list (`Project p) =
    let path = String.concat "/" p in
    let* l =
      Lwt_unix.files_of_directory path
      |> Lwt_stream.map_s (fun f ->
             let path = path ^ "/" ^ f in
             let+ stat = Lwt_unix.stat path in
             match stat.st_kind with
             | Lwt_unix.S_REG ->
                 `Nodo (p @ [f])
             | S_DIR ->
                 `Project (p @ [f])
             | _ ->
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
        FileUtil.rm [path] |> Lwt.return_ok
    | `Project p ->
        let path = String.concat "/" p in
        FileUtil.rm ~recurse:true [path] |> Lwt.return_ok

  let name t =
    let parts = (match t with `Nodo n -> n | `Project p -> p) |> List.rev in
    match parts with [] -> "" | r :: _ -> r

  let with_extension (`Nodo l) e =
    match List.rev l with
    | [] ->
        `Nodo l
    | x :: xs ->
        `Nodo (List.rev ((x ^ "." ^ e) :: xs))

  let sync () = Lwt.return_ok ()
end
