module S
    (Storage : Nodo_core.Storage)
    (Format : Nodo_core.Format)
    (Config : Config.S) =
struct
  let config = Config.default

  let contains e = List.fold_left (fun a i -> a || i = e) false

  let filter_hidden =
    List.filter (fun d ->
        let s = match d with `Nodo n -> n | `Project p -> p in
        let contained = contains s config.hidden_dirs in
        not contained)

  let show_project project =
    Storage.list project |> filter_hidden
    |> List.iter (fun f ->
           match f with
           | `Nodo n -> print_endline n
           | `Project p -> print_endline p)

  let show_nodo nodo = Storage.read nodo |> print_endline

  let exec target =
    match Storage.classify target with
    | None -> print_endline "target not found"
    | Some target -> (
        match target with
        | `Nodo _ as n -> show_nodo n
        | `Project _ as p -> show_project p )
end
