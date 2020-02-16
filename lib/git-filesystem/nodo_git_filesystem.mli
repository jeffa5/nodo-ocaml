module type Remote = sig
  val remote : string
end

module Make (R : Remote) : sig
  include Nodo.Storage
end
