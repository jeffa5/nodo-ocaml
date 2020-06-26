open Astring

let ( let* ) = Lwt.bind

type config = {global: Config.t; generate: string option; all_pos: string list}
[@@deriving show]

let cmdliner_term =
  let build global generate all_pos = {global; generate; all_pos} in
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
  Term.(const build $ Config.cmdliner_term $ generate $ all_pos)

module Make (C : sig
  val t : config
end) =
struct
  module Storage = Storage.Make (struct
    let t = C.t.global.storage
  end)

  let print_targets () =
    let rec loop t =
      match t with
      | `Nodo _ as n ->
          Lwt_io.printl (Storage.location n)
      | `Project _ as p -> (
          let* () =
            let loc = Storage.location p in
            match loc with
            | "" ->
                Lwt.return_unit
            | _ ->
                Lwt_io.printl (loc ^ "/")
          in
          let* l = Storage.list p in
          match l with
          | Ok l ->
              Lwt_list.iter_s loop l
          | Error _ ->
              Lwt.return_unit )
    in
    let* r = Storage.classify "" in
    match r with None -> Lwt.return_unit | Some t -> loop t

  let commands = ["show"; "edit"; "remove"; "sync"]

  let print_commands () = Lwt_list.iter_s Lwt_io.printl commands

  let exec () =
    match C.t.generate with
    | None -> (
      match C.t.all_pos with
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
