open Astring
open Omd

let extensions = ["md"]

let rec flatten_text l : Nodo_core.S.text =
  match l with
  | [] ->
      []
  | [x] ->
      [x]
  | (Nodo_core.S.Plain, s) :: (Plain, t) :: xs ->
      let x = s ^ t in
      flatten_text ((Plain, x) :: xs)
  | x :: xs ->
      x :: flatten_text xs

let rec parse_text i =
  match i.il_desc with
  | Concat l ->
      List.map parse_text l |> List.flatten |> flatten_text
  | Text s ->
      let s = String.trim s in
      [(Nodo_core.S.Plain, s)]
  | Emph i -> (
    match i.il_desc with
    | Text s ->
        [(Italic, s)]
    | _ ->
        Omd.to_sexp [{bl_desc= Paragraph i; bl_attributes= []}] |> print_endline ;
        assert false )
  | Code c ->
      [(Code, c)]
  | Soft_break ->
      []
  | Strong i -> (
    match i.il_desc with
    | Text s ->
        [(Bold, s)]
    | _ ->
        Omd.to_sexp [{bl_desc= Paragraph i; bl_attributes= []}] |> print_endline ;
        assert false )
  | _ ->
      Omd.to_sexp [{bl_desc= Paragraph i; bl_attributes= []}] |> print_endline ;
      assert false

and text_contents l = List.map (fun (_, s) -> s) l |> String.concat ~sep:" "

let rec parse_inner_metadata (m : Nodo_core.S.metadata) l :
    Nodo_core.S.metadata * Omd.doc =
  match l with
  | {bl_desc= Heading (h, i); bl_attributes= _} :: xs when h = 2 -> (
      let text = parse_text i in
      match String.trim (text_contents text) |> String.cut ~sep:":" with
      | None ->
          parse_inner_metadata m xs
      | Some (l, r) ->
          let l = String.trim l and r = String.trim r in
          let meta : Nodo_core.S.metadata =
            match l with "due_date" -> {due_date= r} | _ -> m
          in
          parse_inner_metadata meta xs )
  | {bl_desc= Thematic_break; bl_attributes= _} :: xs ->
      (m, xs)
  | _ ->
      (m, l)

let parse_metadata l =
  match l with
  | {bl_desc= Thematic_break; bl_attributes= _} :: xs ->
      parse_inner_metadata (Nodo_core.S.make_metadata ()) xs
  | _ ->
      (Nodo_core.S.make_metadata (), l)

let rec parse_list_item l =
  match l with
  | {bl_desc= Paragraph e; bl_attributes= _} :: l -> (
      let nested =
        match l with
        | {bl_desc= List (lt, _, bs); bl_attributes= _} :: _ ->
            Some (parse_list lt bs)
        | _ ->
            None
      in
      let x = Tyre.(opt blanks *> opt (str "x") <* opt blanks) in
      let box = Tyre.(start *> str "[" *> x <* str "]") in
      let text = Tyre.(opt blanks *> regex (Re.rep1 @@ Re.any)) in
      let list_item = Tyre.(box <&> text) in
      let re = Tyre.compile list_item in
      let s, rest =
        match parse_text e with (Plain, s) :: xs -> (s, xs) | xs -> ("", xs)
      in
      match Tyre.exec re s with
      | Error (`NoMatch _) ->
          ( Nodo_core.S.Bullet (if s = "" then rest else (Plain, s) :: rest)
          , nested )
      | Error (`ConverterFailure _) ->
          print_endline "converter failure" ;
          assert false
      | Ok (None, s) ->
          ( Nodo_core.S.Task (false, if s = "" then rest else (Plain, s) :: rest)
          , nested )
      | Ok (Some (), s) ->
          ( Nodo_core.S.Task (true, if s = "" then rest else (Plain, s) :: rest)
          , nested ) )
  | t ->
      print_endline @@ Omd.to_sexp t ;
      assert false

and parse_list lt bs =
  let items = List.map parse_list_item bs in
  match lt with
  | Ordered (_, _) ->
      Nodo_core.S.Ordered (List.mapi (fun i (a, b) -> (i + 1, a, b)) items)
  | Bullet _ ->
      Nodo_core.S.Unordered items

let parse_inline = function
  | {il_desc= Text s; il_attributes= _} ->
      (Nodo_core.S.Plain, s)
  | i ->
      Omd.to_sexp [{bl_desc= Paragraph i; bl_attributes= []}] |> print_endline ;
      assert false

let parse_element e : Nodo_core.S.block option =
  match e.bl_desc with
  | Heading (h, i) ->
      Some (Heading (h, [parse_inline i]))
  | Paragraph i ->
      Some (Paragraph (parse_text i))
  | List (lt, _, bs) ->
      Some (List (parse_list lt bs))
  | Code_block (t, content) ->
      Some (Nodo_core.S.Code_block (t, content))
  | Thematic_break ->
      Some Break
  | _ ->
      Omd.to_sexp [e] |> print_endline ;
      assert false

let parse_elements e = List.filter_map parse_element e

let parse content : Nodo_core.S.t =
  let omd = Omd.of_string content in
  let metadata, els = parse_metadata omd in
  let blocks = parse_elements els in
  {metadata; blocks}

let render_metadata (m : Nodo_core.S.metadata) =
  let due_date =
    if m.due_date = "" then "" else "due_date: " ^ m.due_date ^ "\n"
  in
  let rule = "---\n" in
  let meta_text = due_date in
  if meta_text = "" then "" else rule ^ meta_text ^ rule

let render_text_item t =
  match t with
  | Nodo_core.S.Plain, s ->
      s
  | Italic, s ->
      "*" ^ s ^ "*"
  | Bold, s ->
      "**" ^ s ^ "**"
  | Code, s ->
      "`" ^ s ^ "`"

let render_text t = List.map render_text_item t |> String.concat ~sep:" "

let render_list_item ~prefix i =
  match i with
  | Nodo_core.S.Task (b, t) ->
      prefix ^ "[" ^ (if b then "x" else " ") ^ "] " ^ render_text t
  | Bullet t ->
      prefix ^ render_text t

let rec render_list ?(prefix = "") l =
  ( match l with
  | Nodo_core.S.Ordered l ->
      List.map
        (fun (i, li, l) ->
          prefix
          ^ render_list_item ~prefix:(string_of_int i ^ ". ") li
          ^
          match l with
          | None ->
              ""
          | Some l ->
              "\n" ^ render_list ~prefix:(prefix ^ "  ") l)
        l
  | Unordered l ->
      List.map
        (fun (li, l) ->
          prefix
          ^ render_list_item ~prefix:"- " li
          ^
          match l with
          | None ->
              ""
          | Some l ->
              "\n" ^ render_list ~prefix:(prefix ^ "  ") l)
        l )
  |> String.concat ~sep:"\n"

let render_block b =
  match b with
  | Nodo_core.S.Heading (l, t) ->
      String.v ~len:l (fun _ -> '#') ^ " " ^ render_text t
  | Paragraph t ->
      render_text t
  | List l ->
      render_list l
  | Code_block (tag, content) ->
      Printf.sprintf "```%s\n%s```" tag content
  | Break ->
      "---"

let render ({metadata; blocks} : Nodo_core.S.t) =
  let meta = render_metadata metadata in
  let blocks = List.map render_block blocks |> String.concat ~sep:"\n\n" in
  (if meta = "" then "" else meta ^ "\n") ^ blocks

let test_parse t = parse t |> Nodo_core.S.show |> print_endline

let test_render n = render n |> print_endline

let test_format t = parse t |> render |> print_endline

let%expect_test "Empty text gives empty nodo" =
  test_parse "" ; [%expect {| { metadata = { due_date = "" }; blocks = [] } |}]

let%expect_test "reading in a heading" =
  test_parse "# A level 1 heading" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks = [(Heading (1, [(Plain, "A level 1 heading")]))] } |}] ;
  test_parse "## A level 2 heading" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks = [(Heading (2, [(Plain, "A level 2 heading")]))] } |}] ;
  test_parse "### A level 3 heading" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks = [(Heading (3, [(Plain, "A level 3 heading")]))] } |}] ;
  test_parse "#### A level 4 heading" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks = [(Heading (4, [(Plain, "A level 4 heading")]))] } |}] ;
  test_parse "##### A level 5 heading" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks = [(Heading (5, [(Plain, "A level 5 heading")]))] } |}] ;
  test_parse "###### A level 6 heading" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks = [(Heading (6, [(Plain, "A level 6 heading")]))] } |}]

let%expect_test "reading in metadata" =
  test_parse "---\n due_date: test\n  ---" ;
  [%expect {| { metadata = { due_date = "test" }; blocks = [] } |}] ;
  test_parse "---\n---" ;
  [%expect {| { metadata = { due_date = "" }; blocks = [] } |}]

let%expect_test "reading in a plain list" =
  test_parse "- some text" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks = [(List (Unordered [((Bullet [(Plain, "some text")]), None)]))] } |}]

let%expect_test "reading in an incomplete task list" =
  test_parse "- [] some text" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks =
        [(List (Unordered [((Task (false, [(Plain, "some text")])), None)]))] } |}] ;
  test_parse "-   [   ]   some text" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks =
        [(List (Unordered [((Task (false, [(Plain, "some text")])), None)]))] } |}]

let%expect_test "reading in a complete task list" =
  test_parse "- [x] some text" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks =
        [(List (Unordered [((Task (true, [(Plain, "some text")])), None)]))] } |}] ;
  test_parse "-    [   x  ]   some text" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks =
        [(List (Unordered [((Task (true, [(Plain, "some text")])), None)]))] } |}]

let%expect_test "reading in a nested list" =
  test_parse "- text\n  - nested" ;
  [%expect
    {|
    { metadata = { due_date = "" };
      blocks =
      [(List
          (Unordered
             [((Bullet [(Plain, "text")]),
               (Some (Unordered [((Bullet [(Plain, "nested")]), None)])))]))
        ]
      } |}]

let%expect_test "reading in a heading after metadata" =
  test_parse "---\n---\n# A level 1 heading" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks = [(Heading (1, [(Plain, "A level 1 heading")]))] } |}] ;
  test_parse "---\n---\n\n\n# A level 1 heading" ;
  [%expect
    {|
      { metadata = { due_date = "" };
        blocks = [(Heading (1, [(Plain, "A level 1 heading")]))] } |}] ;
  test_parse "---\ndue_date: test\n---\n\n# A level 1 heading" ;
  [%expect
    {|
      { metadata = { due_date = "test" };
        blocks = [(Heading (1, [(Plain, "A level 1 heading")]))] } |}]

let%expect_test "reading in bold text" =
  test_parse "**bold**" ;
  [%expect
    {| { metadata = { due_date = "" }; blocks = [(Paragraph [(Bold, "bold")])] }|}] ;
  test_parse "__bold__" ;
  [%expect
    {| { metadata = { due_date = "" }; blocks = [(Paragraph [(Bold, "bold")])] }|}]

let%expect_test "reading in italic text" =
  test_parse "*italic*" ;
  [%expect
    {| { metadata = { due_date = "" }; blocks = [(Paragraph [(Italic, "italic")])] }|}] ;
  test_parse "_italic_" ;
  [%expect
    {| { metadata = { due_date = "" }; blocks = [(Paragraph [(Italic, "italic")])] }|}]

let%expect_test "reading in inline code text" =
  test_parse "`code`" ;
  [%expect
    {| { metadata = { due_date = "" }; blocks = [(Paragraph [(Code, "code")])] }|}] ;
  test_parse "`code text`" ;
  [%expect
    {|
    { metadata = { due_date = "" }; blocks = [(Paragraph [(Code, "code text")])]
      }|}]

let%expect_test "parse in code" =
  test_parse "`code`" ;
  [%expect
    {| { metadata = { due_date = "" }; blocks = [(Paragraph [(Code, "code")])] } |}] ;
  test_parse "text `code`" ;
  [%expect
    {|
    { metadata = { due_date = "" };
      blocks = [(Paragraph [(Plain, "text"); (Code, "code")])] } |}] ;
  test_parse "text `code` text" ;
  [%expect
    {|
    { metadata = { due_date = "" };
      blocks = [(Paragraph [(Plain, "text"); (Code, "code"); (Plain, "text")])] } |}]

let%expect_test "render metadata" =
  test_render (Nodo_core.S.make ~metadata:(Nodo_core.S.make_metadata ()) ()) ;
  [%expect {| |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ~due_date:"test" ())
       ()) ;
  [%expect {|
    ---
    due_date: test
    --- |}]

let%expect_test "render paragraph block" =
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:[Nodo_core.S.Paragraph [(Plain, "text")]]
       ()) ;
  [%expect {| text |}]

let%expect_test "render heading block" =
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:[Nodo_core.S.Heading (1, [(Plain, "text")])]
       ()) ;
  [%expect {| # text |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:[Nodo_core.S.Heading (2, [(Plain, "text")])]
       ()) ;
  [%expect {| ## text |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:[Nodo_core.S.Heading (3, [(Plain, "text")])]
       ()) ;
  [%expect {| ### text |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:[Nodo_core.S.Heading (4, [(Plain, "text")])]
       ()) ;
  [%expect {| #### text |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:[Nodo_core.S.Heading (5, [(Plain, "text")])]
       ()) ;
  [%expect {| ##### text |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:[Nodo_core.S.Heading (6, [(Plain, "text")])]
       ()) ;
  [%expect {| ###### text |}]

let%expect_test "render unordered bullet list" =
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:[Nodo_core.S.List (Unordered [(Bullet [(Plain, "text")], None)])]
       ()) ;
  [%expect {| - text |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:
         [ Nodo_core.S.List
             (Unordered
                [ (Bullet [(Plain, "text")], None)
                ; (Bullet [(Plain, "next")], None) ]) ]
       ()) ;
  [%expect {|
    - text
    - next |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:
         [ Nodo_core.S.List
             (Unordered
                [ ( Bullet [(Plain, "text")]
                  , Some (Unordered [(Bullet [(Plain, "text")], None)]) )
                ; (Bullet [(Plain, "next")], None) ]) ]
       ()) ;
  [%expect {|
    - text
      - text
    - next |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:
         [ Nodo_core.S.List
             (Unordered
                [ ( Bullet [(Plain, "text")]
                  , Some (Ordered [(1, Bullet [(Plain, "text")], None)]) )
                ; (Bullet [(Plain, "next")], None) ]) ]
       ()) ;
  [%expect {|
    - text
      1. text
    - next |}]

let%expect_test "render ordered bullet list" =
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:[Nodo_core.S.List (Ordered [(1, Bullet [(Plain, "text")], None)])]
       ()) ;
  [%expect {| 1. text |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:
         [ Nodo_core.S.List
             (Ordered
                [ (1, Bullet [(Plain, "text")], None)
                ; (2, Bullet [(Plain, "next")], None) ]) ]
       ()) ;
  [%expect {|
    1. text
    2. next |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:
         [ Nodo_core.S.List
             (Ordered
                [ ( 1
                  , Bullet [(Plain, "text")]
                  , Some (Ordered [(1, Bullet [(Plain, "text")], None)]) )
                ; (2, Bullet [(Plain, "next")], None) ]) ]
       ()) ;
  [%expect {|
    1. text
      1. text
    2. next |}] ;
  test_render
    (Nodo_core.S.make
       ~metadata:(Nodo_core.S.make_metadata ())
       ~blocks:
         [ Nodo_core.S.List
             (Ordered
                [ ( 1
                  , Bullet [(Plain, "text")]
                  , Some (Unordered [(Bullet [(Plain, "text")], None)]) )
                ; (2, Bullet [(Plain, "next")], None) ]) ]
       ()) ;
  [%expect {|
    1. text
      - text
    2. next |}]

let%expect_test "format" =
  test_format "# test" ;
  [%expect {| # test |}] ;
  test_format "\n\n\n# test" ;
  [%expect {| # test |}] ;
  test_format "- (test1)" ;
  [%expect {| - (test1) |}] ;
  test_format "- something (1)" ;
  [%expect {| - something (1) |}] ;
  test_format "- [ ] something (1)" ;
  [%expect {| - [ ] something (1) |}] ;
  test_format "- [ ] something [1]" ;
  [%expect {| - [ ] something [1] |}] ;
  test_format "- [ ] something {1}" ;
  [%expect {| - [ ] something {1} |}] ;
  test_format "- [x] something (1)" ;
  [%expect {| - [x] something (1) |}] ;
  test_format "- [x] something [1]" ;
  [%expect {| - [x] something [1] |}] ;
  test_format "- [x] something {1}" ;
  [%expect {| - [x] something {1} |}] ;
  test_format "- [x] something 1.2.3" ;
  [%expect {| - [x] something 1.2.3 |}] ;
  test_format "- [x] something '1.2.3'" ;
  [%expect {| - [x] something '1.2.3' |}] ;
  test_format "`code`" ;
  [%expect {| `code` |}] ;
  test_format "text `code`" ;
  [%expect {| text `code` |}] ;
  test_format "text `code` text" ;
  [%expect {| text `code` text |}] ;
  test_format "don't" ;
  [%expect {| don't |}] ;
  test_format "1. one" ;
  [%expect {| 1. one |}] ;
  test_format "1. one\n2. two" ;
  [%expect {|
    1. one
    2. two |}] ;
  test_format
    {|# a title

  a test **paragraph**

  a *test* paragraph _with_
  soft __break__

  - an unordered `list`
  - another element

  ```yaml
  a: code block
  ```

  1. a numbered list
  2. another item

  ---

  that was a rule

  ## another title

  - a
    - b
    - c
  - d

  1. a
      - b
      - c
  3. 2
  |} ;
  [%expect
    {|
    # a title

    a test **paragraph**

    a *test* paragraph *with* soft **break**

    - an unordered `list`
    - another element

    ```yaml
    a: code block
    ```

    1. a numbered list
    2. another item

    ---

    that was a rule

    ## another title

    - a
      - b
      - c
    - d

    1. a
      - b
      - c
    2. 2 |}]
