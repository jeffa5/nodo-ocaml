open Option

module type Prefix_type = sig
  val prefix : string
end

module Make (Prefix : Prefix_type) = struct
  include Nodo_core.Storage_types

  let build_path p = Prefix.prefix ^ "/" ^ p

  let read (`Nodo nodo) =
    let chan = open_in nodo and lines = ref [] in
    ( try
        while true do
          lines := input_line chan :: !lines
        done;
        !lines
      with End_of_file ->
        close_in chan;
        List.rev !lines )
    |> String.concat "\n"

  let write content (`Nodo nodo) =
    let chan = open_out nodo in
    output_string chan content

  let list (`Project project) =
    Sys.readdir project |> Array.to_list
    |> List.map (fun f ->
           if Sys.is_directory (project ^ "/" ^ f) then `Project f else `Nodo f)

  let classify target =
    let path = build_path target in
    if Sys.file_exists path then
      if Sys.is_directory path then some (`Project path) else some (`Nodo path)
    else None

  let create f =
    let path = build_path f in
    let nodo = `Nodo path in
    write "" nodo;
    nodo

  let remove = function
    | `Nodo n -> FileUtil.rm [ n ]
    | `Project p -> FileUtil.rm ~recurse:true [ p ]
end
