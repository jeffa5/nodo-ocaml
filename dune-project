(lang dune 2.3)

(strict_package_deps)

(name nodo)

(generate_opam_files true)

(authors "Andrew Jeffery <dev@jeffas.io>")

(maintainers "Andrew Jeffery <dev@jeffas.io>")

(source
 (github jeffa5/nodo))

(package
 (name nodo)
 (synopsis "A notes and todo organiser")
 (depends
  (ppx_deriving
   (>= 4.4))
  (lwt
   (>= 5.1))
  (ISO8601
   (= 0.2.6))))

(package
 (name nodo-markdown)
 (synopsis "A markdown formatting backend for Nodo")
 (depends
  (tyre
   (>= 0.5))
  (omd
   (>= 2.0.0))
  (astring
   (= 0.8.3))
  (nodo :dev)
  (ppx_expect
   (>= v0.13.0))))

(package
 (name nodo-json)
 (synopsis "A json formatting backend for Nodo")
 (depends
  (ppx_deriving_yojson
   (>= 3.5.1))
  (ppx_import
   (>= 1.7.1))
  (nodo :dev)
  (ppx_expect
   (>= v0.13.0))))

(package
 (name nodo-cli)
 (synopsis "The Nodo cli")
 (depends
  (tls
   (>= 0.10.4))
  (nodo :dev)
  (nodo-markdown :dev)
  (nodo-git-binary :dev)))
