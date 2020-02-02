module Nodo = struct
  type metadata = { due_date : float }

  type text_type = Plain | Bold | Italic | Code

  type text_item = text_type * string

  type text = text_item list

  type list_type = Ordered | Unordered

  type list_item = Task of bool * text | Plain of text

  type list_ =
    | Ordered of (int * list_item * list_ option) list
    | Unordered of (list_item * list_ option) list

  type block = Paragraph of text list | List of list_

  type t = Omd.t
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
