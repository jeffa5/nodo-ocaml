let () =
  let suites = [Storage.suite] in
  Lwt_list.iter_s (fun s -> s ()) suites |> Lwt_main.run
