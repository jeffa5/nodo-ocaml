module type Config = sig
  val dir : string ref

  val remote : string ref

  val remote_headers : Cohttp.Header.t option ref

  val author : string ref
end

module Make (C : Config) : sig
  include Nodo.Storage
end
