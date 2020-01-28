module type S = sig
  type t = { hidden_dirs : string list }

  val default : t
end

module V = struct
  type t = { hidden_dirs : string list }

  let default = { hidden_dirs = [ "archive"; "temp" ] }
end
