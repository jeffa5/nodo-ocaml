let parse _config _content = Nodo.Markdown.parse ""

let render _config _nodo = ""

let formats = []

let find_format_from_extension ext fs =
  List.find_opt
    (fun f ->
      let module F = (val f : Nodo.Format) in
      List.find_opt (fun e -> e = ext) F.extensions |> Option.is_some)
    fs
