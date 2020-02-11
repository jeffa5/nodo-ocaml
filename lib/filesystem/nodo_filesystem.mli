module type Prefix_type = sig
  val prefix : string
end

module Make (Prefix : Prefix_type) : sig
  include Nodo.Storage
end
