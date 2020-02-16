module Result = Stdlib.Result

module Make (Storage : Nodo.Storage) (Format : Nodo.Format) = struct
  let format_nodo n =
    Result.bind (Storage.read n) (fun s ->
        Format.parse s |> Format.render |> Storage.write n)

  let rec format_project p =
    Result.bind (Storage.list p)
      (List.fold_left
         (fun i item ->
           Result.bind i (fun () ->
               match item with
               | `Project _ as p ->
                   format_project p
               | `Nodo _ as n ->
                   format_nodo n))
         (Ok ()))

  let exec target =
    let open Astring in
    let target = String.cuts ~sep:"/" target in
    match Storage.classify target with
    | None ->
        print_endline "target doesn't exist"
    | Some (`Nodo _ as n) ->
        Storage.with_extension n (List.hd Format.extensions)
        |> format_nodo
        |> Result.iter_error print_endline
    | Some (`Project _ as p) ->
        format_project p |> Result.iter_error print_endline
end
