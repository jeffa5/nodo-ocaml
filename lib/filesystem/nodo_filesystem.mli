module type Prefix_type = sig
  val prefix : string list
end

module Make (Prefix : Prefix_type) : sig
  include Nodo.Storage
end
