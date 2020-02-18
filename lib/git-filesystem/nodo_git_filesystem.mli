module type Config = sig
  val dir : string

  val remote : string

  val author : string
end

module Make (C : Config) : sig
  include Nodo.Storage
end
