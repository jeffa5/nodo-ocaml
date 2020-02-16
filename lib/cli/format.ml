module Result = Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

module Make (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let format_nodo n =
    Lwt_result.bind (Storage.read n) (fun s ->
        Format.parse s |> Format.render |> Storage.write n)

  let rec format_project p =
    Lwt_result.bind (Storage.list p)
      (List.fold_left
         (fun i item ->
           Lwt_result.bind i (fun () ->
               match item with
               | `Project _ as p ->
                   format_project p
               | `Nodo _ as n ->
                   format_nodo n))
         (Lwt.return_ok ()))

  let exec target =
    let open Astring in
    let target = String.cuts ~sep:"/" target in
    let* t = Storage.classify target in
    match t with
    | None ->
        Lwt_io.printl "target doesn't exist"
    | Some (`Nodo _ as n) -> (
        let* r =
          Storage.with_extension n (List.hd Format.extensions) |> format_nodo
        in
        match r with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
    | Some (`Project _ as p) -> (
        let* r = format_project p in
        match r with Ok () -> Lwt.return_unit | Error e -> Lwt_io.printl e )
end
