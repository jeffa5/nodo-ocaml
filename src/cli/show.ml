let contains e = List.fold_left (fun a i -> a || i = e) false

module S
    (Storage : Nodo_core.Storage)
    (Format : Nodo_core.Format)
    (Config : Config.S) =
struct
  let config = Config.default

  type tree = Project of (string * tree list) | Nodo of string

  let rec build_tree l =
    List.map
      (fun item ->
        match item with
        | `Nodo n -> Nodo n
        | `Project p ->
            let sub_tree = Storage.list (`Project p) |> build_tree in
            Project (p, sub_tree))
      l

  let filter_hidden =
    List.filter (fun d ->
        let s = Storage.name d in
        let contained = contains s config.hidden_dirs in
        not contained)

  let show_nodo nodo =
    match Storage.read nodo |> Format.parse with
    | Error s -> print_endline @@ "Failed to read nodo" ^ s
    | Ok nodo -> (
        match Format.render nodo with
        | Error s -> print_endline @@ "Failed to render nodo" ^ s
        | Ok s -> print_endline s )

  let rec map_but_last prefix a l = function
    | [] -> ""
    | [ x ] -> (prefix ^ "└─ ") ^ show_tree (prefix ^ l) x
    | x :: xs ->
        (prefix ^ "├─ ")
        ^ show_tree (prefix ^ a) x
        ^ map_but_last prefix a l xs

  and show_tree prefix t =
    match t with
    | Nodo n -> "N: " ^ Storage.name (`Nodo n) ^ "\n"
    | Project (p, tl) ->
        ("P: " ^ Storage.name (`Project p) ^ "\n")
        ^ map_but_last prefix "│  " "   " tl

  let exec target =
    let target = target ^ "." ^ List.hd Format.extensions in
    match Storage.classify target with
    | None -> print_endline "target not found"
    | Some target -> (
        match target with
        | `Nodo _ as n -> show_nodo n
        | `Project _ as p ->
            Storage.list p |> filter_hidden |> build_tree
            |> List.map @@ show_tree ""
            |> String.concat "" |> print_string )
end
