open Astring
module Nodo = Nodo.T

let extensions = [ "md" ]

let rec flatten_text l : Nodo.text =
  match l with
  | [] -> []
  | [ x ] -> [ x ]
  | (Nodo.Plain, s) :: (Plain, t) :: xs ->
      let x = s ^ t in
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

and text_contents l = List.map (fun (_, s) -> s) l |> String.concat ~sep:" "

let rec parse_inner_metadata (m : Nodo.metadata) l : Nodo.metadata * Omd.t =
  match l with
  | Omd.Heading h :: xs when h.level = 2 -> (
      let text = parse_text h.text in
      match String.trim (text_contents text) |> String.cut ~sep:":" with
      | None -> parse_inner_metadata m xs
      | Some (l, r) ->
          let l = String.trim l and r = String.trim r in
          let meta : Nodo.metadata =
            match l with "due_date" -> { due_date = r } | _ -> m
          in
          parse_inner_metadata meta xs )
  | Thematic_break :: xs -> (m, xs)
  | _ -> (m, l)

let parse_metadata l =
  match l with
  | Omd.Thematic_break :: xs -> parse_inner_metadata (Nodo.make_metadata ()) xs
  | _ -> (Nodo.make_metadata (), l)

let rec parse_list_item l =
  match l with
  | Omd.Paragraph e :: l -> (
      let nested =
        match l with Omd.List l :: _ -> Some (parse_list l) | _ -> None
      in
      let x = Tyre.(opt blanks *> opt (str "x") <* opt blanks) in
      let box = Tyre.(start *> str "[" *> x <* str "]") in
      let text =
        Tyre.(opt blanks *> regex (Re.rep1 @@ Re.alt [ Re.wordc; Re.blank ]))
      in
      let list_item = Tyre.(box <&> text) in
      let re = Tyre.compile list_item in
      let s, rest =
        match parse_text e with (Plain, s) :: xs -> (s, xs) | xs -> ("", xs)
      in
      match Tyre.exec re s with
      | Error (`NoMatch _) ->
          (Nodo.Bullet (if s = "" then rest else (Plain, s) :: rest), nested)
      | Error (`ConverterFailure _) ->
          print_endline "converter failure";
          assert false
      | Ok (None, s) ->
          ( Nodo.Task (false, if s = "" then rest else (Plain, s) :: rest),
            nested )
      | Ok (Some (), s) ->
          (Nodo.Task (true, if s = "" then rest else (Plain, s) :: rest), nested)
      )
  | t ->
      print_endline @@ Omd.to_sexp t;
      assert false

and parse_list l =
  let items = List.map parse_list_item l.blocks in
  match l.kind with
  | Ordered (_, _) -> Nodo.Ordered (List.mapi (fun i (a, b) -> (i, a, b)) items)
  | Unordered _ -> Nodo.Unordered items

let parse_inline = function
  | Omd.Text s -> (Nodo.Plain, s)
  | i ->
      Omd.to_sexp [ Omd.Paragraph i ] |> print_endline;
      assert false

let parse_element (e : Omd.inline Omd.block) : Nodo.block option =
  match e with
  | Omd.Heading h -> Some (Heading (h.level, [ parse_inline h.text ]))
  | Paragraph i -> Some (Paragraph (parse_text i))
  | List l -> Some (List (parse_list l))
  | _ ->
      Omd.to_sexp [ e ] |> print_endline;
      assert false

let parse_elements e = List.filter_map parse_element e

let parse content =
  let omd = Omd.of_string content in
  let metadata, els = parse_metadata omd in
  let l = parse_elements els in
  (metadata, l)

let render_metadata (m : Nodo.metadata) =
  let due_date =
    if m.due_date = "" then "" else "due_date: " ^ m.due_date ^ "\n"
  in
  let rule = "---\n" in
  let meta_text = due_date in
  if meta_text = "" then "" else rule ^ meta_text ^ rule

let render_text_item t =
  match t with
  | Nodo.Plain, s -> s
  | Italic, s -> "*" ^ s ^ "*"
  | Bold, s -> "**" ^ s ^ "**"
  | Code, s -> "`" ^ s ^ "`"

let render_text t = List.map render_text_item t |> String.concat ~sep:" "

let render_list_item ~prefix i =
  match i with
  | Nodo.Task (b, t) ->
      prefix ^ "[" ^ (if b then "x" else " ") ^ "] " ^ render_text t
  | Bullet t -> prefix ^ render_text t

let rec render_list ?(prefix = "") l =
  ( match l with
  | Nodo.Ordered l ->
      List.map
        (fun (i, li, l) ->
          prefix
          ^ render_list_item ~prefix:(string_of_int i ^ ". ") li
          ^
          match l with
          | None -> ""
          | Some l -> "\n" ^ render_list ~prefix:(prefix ^ "  ") l)
        l
  | Unordered l ->
      List.map
        (fun (li, l) ->
          prefix
          ^ render_list_item ~prefix:"- " li
          ^
          match l with
          | None -> ""
          | Some l -> "\n" ^ render_list ~prefix:(prefix ^ "  ") l)
        l )
  |> String.concat ~sep:"\n"

let render_block b =
  match b with
  | Nodo.Heading (l, t) -> String.v ~len:l (fun _ -> '#') ^ " " ^ render_text t
  | Paragraph t -> render_text t
  | List l -> render_list l

let render (m, bs) =
  let meta = render_metadata m in
  let blocks = List.map render_block bs |> String.concat ~sep:"\n\n" in
  meta ^ "\n" ^ blocks

let test_parse t = parse t |> Nodo.show |> print_endline

let test_render n = render n |> print_endline

let%expect_test "Empty text gives empty nodo" =
  test_parse "";
  [%expect {| ({ due_date = "" }, []) |}]

let%expect_test "reading in a heading" =
  test_parse "# A level 1 heading";
  [%expect
    {| ({ due_date = "" }, [(Heading (1, [(Plain, "A level 1 heading")]))]) |}];
  test_parse "## A level 2 heading";
  [%expect
    {| ({ due_date = "" }, [(Heading (2, [(Plain, "A level 2 heading")]))]) |}];
  test_parse "### A level 3 heading";
  [%expect
    {| ({ due_date = "" }, [(Heading (3, [(Plain, "A level 3 heading")]))]) |}];
  test_parse "#### A level 4 heading";
  [%expect
    {| ({ due_date = "" }, [(Heading (4, [(Plain, "A level 4 heading")]))]) |}];
  test_parse "##### A level 5 heading";
  [%expect
    {| ({ due_date = "" }, [(Heading (5, [(Plain, "A level 5 heading")]))]) |}];
  test_parse "###### A level 6 heading";
  [%expect
    {| ({ due_date = "" }, [(Heading (6, [(Plain, "A level 6 heading")]))]) |}]

let%expect_test "reading in metadata" =
  test_parse "---\n due_date: test\n  ---";
  [%expect {| ({ due_date = "test" }, []) |}];
  test_parse "---\n---";
  [%expect {| ({ due_date = "" }, []) |}]

let%expect_test "reading in a plain list" =
  test_parse "- some text";
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Bullet [(Plain, "some text")]), None)]))]) |}]

let%expect_test "reading in an incomplete task list" =
  test_parse "- [] some text";
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Task (false, [(Plain, "some text")])), None)]))]) |}];
  test_parse "-   [   ]   some text";
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Task (false, [(Plain, "some text")])), None)]))]) |}]

let%expect_test "reading in a complete task list" =
  test_parse "- [x] some text";
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Task (true, [(Plain, "some text")])), None)]))]) |}];
  test_parse "-    [   x  ]   some text";
  [%expect
    {|
      ({ due_date = "" },
       [(List (Unordered [((Task (true, [(Plain, "some text")])), None)]))]) |}]

let%expect_test "reading in a nested list" =
  test_parse "- text\n  - nested";
  [%expect
    {|
    ({ due_date = "" },
     [(List
         (Unordered
            [((Bullet [(Plain, "text")]),
              (Some (Unordered [((Bullet [(Plain, "nested")]), None)])))]))
       ]) |}]

let%expect_test "reading in a heading after metadata" =
  test_parse "---\n---\n# A level 1 heading";
  [%expect
    {| ({ due_date = "" }, [(Heading (1, [(Plain, "A level 1 heading")]))]) |}];
  test_parse "---\n---\n\n\n# A level 1 heading";
  [%expect
    {| ({ due_date = "" }, [(Heading (1, [(Plain, "A level 1 heading")]))]) |}];
  test_parse "---\ndue_date: test\n---\n\n# A level 1 heading";
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
  [%expect {| ({ due_date = "" }, [(Paragraph [(Code, "code")])])|}];
  test_parse "`code text`";
  [%expect {| ({ due_date = "" }, [(Paragraph [(Code, "code text")])])|}]

let%expect_test "render metadata" =
  test_render (Nodo.make_metadata (), []);
  [%expect {| |}];
  test_render ({ due_date = "test" }, []);
  [%expect {|
    ---
    due_date: test
    --- |}]

let%expect_test "render paragraph block" =
  test_render (Nodo.make_metadata (), [ Nodo.Paragraph [ (Plain, "text") ] ]);
  [%expect {| text |}]

let%expect_test "render heading block" =
  test_render (Nodo.make_metadata (), [ Nodo.Heading (1, [ (Plain, "text") ]) ]);
  [%expect {| # text |}];
  test_render (Nodo.make_metadata (), [ Nodo.Heading (2, [ (Plain, "text") ]) ]);
  [%expect {| ## text |}];
  test_render (Nodo.make_metadata (), [ Nodo.Heading (3, [ (Plain, "text") ]) ]);
  [%expect {| ### text |}];
  test_render (Nodo.make_metadata (), [ Nodo.Heading (4, [ (Plain, "text") ]) ]);
  [%expect {| #### text |}];
  test_render (Nodo.make_metadata (), [ Nodo.Heading (5, [ (Plain, "text") ]) ]);
  [%expect {| ##### text |}];
  test_render (Nodo.make_metadata (), [ Nodo.Heading (6, [ (Plain, "text") ]) ]);
  [%expect {| ###### text |}]

let%expect_test "render unordered bullet list" =
  test_render
    ( Nodo.make_metadata (),
      [ Nodo.List (Unordered [ (Bullet [ (Plain, "text") ], None) ]) ] );
  [%expect {| - text |}];
  test_render
    ( Nodo.make_metadata (),
      [
        Nodo.List
          (Unordered
             [
               (Bullet [ (Plain, "text") ], None);
               (Bullet [ (Plain, "next") ], None);
             ]);
      ] );
  [%expect {|
    - text
    - next |}];
  test_render
    ( Nodo.make_metadata (),
      [
        Nodo.List
          (Unordered
             [
               ( Bullet [ (Plain, "text") ],
                 Some (Unordered [ (Bullet [ (Plain, "text") ], None) ]) );
               (Bullet [ (Plain, "next") ], None);
             ]);
      ] );
  [%expect {|
    - text
      - text
    - next |}];
  test_render
    ( Nodo.make_metadata (),
      [
        Nodo.List
          (Unordered
             [
               ( Bullet [ (Plain, "text") ],
                 Some (Ordered [ (1, Bullet [ (Plain, "text") ], None) ]) );
               (Bullet [ (Plain, "next") ], None);
             ]);
      ] );
  [%expect {|
    - text
      1. text
    - next |}]

let%expect_test "render ordered bullet list" =
  test_render
    ( Nodo.make_metadata (),
      [ Nodo.List (Ordered [ (1, Bullet [ (Plain, "text") ], None) ]) ] );
  [%expect {| 1. text |}];
  test_render
    ( Nodo.make_metadata (),
      [
        Nodo.List
          (Ordered
             [
               (1, Bullet [ (Plain, "text") ], None);
               (2, Bullet [ (Plain, "next") ], None);
             ]);
      ] );
  [%expect {|
    1. text
    2. next |}];
  test_render
    ( Nodo.make_metadata (),
      [
        Nodo.List
          (Ordered
             [
               ( 1,
                 Bullet [ (Plain, "text") ],
                 Some (Ordered [ (1, Bullet [ (Plain, "text") ], None) ]) );
               (2, Bullet [ (Plain, "next") ], None);
             ]);
      ] );
  [%expect {|
    1. text
      1. text
    2. next |}];
  test_render
    ( Nodo.make_metadata (),
      [
        Nodo.List
          (Ordered
             [
               ( 1,
                 Bullet [ (Plain, "text") ],
                 Some (Unordered [ (Bullet [ (Plain, "text") ], None) ]) );
               (2, Bullet [ (Plain, "next") ], None);
             ]);
      ] );
  [%expect {|
    1. text
      - text
    2. next |}]
