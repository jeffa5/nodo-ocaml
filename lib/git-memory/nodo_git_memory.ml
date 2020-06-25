open Option
open Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

module type Config = sig
  val dir : string ref

  val remote : string ref

  val remote_headers : Cohttp.Header.t option ref

  val author : string ref
end

module Make (C : Config) = struct
  module Store = Irmin_mirage_git.KV (Irmin_git.Mem) (Irmin.Contents.String)
  module Sync = Irmin.Sync (Store)

  let remote () =
    match !C.remote_headers with
    | None ->
        Store.remote !C.remote
    | Some h ->
        Store.remote ~headers:h !C.remote

  let irmin_config () = Irmin_git.config ~bare:true !C.dir

  let irmin_info msg () =
    let date = Int64.of_float (Unix.gettimeofday ()) in
    Irmin.Info.v ~date ~author:!C.author msg

  include Nodo.Storage_types

  type n = location

  type nodo = [`Nodo of n]

  type p = location

  type project = [`Project of p]

  type t = [nodo | project]

  let store () =
    let* repo = Store.Repo.v (irmin_config ()) in
    Store.master repo

  let read (`Nodo p) =
    let* master = store () in
    let* contents = Store.get master p in
    Lwt.return_ok contents

  let write (`Nodo p) content =
    let* master = store () in
    let res = Store.set master ~info:(irmin_info "test update") p content in
    Lwt_result.map_err
      (function
        | `Too_many_retries _ ->
            "too many retries"
        | `Test_was _ ->
            "test was"
        | `Conflict s ->
            s)
      res

  let list (`Project p) =
    let* master = store () in
    let+ l = Store.list master p in
    l
    |> List.map (function
         | s, `Contents ->
             `Nodo (p @ [s])
         | s, `Node ->
             `Project (p @ [s]))
    |> ok

  let classify p =
    let* master = store () in
    match p with
    | [] ->
        Lwt.return_some (`Project [])
    | _ -> (
        let* kind = Store.kind master p in
        match kind with
        | None ->
            Lwt.return_none
        | Some `Contents ->
            Lwt.return_some (`Nodo p)
        | Some `Node ->
            Lwt.return_some (`Project p) )

  let create l =
    let nodo = `Nodo l in
    write nodo "" |> Lwt_result.map (fun () -> nodo)

  let remove t =
    let* master = store () in
    (match t with `Nodo n -> n | `Project p -> p)
    |> Store.remove master ~info:(irmin_info "removing item")
    |> Lwt_result.map_err (function
         | `Too_many_retries _ ->
             "too many retries"
         | `Test_was _ ->
             "test was"
         | `Conflict s ->
             s)

  let location = function `Nodo n -> n | `Project p -> p

  let name t =
    let parts = (match t with `Nodo n -> n | `Project p -> p) |> List.rev in
    match parts with [] -> "" | r :: _ -> r

  let with_extension l e =
    match List.rev l with [] -> l | x :: xs -> List.rev ((x ^ "." ^ e) :: xs)

  let sync () =
    let* master = store () in
    let remote = remote () in
    let* res = Sync.pull master remote `Set in
    match res with
    | Ok _ -> (
        let* res = Sync.push master remote in
        match res with
        | Ok _ ->
            Lwt.return_ok ()
        | Error p ->
            let* () = Lwt.return @@ Sync.pp_push_error Fmt.stdout p in
            Lwt.return_error "push error" )
    | Error p ->
        let* () = Lwt.return @@ Sync.pp_pull_error Fmt.stdout p in
        Lwt.return_error "pull error"
end