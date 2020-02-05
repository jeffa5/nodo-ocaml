module Nodo = struct
  type metadata = { due_date : string [@default ""] }
  [@@deriving show { with_path = false }, make]

  type text_type = Plain | Bold | Italic | Code of string
  [@@deriving show { with_path = false }]

  type text_item = text_type * string [@@deriving show { with_path = false }]

  type text = text_item list [@@deriving show { with_path = false }]

  type list_type = Ordered | Unordered [@@deriving show { with_path = false }]

  type list_item = Task of bool * text | Bullet of text
  [@@deriving show { with_path = false }]

  type list_ =
    | Ordered of (int * list_item * list_ option) list
    | Unordered of (list_item * list_ option) list
  [@@deriving show { with_path = false }]

  type block = Paragraph of text list | List of list_ | Heading of int * text
  [@@deriving show { with_path = false }]

  type t = metadata * block list [@@deriving show { with_path = false }]
end

module type Format = sig
  val parse : string -> (Nodo.t, string) result

  val render : Nodo.t -> (string, string) result

  val extensions : string list
end

module type Storage_types = sig
  type nodo = [ `Nodo of string ]

  type project = [ `Project of string ]

  type t = [ nodo | project ]
end

module Storage_types = struct
  type nodo = [ `Nodo of string ]

  type project = [ `Project of string ]

  type t = [ nodo | project ]
end

module type Storage = sig
  include Storage_types

  val read : nodo -> string

  val write : string -> nodo -> unit

  val list : project -> t list

  val create : string -> nodo

  val remove : t -> unit

  val classify : string -> t option

  val name : t -> string
end
