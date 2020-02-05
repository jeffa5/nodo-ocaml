module Nodo = Nodo_core.Nodo

exception ParseError of string

let extensions = [ "md" ]

let rec parse_inner_metadata (m : Nodo_core.Nodo.metadata) l :
    Nodo_core.Nodo.metadata * Omd.t =
  match l with
  | Omd.Text s :: xs -> (
      match Astring.String.trim s |> Astring.String.cut ~sep:":" with
      | None -> parse_inner_metadata m xs
      | Some (l, r) ->
          let l = Astring.String.trim l and r = Astring.String.trim r in
          let meta : Nodo_core.Nodo.metadata =
            match l with "due_date" -> { due_date = r } | _ -> m
          in
          parse_inner_metadata meta xs )
  | NL :: xs -> parse_inner_metadata m xs
  | Hr :: xs -> (m, xs)
  | xs -> (m, xs)

let parse_metadata l : Nodo_core.Nodo.metadata * Omd.t =
  match l with
  | Omd.Paragraph (Omd.Hr :: meta) :: _ ->
      parse_inner_metadata (Nodo_core.Nodo.make_metadata ()) meta
  | _ -> (Nodo_core.Nodo.make_metadata (), l)

let parse_plaintext e =
  match e with
  | Omd.Text s -> s
  | t ->
      raise @@ ParseError ("Failed to parse plaintext " ^ Omd.to_markdown [ t ])

let rec flatten_text l : Nodo.text =
  match l with
  | [] -> []
  | [ x ] -> [ x ]
  | (Nodo.Plain, s) :: (Plain, t) :: xs -> flatten_text ((Plain, s ^ t) :: xs)
  | x :: xs -> x :: flatten_text xs

let parse_text e : Nodo_core.Nodo.text_item =
  match e with
  | Omd.Text s -> (Plain, s)
  | Emph e -> (Italic, String.concat " " @@ List.map parse_plaintext e)
  | Bold e -> (Bold, String.concat " " @@ List.map parse_plaintext e)
  | Code (n, s) -> (Code n, s)
  | t -> raise @@ ParseError ("Failed to parse text " ^ Omd.to_html [ t ])

let parse_element e : Nodo_core.Nodo.block option =
  match e with
  | Omd.H1 e -> Some (Heading (1, flatten_text @@ List.map parse_text e))
  | H2 _ -> Some (Heading (2, []))
  | H3 _ -> Some (Heading (3, []))
  | H4 _ -> Some (Heading (4, []))
  | H5 _ -> Some (Heading (5, []))
  | H6 _ -> Some (Heading (6, []))
  | Paragraph e -> Some (Paragraph [ List.map parse_text e ])
  | Ul _ -> Some (List (Unordered []))
  | Ol _ -> Some (List (Ordered []))
  | Br -> None
  | NL -> None
  | t -> raise @@ ParseError ("Failed to parse element " ^ Omd.to_html [ t ])

let parse content =
  let omd = Omd.of_string content in
  let metadata, els = parse_metadata omd in
  let l = List.filter_map parse_element els in
  Ok (metadata, l)

let render_metadata (m : Nodo_core.Nodo.metadata) = Ok m.due_date

let render (m, _) =
  let meta = render_metadata m in
  meta

let%expect_test "reading in a heading" =
  let text = "# A level 1 heading" in
  match parse text with
  | Error e -> print_endline e
  | Ok nodo ->
      Nodo.show nodo |> print_endline;
      [%expect
        {| ({ due_date = "" }, [(Heading (1, [(Plain, "A level 1 heading")]))]) |}]

let%expect_test "reading in metadata" =
  let text = "---\n  due_date: test\n  ---" in
  match parse text with
  | Error e -> print_endline e
  | Ok nodo ->
      Nodo.show nodo |> print_endline;
      [%expect {| ({ due_date = "test" }, []) |}]