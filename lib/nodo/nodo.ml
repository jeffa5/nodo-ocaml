module T = struct
  type metadata = {due_date: string [@default ""]}
  [@@deriving show {with_path= false}, make]

  type text_type = Plain | Bold | Italic | Code
  [@@deriving show {with_path= false}]

  type text_item = text_type * string [@@deriving show {with_path= false}]

  type text = text_item list [@@deriving show {with_path= false}]

  type list_item = Task of bool * text | Bullet of text
  [@@deriving show {with_path= false}]

  type list_ =
    | Ordered of (int * list_item * list_ option) list
    | Unordered of (list_item * list_ option) list
  [@@deriving show {with_path= false}]

  type block = Paragraph of text | List of list_ | Heading of int * text
  [@@deriving show {with_path= false}]

  type t = metadata * block list [@@deriving show {with_path= false}]
end

module type Format = sig
  val parse : string -> T.t

  val render : T.t -> string

  val extensions : string list
end

module type Storage_types = sig
  type location = string list

  type nodo = [`Nodo of location]

  type project = [`Project of location]

  type t = [nodo | project]
end

module Storage_types = struct
  type location = string list

  type nodo = [`Nodo of location]

  type project = [`Project of location]

  type t = [nodo | project]
end

module type Storage = sig
  include Storage_types

  val read : nodo -> string

  val write : string -> nodo -> unit

  val list : project -> t list

  val create : location -> nodo

  val remove : t -> unit

  val classify : location -> t option

  val name : t -> string
end
