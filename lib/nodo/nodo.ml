module T = struct
  type metadata = {due_date: string [@default ""]}
  [@@deriving show {with_path= false}, make]
  (** [metadata] is the available metadata about a nodo *)

  (** [text_type] is the possible stylings of text *)
  type text_type = Plain | Bold | Italic | Code
  [@@deriving show {with_path= false}]

  type text_item = text_type * string [@@deriving show {with_path= false}]
  (** [text_item] is a style and a string *)

  type text = text_item list [@@deriving show {with_path= false}]
  (** [text] is a list of styled strings *)

  (** [list_item] is an item of either an ordered or unordered list *)
  type list_item = Task of bool * text | Bullet of text
  [@@deriving show {with_path= false}]

  (** [list_] is either an ordered or unordered list *)
  type list_ =
    | Ordered of (int * list_item * list_ option) list
    | Unordered of (list_item * list_ option) list
  [@@deriving show {with_path= false}]

  (** [block] is a content block *)
  type block = Paragraph of text | List of list_ | Heading of int * text
  [@@deriving show {with_path= false}]

  type t = metadata * block list [@@deriving show {with_path= false}]
  (** [t] is the root type for a nodo. It contains some metadata about the nodo and a list of the content blocks *)
end

module type Format = sig
  val parse : string -> T.t
  (** [parse s] attempts to parse [s] into a t *)

  val render : T.t -> string
  (** [render t] formats [t] *)

  val extensions : string list
  (** [extensions] represents the list of associated extensions for this format. E.g. md for markdown. This list should be sorted in order of preference (first being most preferred) *)
end

module type Storage_types = sig
  type n

  type nodo = [`Nodo of n]

  type p

  type project = [`Project of p]

  type t = [nodo | project]

  type location = string list
end

module Storage_types = struct
  type location = string list
end

module type Storage = sig
  include Storage_types

  val read : nodo -> (string, string) Lwt_result.t
  (** [read n] reads the nodo and returns the entire contents *)

  val write : nodo -> string -> (unit, string) Lwt_result.t
  (** [write s n] writes [s] (likely from a formatter) to the given nodo *)

  val list : project -> (t list, string) Lwt_result.t
  (** [list p] returns the list of projects and nodos that are immediate children of the project  *)

  val create : location -> (nodo, string) Lwt_result.t
  (** [create l] creates a nodo at the given location *)

  val remove : [< nodo | project] -> (unit, string) Lwt_result.t
  (** [remove t] removes [t], regardless of whether it is a project or nodo. If [t] is a project then it should remove all contained nodos and projects before removing itself *)

  val classify : location -> t option Lwt.t
  (** [classify l] attempts to classify the given location as either a project or a nodo *)

  val name : [< nodo | project] -> string
  (** [name t] returns the name part of the location in [t] *)

  val with_extension : nodo -> string -> nodo
  (** [with_format n e] returns [n] with the format (extension) added *)

  val sync : unit -> (unit, string) Lwt_result.t
  (** [sync ()] asks the storage module to sync the current state with a configured remote state *)
end
