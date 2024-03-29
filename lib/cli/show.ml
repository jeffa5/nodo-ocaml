module Result = Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

type config = {global: Config.t; target: string; depth: int} [@@deriving show]

let cmdliner_term =
  let build global target depth = {global; target; depth} in
  let open Cmdliner in
  let depth =
    let doc = "Depth of the tree to show" in
    Arg.(value & opt int 1 (info ~doc ["depth"; "d"]))
  in
  Term.(const build $ Config.cmdliner_term $ Common.target_arg $ depth)

module Make (C : sig
  val t : config
end) =
struct
  module Storage = Storage.Make (struct
    let t = C.t.global.storage
  end)

  let contains e = List.fold_left (fun a i -> a || i = e) false

  type tree = Project of (Storage.project * tree list) | Nodo of Storage.nodo

  let build_tree depth l =
    let rec loop depth l =
      if depth = 0 then Lwt.return_nil
      else
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
                     let l = Storage.list p in
                     Lwt_result.bind l (fun l ->
                         let* sub_tree = loop (depth - 1) l in
                         Lwt.return_ok (Project (p, sub_tree)))
                   in
                   Result.to_option r)
    in
    loop (if depth = 0 then -1 else depth) l

  let filter_hidden =
    List.filter (fun d ->
        let s = Storage.name d in
        let contained = contains s C.t.global.hidden_dirs in
        not contained)

  let progress (`Nodo _ as n) =
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
    let* r = Storage.read n in
    match r with
    | Ok c -> (
      match Format.find_format_from_extension C.t.global.format_ext with
      | None ->
          Lwt.return ""
      | Some (module F) ->
          let c, t = F.parse c |> handle_t in
          (if t > 0 then Printf.sprintf "(%i/%i)" c t else "") |> Lwt.return )
    | Error e ->
        Lwt.return e

  let show_nodo nodo =
    let* r = Storage.read nodo in
    match r with
    | Ok c -> (
      match Format.find_format_from_extension C.t.global.format_ext with
      | None ->
          Lwt.return_unit
      | Some (module F) ->
          F.parse c |> F.render |> Lwt_io.printl )
    | Error e ->
        Lwt_io.printl e

  let rec map_but_last prefix a l = function
    | [] ->
        Lwt.return ""
    | [x] ->
        let+ t = show_tree ~prefix:(prefix ^ l) x in
        (prefix ^ "└─ ") ^ t
    | x :: xs ->
        let* t = show_tree ~prefix:(prefix ^ a) x in
        let+ m = map_but_last prefix a l xs in
        (prefix ^ "├─ ") ^ t ^ m

  and show_tree ~prefix t =
    match t with
    | Nodo n ->
        let+ p = progress n in
        "N: " ^ Storage.name n ^ " " ^ p ^ "\n"
    | Project (p, tl) ->
        let+ m = map_but_last prefix "│  " "   " tl in
        ("P: " ^ Storage.name p ^ "\n") ^ m

  let exec () =
    let open Astring in
    let* r = Storage.classify C.t.target in
    match r with
    | None -> (
      match C.t.target with
      | "" ->
          Lwt_io.printl "Target not found"
      | _ -> (
          let target =
            Storage.with_extension ~ext:C.t.global.format_ext C.t.target
          in
          let* r = Storage.classify target in
          match r with
          | None ->
              Lwt_io.printl "Target not found"
          | Some t -> (
            match t with
            | `Nodo _ as n ->
                show_nodo n
            | `Project _ as p -> (
                let* l = Storage.list p in
                match l with
                | Ok l ->
                    let* t = filter_hidden l |> build_tree C.t.depth in
                    let* ts = Lwt_list.map_s (show_tree ~prefix:"") t in
                    String.concat ~sep:"" ts |> Lwt_io.print
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
              let* t = filter_hidden l |> build_tree C.t.depth in
              let* ts = Lwt_list.map_s (show_tree ~prefix:"") t in
              String.concat ~sep:"" ts |> Lwt_io.print
          | Error e ->
              Lwt_io.printl e ) )
end
