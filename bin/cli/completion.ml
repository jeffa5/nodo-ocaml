open Astring

let ( let* ) = Lwt.bind

type config = {generate: string option; all_pos: string list}

let cmdliner_term =
  let build generate all_pos = {generate; all_pos} in
  let open Cmdliner in
  let generate =
    let doc =
      "The shell to generate completion scripts for. The value of $(docv) must \
       be "
      ^ Arg.doc_alts ~quoted:true ["zsh"]
      ^ "."
    in
    Arg.(value & opt (some string) None (info ~docv:"SHELL" ~doc ["generate"]))
  in
  let all_pos = Arg.(value & pos_all string [] (info [])) in
  Term.(const build $ generate $ all_pos)

let print_targets config () =
  let rec loop t =
    match t with
    | `Nodo _ as n ->
        Lwt_io.printl (Storage.location n)
    | `Project _ as p -> (
        let* () =
          let loc = Storage.location p in
          match loc with "" -> Lwt.return_unit | _ -> Lwt_io.printl (loc ^ "/")
        in
        let* l = Storage.list config p in
        match l with
        | Ok l ->
            Lwt_list.iter_s loop l
        | Error _ ->
            Lwt.return_unit )
  in
  let* r = Storage.classify config "" in
  match r with None -> Lwt.return_unit | Some t -> loop t

let commands = ["show"; "edit"; "remove"; "sync"]

let print_commands () = Lwt_list.iter_s Lwt_io.printl commands

let exec gconf conf =
  match conf.generate with
  | None -> (
    match conf.all_pos with
    | [] ->
        assert false
    | [_] ->
        print_commands ()
    | [_; x] ->
        if List.exists (fun c -> x = c) commands then print_targets gconf ()
        else print_commands ()
    | _ ->
        print_targets gconf () )
  | Some shell -> (
    match shell with
    | "zsh" ->
        Lwt_io.printl Zshcomp.completions
    | _ ->
        Lwt_io.printl @@ "No completion available for " ^ shell )
