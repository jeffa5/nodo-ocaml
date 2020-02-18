module Result = Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

let contains e = List.fold_left (fun a i -> a || i = e) false

module Make (Storage : Nodo.Storage) (Format : Nodo.Format) (Config : Config.S) =
struct
  let config = Config.default

  type tree = Project of (Storage.project * tree list) | Nodo of Storage.nodo

  let rec build_tree l =
    Lwt_list.filter_map_s
      (fun item ->
        match item with
        | `Nodo _ as n ->
            Lwt.return_some (Nodo n)
        | `Project _ as p ->
            let+ r =
              let l = Storage.list p in
              Lwt_result.bind l (fun l ->
                  let* sub_tree = build_tree l in
                  Lwt.return_ok (Project (p, sub_tree)))
            in
            Result.to_option r)
      l

  let filter_hidden =
    List.filter (fun d ->
        let s = Storage.name d in
        let contained = contains s config.hidden_dirs in
        not contained)

  let show_nodo nodo =
    let* r = Storage.read nodo in
    match r with
    | Ok c ->
        Format.parse c |> Format.render |> Lwt_io.printl
    | Error e ->
        Lwt_io.printl e

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
    let open Astring in
    let target = String.cuts ~empty:false ~sep:"/" target in
    let* r = Storage.classify target in
    match r with
    | None -> (
      match target with
      | [] ->
          Lwt_io.printl "target not found"
      | _ -> (
          let target =
            Storage.with_extension target (List.hd Format.extensions)
          in
          let* r = Storage.classify target in
          match r with
          | None ->
              Lwt_io.printl "target not found"
          | Some t -> (
            match t with
            | `Nodo _ as n ->
                show_nodo n
            | `Project _ as p -> (
                let* l = Storage.list p in
                match l with
                | Ok l ->
                    let* t = filter_hidden l |> build_tree in
                    List.map (show_tree ~prefix:"") t
                    |> String.concat ~sep:"" |> Lwt_io.print
                | Error e ->
                    Lwt_io.printl e ) ) ) )
    | Some t -> (
      match t with
      | `Nodo _ as n ->
          show_nodo n
      | `Project _ as p -> (
          let* l = Storage.list p in
          match l with
          | Ok l ->
              let* t = filter_hidden l |> build_tree in
              List.map (show_tree ~prefix:"") t
              |> String.concat ~sep:"" |> Lwt_io.print
          | Error e ->
              Lwt_io.printl e ) )
end
