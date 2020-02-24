module type Config = sig
  val dir : string

  val remote : string

  val remote_headers : Cohttp.Header.t option

  val author : string
end

module Make (C : Config) : sig
  include Nodo.Storage
end
