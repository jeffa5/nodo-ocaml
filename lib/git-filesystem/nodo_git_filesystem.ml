open Option
open Stdlib.Result

let ( let* ) = Lwt.bind

let ( let+ ) x y = Lwt.map y x

module type Remote = sig
  val remote : string
end

module Make (R : Remote) = struct
  module Store = Irmin_unix.Git.FS.KV (Irmin.Contents.String)
  module Sync = Irmin.Sync (Store)

  let remote = Store.remote R.remote

  let irmin_config = Irmin_git.config ~bare:true "/tmp/irmin/test"

  let irmin_info fmt = Irmin_unix.info ~author:"andrew" fmt

  include Nodo.Storage_types

  type n = location

  type nodo = [`Nodo of n]

  type p = location

  type project = [`Project of p]

  type t = [nodo | project]

  let read (`Nodo p) =
    let* repo = Store.Repo.v irmin_config in
    let* master = Store.master repo in
    let* contents = Store.get master p in
    Lwt.return_ok contents

  let write (`Nodo p) content =
    let* repo = Store.Repo.v irmin_config in
    let* master = Store.master repo in
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
    let* repo = Store.Repo.v irmin_config in
    let* master = Store.master repo in
    let+ l = Store.list master p in
    l
    |> List.map (function
         | s, `Contents ->
             `Project (p @ [s])
         | s, `Node ->
             `Nodo (p @ [s]))
    |> ok

  let classify p =
    let* repo = Store.Repo.v irmin_config in
    let* master = Store.master repo in
    let* kind = Store.kind master p in
    match kind with
    | None ->
        Lwt.return_none
    | Some `Contents ->
        Lwt.return_some (`Nodo p)
    | Some `Node ->
        Lwt.return_some (`Project p)

  let create l =
    let nodo = `Nodo l in
    write nodo "" |> Lwt_result.map (fun () -> nodo)

  let remove t =
    let* repo = Store.Repo.v irmin_config in
    let* master = Store.master repo in
    ( match t with
    | `Nodo n ->
        Store.remove master ~info:(irmin_info "removing item") n
    | `Project p ->
        Store.remove master ~info:(irmin_info "removing item") p )
    |> Lwt_result.map_err (function
         | `Too_many_retries _ ->
             "too many retries"
         | `Test_was _ ->
             "test was"
         | `Conflict s ->
             s)

  let name t =
    let parts = (match t with `Nodo n -> n | `Project p -> p) |> List.rev in
    match parts with [] -> "" | r :: _ -> r

  let with_extension (`Nodo l) e =
    match List.rev l with
    | [] ->
        `Nodo l
    | x :: xs ->
        `Nodo (List.rev ((x ^ "." ^ e) :: xs))

  let sync () =
    let* repo = Store.Repo.v irmin_config in
    let* master = Store.master repo in
    let* res = Sync.pull master remote `Set in
    match res with
    | Ok _ -> (
        let* res = Sync.push master remote in
        match res with
        | Ok _ ->
            Lwt.return_ok ()
        | Error _ ->
            Lwt.return_error "push error" )
    | Error _ ->
        Lwt.return_error "pull error"
end
