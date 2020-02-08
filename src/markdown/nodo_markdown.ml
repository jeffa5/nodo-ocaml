open Astring
module Nodo = Nodo_core.Nodo

let extensions = [ "md" ]

let rec flatten_text l : Nodo.text =
  match l with
  | [] -> []
  | [ x ] -> [ x ]
  | (Nodo.Plain, s) :: (Plain, t) :: xs ->
      let x = String.trim (s ^ t) in
      flatten_text ((Plain, x) :: xs)
  | x :: xs -> x :: flatten_text xs

let rec parse_text i =
  match i with
  | Omd.Concat l -> List.map parse_text l |> List.flatten |> flatten_text
  | Text s -> [ (Nodo.Plain, s) ]
  | Emph e -> (
      match e.kind with
      | Normal -> (
          match e.content with
          | Omd.Text s -> [ (Italic, s) ]
          | _ ->
              Omd.to_sexp [ Omd.Paragraph i ] |> print_endline;
              assert false )
      | Strong -> (
          match e.content with
          | Omd.Text s -> [ (Bold, s) ]
          | _ ->
              Omd.to_sexp [ Omd.Paragraph i ] |> print_endline;
              assert false ) )
  | Code c -> [ (Code, c.content) ]
  | i ->
      Omd.to_sexp [ Omd.Paragraph i ] |> print_endline;
      assert false

and text_contents l =
  List.map (function Nodo.Plain, s -> s | _, s -> s) l
  |> String.concat ~sep:" "

let rec parse_inner_metadata (m : Nodo_core.Nodo.metadata) l :
    Nodo_core.Nodo.metadata * Omd.t =
  match l with
  | Omd.Heading h :: xs when h.level = 2 -> (
      let text = parse_text h.text in
      match String.trim (text_contents text) |> String.cut ~sep:":" with
      | None -> parse_inner_metadata m xs
      | Some (l, r) ->
          let l = String.trim l and r = String.trim r in
          let meta : Nodo_core.Nodo.metadata =
            match l with "due_date" -> { due_date = r } | _ -> m
          in
          parse_inner_metadata meta xs )
  | Thematic_break :: xs -> (m, xs)
  | _ -> (m, l)

let parse_metadata l =
  match l with
  | Omd.Thematic_break :: xs -> parse_inner_metadata (Nodo.make_metadata ()) xs
  | _ -> (Nodo_core.Nodo.make_metadata (), l)

let parse_list_item l =
  match l with
  | Omd.Paragraph e :: _ -> (
      let x = Tyre.(opt blanks *> opt (str "x") <* opt blanks) in
      let box = Tyre.(start *> str "[" *> x <* str "]") in
      let text =
        Tyre.(opt blanks *> regex (Re.rep1 @@ Re.alt [ Re.wordc; Re.blank ]))
      in
      let list_item = Tyre.(box <&> text) in
      let re = Tyre.compile list_item in
      let s = parse_text e |> text_contents in
      match Tyre.exec re s with
      | Error (`NoMatch _) -> (Nodo.Bullet [ (Plain, s) ], None)
      | Error (`ConverterFailure _) ->
          print_endline "converter failure";
          assert false
      | Ok (None, s) -> (Nodo.Task (false, [ (Plain, s) ]), None)
      | Ok (Some (), s) -> (Nodo.Task (true, [ (Plain, s) ]), None) )
  | t ->
      print_endline @@ Omd.to_sexp t;
      assert false

let parse_inline = function
  | Omd.Text s -> (Nodo.Plain, s)
  | i ->
      Omd.to_sexp [ Omd.Paragraph i ] |> print_endline;
      assert false

let parse_element (e : Omd.inline Omd.block) : Nodo_core.Nodo.block option =
  match e with
  | Omd.Heading h -> Some (Heading (h.level, [ parse_inline h.text ]))
  | Paragraph i -> Some (Paragraph (parse_text i))
  | List l -> (
      let items = List.map parse_list_item l.blocks in
      match l.kind with
      | Ordered (_, _) -> Some (List (Ordered []))
      | Unordered _ -> Some (List (Unordered items)) )
  | _ ->
      Omd.to_sexp [ e ] |> print_endline;
      assert false

let parse_elements e = List.filter_map parse_element e

let parse content =
  let omd = Omd.of_string content in
  let metadata, els = parse_metadata omd in
  let l = parse_elements els in
  (metadata, l)

let render_metadata (m : Nodo_core.Nodo.metadata) = m.due_date

let render_text_item t = match t with Nodo.Plain, s -> s | _ -> assert false

let render_text t = List.map render_text_item t |> String.concat ~sep:" "

let render_block b =
  match b with
  | Nodo.Heading (l, t) -> String.v ~len:l (fun _ -> '#') ^ render_text t
  | _ -> assert false

let render (m, bs) =
  let meta = render_metadata m in
  let blocks = List.map render_block bs |> String.concat ~sep:"\n" in
  meta ^ "\n" ^ blocks

let test_parse t = parse t |> Nodo.show |> print_endline

let%expect_test "Empty text gives empty nodo" =
  let text = "" in
  test_parse text;
  [%expect {| ({ due_date = "" }, []) |}]

let%expect_test "reading in a heading" =
  let text = "# A level 1 heading" in
  test_parse text;
  [%expect
    {| ({ due_date = "" }, [(Heading (1, [(Plain, "A level 1 heading")]))]) |}];
  let text = "## A level 2 heading" in
  test_parse text;
  [%expect
    {| ({ due_date = "" }, [(Heading (2, [(Plain, "A level 2 heading")]))]) |}];
  let text = "### A level 3 heading" in
  test_parse text;
  [%expect
    {| ({ due_date = "" }, [(Heading (3, [(Plain, "A level 3 heading")]))]) |}];
  let text = "#### A level 4 heading" in
  test_parse text;
  [%expect
    {| ({ due_date = "" }, [(Heading (4, [(Plain, "A level 4 heading")]))]) |}];
  let text = "##### A level 5 heading" in
  test_parse text;
  [%expect
    {| ({ due_date = "" }, [(Heading (5, [(Plain, "A level 5 heading")]))]) |}];
  let text = "###### A level 6 heading" in
  test_parse text;
  [%expect
    {| ({ due_date = "" }, [(Heading (6, [(Plain, "A level 6 heading")]))]) |}]

let%expect_test "reading in metadata" =
  let text = "---\n due_date: test\n  ---" in
  test_parse text;
  [%expect {| ({ due_date = "test" }, []) |}];
  let text = "---\n---" in
  test_parse text;
  [%expect {| ({ due_date = "" }, []) |}]

let%expect_test "reading in a plain list" =
  let text = "- some text" in
  test_parse text;
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Bullet [(Plain, "some text")]), None)]))]) |}]

let%expect_test "reading in an incomplete task list" =
  let text = "- [] some text" in
  test_parse text;
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Task (false, [(Plain, "some text")])), None)]))]) |}];
  let text = "-   [   ]   some text" in
  test_parse text;
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Task (false, [(Plain, "some text")])), None)]))]) |}]

let%expect_test "reading in a complete task list" =
  let text = "- [x] some text" in
  test_parse text;
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Task (true, [(Plain, "some text")])), None)]))]) |}];
  let text = "-    [   x  ]   some text" in
  test_parse text;
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Task (true, [(Plain, "some text")])), None)]))]) |}]

let%expect_test "reading in a heading after metadata" =
  let text = "---\n---\n# A level 1 heading" in
  test_parse text;
  [%expect
    {| ({ due_date = "" }, [(Heading (1, [(Plain, "A level 1 heading")]))]) |}];
  let text = "---\n---\n\n\n# A level 1 heading" in
  test_parse text;
  [%expect
    {| ({ due_date = "" }, [(Heading (1, [(Plain, "A level 1 heading")]))]) |}];
  let text = "---\ndue_date: test\n---\n\n# A level 1 heading" in
  test_parse text;
  [%expect
    {| ({ due_date = "test" }, [(Heading (1, [(Plain, "A level 1 heading")]))]) |}]

let%expect_test "reading in bold text" =
  test_parse "**bold**";
  [%expect {| ({ due_date = "" }, [(Paragraph [(Bold, "bold")])])|}];
  test_parse "__bold__";
  [%expect {| ({ due_date = "" }, [(Paragraph [(Bold, "bold")])])|}]

let%expect_test "reading in italic text" =
  test_parse "*italic*";
  [%expect {| ({ due_date = "" }, [(Paragraph [(Italic, "italic")])])|}];
  test_parse "_italic_";
  [%expect {| ({ due_date = "" }, [(Paragraph [(Italic, "italic")])])|}]

let%expect_test "reading in inline code text" =
  test_parse "`code`";
  [%expect {| ({ due_date = "" }, [(Paragraph [(Code, "code")])])|}]
