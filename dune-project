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
 (name nodo-cli)
 (synopsis "The Nodo cli")
 (depends
  (tls
   (>= 0.10.4))
  (nodo :dev)
  (nodo-markdown :dev)))
