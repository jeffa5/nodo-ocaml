let find_format_from_extension ext =
  List.find_opt
    (fun f ->
      let module F = (val f : Nodo.Format) in
      List.find_opt (fun e -> e = ext) F.extensions |> Option.is_some)
    [(module Nodo.Markdown)]
