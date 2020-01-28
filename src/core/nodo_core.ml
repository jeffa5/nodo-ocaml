module Nodo = struct
  type metadata = { due_date : float }

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
end
