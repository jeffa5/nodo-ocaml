open Astring

let ( let* ) = Lwt.bind

module Make (Storage : Nodo.Storage) (Format : Nodo.Format) (Config : Config.S) =
struct
  let print_targets () =
    let rec loop t =
      match t with
      | `Nodo _ as n ->
          Lwt_io.printl (String.concat ~sep:"/" (Storage.location n))
      | `Project _ as p -> (
          let* () =
            let loc = Storage.location p in
            match loc with
            | [] ->
                Lwt.return_unit
            | _ ->
                Lwt_io.printl (String.concat ~sep:"/" loc ^ "/")
          in
          let* l = Storage.list p in
          match l with
          | Ok l ->
              Lwt_list.iter_s loop l
          | Error _ ->
              Lwt.return_unit )
    in
    let* r = Storage.classify [] in
    match r with None -> Lwt.return_unit | Some t -> loop t

  let commands = ["show"; "edit"; "remove"; "sync"]

  let print_commands () = Lwt_list.iter_s Lwt_io.printl commands

  let exec gen opts =
    match gen with
    | None -> (
      match opts with
      | [] ->
          assert false
      | [_] ->
          print_commands ()
      | [_; x] ->
          if List.exists (fun c -> x = c) commands then print_targets ()
          else print_commands ()
      | _ ->
          print_targets () )
    | Some shell -> (
      match shell with
      | "zsh" ->
          Lwt_io.printl Zshcomp.completions
      | _ ->
          Lwt_io.printl @@ "No completion available for " ^ shell )
end
