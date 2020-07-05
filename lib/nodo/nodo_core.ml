module S = struct
  (** [metadata] is the available metadata about a nodo *)
  type metadata = {due_date: string [@default ""]}
  [@@deriving show {with_path= false}, make, yojson]

  (** [text_type] is the possible stylings of text *)
  type text_type = Plain | Bold | Italic | Code
  [@@deriving show {with_path= false}, yojson]

  (** [text_item] is a style and a string *)
  type text_item = text_type * string
  [@@deriving show {with_path= false}, yojson]

  (** [text] is a list of styled strings *)
  type text = text_item list [@@deriving show {with_path= false}, yojson]

  (** [list_item] is an item of either an ordered or unordered list *)
  type list_item = Task of bool * text | Bullet of text
  [@@deriving show {with_path= false}, yojson]

  (** [list_] is either an ordered or unordered list *)
  type list_ =
    | Ordered of (int * list_item * list_ option) list
    | Unordered of (list_item * list_ option) list
  [@@deriving show {with_path= false}, yojson]

  (** [block] is a content block *)
  type block =
    | Paragraph of text
    | List of list_
    | Heading of int * text
    | Code_block of string * string
    | Break
  [@@deriving show {with_path= false}, yojson]

  (** [t] is the root type for a nodo. It contains some metadata about the nodo and a list of the content blocks *)
  type t = {metadata: metadata; blocks: block list}
  [@@deriving show {with_path= false}, make, yojson]
end

module type Format = sig
  val parse : string -> S.t
  (** [parse s] attempts to parse [s] into a t *)

  val render : S.t -> string
  (** [render t] formats [t] *)

  val extensions : string list
  (** [extensions] represents the list of associated extensions for this format. E.g. md for markdown. This list should be sorted in order of preference (first being most preferred) *)
end
