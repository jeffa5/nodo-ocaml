open Option

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
    let chan = open_in path and lines = ref [] in
    ( try
        while true do
          lines := input_line chan :: !lines
        done ;
        !lines
      with End_of_file -> close_in chan ; List.rev !lines )
    |> String.concat "\n"

  let write (`Nodo p) content =
    let path = String.concat "/" p in
    let chan = open_out path in
    output_string chan content

  let list (`Project p) =
    let path = String.concat "/" p in
    Sys.readdir path |> Array.to_list
    |> List.map (fun f ->
           let path = path ^ "/" ^ f in
           if Sys.is_directory path then `Project (p @ [f]) else `Nodo (p @ [f]))

  let classify target =
    let p = build_path target in
    let path = String.concat "/" p in
    if Sys.file_exists path then
      if Sys.is_directory path then some (`Project p) else some (`Nodo p)
    else None

  let create l =
    let path = build_path l in
    let nodo = `Nodo path in
    write nodo "" ; nodo

  let remove = function
    | `Nodo n ->
        let path = String.concat "/" n in
        FileUtil.rm [path]
    | `Project p ->
        let path = String.concat "/" p in
        FileUtil.rm ~recurse:true [path]

  let name t =
    let parts = (match t with `Nodo n -> n | `Project p -> p) |> List.rev in
    match parts with [] -> "" | r :: _ -> r

  let with_extension (`Nodo l) e =
    match List.rev l with
    | [] ->
        `Nodo l
    | x :: xs ->
        `Nodo (List.rev ((x ^ "." ^ e) :: xs))
end
