let contains e = List.fold_left (fun a i -> a || i = e) false

module Make (Storage : Nodo.Storage) (Format : Nodo.Format) (Config : Config.S) =
struct
  let config = Config.default

  type tree = Project of (Storage.project * tree list) | Nodo of Storage.nodo

  let rec build_tree l =
    List.map
      (fun item ->
        match item with
        | `Nodo _ as n ->
            Nodo n
        | `Project _ as p ->
            let sub_tree = Storage.list p |> build_tree in
            Project (p, sub_tree))
      l

  let filter_hidden =
    List.filter (fun d ->
        let s = Storage.name d in
        let contained = contains s config.hidden_dirs in
        not contained)

  let show_nodo nodo =
    Storage.read nodo |> Format.parse |> Format.render |> print_endline

  let rec map_but_last prefix a l = function
    | [] ->
        ""
    | [x] ->
        (prefix ^ "└─ ") ^ show_tree ~prefix:(prefix ^ l) x
    | x :: xs ->
        (prefix ^ "├─ ")
        ^ show_tree ~prefix:(prefix ^ a) x
        ^ map_but_last prefix a l xs

  and show_tree ~prefix t =
    match t with
    | Nodo n ->
        "N: " ^ Storage.name n ^ "\n"
    | Project (p, tl) ->
        ("P: " ^ Storage.name p ^ "\n") ^ map_but_last prefix "│  " "   " tl

  let exec target =
    let target =
      if target = "" then target else target ^ "." ^ List.hd Format.extensions
    in
    let open Astring in
    let target = String.cuts ~sep:"/" target in
    match Storage.classify target with
    | None ->
        print_endline "target not found"
    | Some t -> (
      match t with
      | `Nodo _ as n ->
          show_nodo n
      | `Project _ as p ->
          Storage.list p |> filter_hidden |> build_tree
          |> List.map (show_tree ~prefix:"")
          |> String.concat ~sep:"" |> print_string )
end
