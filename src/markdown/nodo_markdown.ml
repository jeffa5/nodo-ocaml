open Result

let extensions = [ "md" ]

let parse content = Omd.of_string content |> ok

let render nodo = Omd.to_markdown nodo |> ok
