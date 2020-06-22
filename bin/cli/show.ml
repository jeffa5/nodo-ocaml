module Result = Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

let contains e = List.fold_left (fun a i -> a || i = e) false

type tree = Project of (Storage.project * tree list) | Nodo of Storage.nodo

type config = {global: Config.t; target: string}

let cmdliner_term =
  let build global target = {global; target} in
  let open Cmdliner in
  let target =
    let doc = "The target to show" in
    Arg.(required & pos 0 (some string) None & info [] ~doc)
  in
  Term.(const build $ Config.cmdliner_term $ target)

let rec build_tree (config : Config.t) l =
  List.sort
    (fun a b ->
      match (a, b) with
      | (`Nodo _ as a), (`Nodo _ as b) ->
          String.compare (Storage.name a) (Storage.name b)
      | `Nodo _, `Project _ ->
          -1
      | `Project _, `Nodo _ ->
          1
      | (`Project _ as a), (`Project _ as b) ->
          String.compare (Storage.name a) (Storage.name b))
    l
  |> Lwt_list.filter_map_s (fun item ->
         match item with
         | `Nodo _ as n ->
             Lwt.return_some (Nodo n)
         | `Project _ as p ->
             let+ r =
               let l = Storage.list config.storage p in
               Lwt_result.bind l (fun l ->
                   let* sub_tree = build_tree config l in
                   Lwt.return_ok (Project (p, sub_tree)))
             in
             Result.to_option r)

let filter_hidden (config : Config.t) =
  List.filter (fun d ->
      let s = Storage.name d in
      let contained = contains s config.hidden_dirs in
      not contained)

let progress (config : Config.t) (`Nodo _ as n) =
  let handle_item = function
    | Nodo.S.Task (c, _) ->
        Some c
    | Bullet _ ->
        None
  in
  let handle_t (t : Nodo.S.t) =
    List.map
      (function
        | Nodo.S.List l ->
            let l =
              match l with
              | Ordered l ->
                  List.filter_map (fun (_, i, _) -> handle_item i) l
              | Unordered l ->
                  List.filter_map (fun (i, _) -> handle_item i) l
            in
            ( List.fold_left (fun a b -> if b then a + 1 else a) 0 l
            , List.length l )
        | _ ->
            (0, 0))
      t.blocks
    |> List.fold_left (fun (a, b) (c, d) -> (a + c, b + d)) (0, 0)
  in
  let* r = Storage.read config.storage n in
  match r with
  | Ok c ->
      let c, t = Format.parse config c |> handle_t in
      (if t > 0 then Printf.sprintf "(%i/%i)" c t else "") |> Lwt.return
  | Error e ->
      Lwt.return e

let show_nodo (config : Config.t) nodo =
  let* r = Storage.read config.storage nodo in
  match r with
  | Ok c ->
      Format.parse config c |> Format.render config |> Lwt_io.printl
  | Error e ->
      Lwt_io.printl e

let rec map_but_last config prefix a l = function
  | [] ->
      Lwt.return ""
  | [x] ->
      let+ t = show_tree config ~prefix:(prefix ^ l) x in
      (prefix ^ "└─ ") ^ t
  | x :: xs ->
      let* t = show_tree config ~prefix:(prefix ^ a) x in
      let+ m = map_but_last config prefix a l xs in
      (prefix ^ "├─ ") ^ t ^ m

and show_tree config ~prefix t =
  match t with
  | Nodo n ->
      let+ p = progress config n in
      "N: " ^ Storage.name n ^ " " ^ p ^ "\n"
  | Project (p, tl) ->
      let+ m = map_but_last config prefix "│  " "   " tl in
      ("P: " ^ Storage.name p ^ "\n") ^ m

let exec (conf : config) =
  let open Astring in
  let* r = Storage.classify conf.global.storage conf.target in
  match r with
  | None -> (
    match conf.target with
    | "" ->
        Lwt_io.printl "target not found"
    | _ -> (
        let target =
          Storage.with_extension conf.target (List.hd Format.formats)
        in
        let* r = Storage.classify conf.global.storage target in
        match r with
        | None ->
            Lwt_io.printl "target not found"
        | Some t -> (
          match t with
          | `Nodo _ as n ->
              show_nodo conf.global n
          | `Project _ as p -> (
              let* l = Storage.list conf.global.storage p in
              match l with
              | Ok l ->
                  let* t =
                    filter_hidden conf.global l |> build_tree conf.global
                  in
                  let* ts =
                    Lwt_list.map_s (show_tree conf.global ~prefix:"") t
                  in
                  String.concat ~sep:"" ts |> Lwt_io.print
              | Error e ->
                  Lwt_io.printl e ) ) ) )
  | Some t -> (
    match t with
    | `Nodo _ as n ->
        show_nodo conf.global n
    | `Project _ as p -> (
        let* l = Storage.list conf.global.storage p in
        match l with
        | Ok l ->
            let* t = filter_hidden conf.global l |> build_tree conf.global in
            let* ts = Lwt_list.map_s (show_tree conf.global ~prefix:"") t in
            String.concat ~sep:"" ts |> Lwt_io.print
        | Error e ->
            Lwt_io.printl e ) )
