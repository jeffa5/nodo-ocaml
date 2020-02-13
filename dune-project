(lang dune 2.1)

(name nodo)

(generate_opam_files true)

(authors "Andrew Jeffery <dev@jeffas.io>")

(maintainers "Andrew Jeffery <dev@jeffas.io>")

(source
 (github jeffa5/nodo))

(package
 (name nodo)
 (synopsis "A notes and todo organiser"))

(package
 (name nodo-markdown)
 (synopsis "A markdown formatting backend for Nodo")
 (depends
  (nodo :dev)
  (dune
   (>= 2.0.0))))

(package
 (name nodo-filesystem)
 (synopsis "A filesystem storage backend for Nodo")
 (depends
  (nodo :dev)
  (dune
   (>= 2.0.0))))

(package
 (name nodo-cli-lib)
 (synopsis "A cli frontend for Nodo")
 (depends
  (nodo :dev)
  (dune
   (>= 2.0.0))))

(package
 (name nodo-cli)
 (synopsis "The Nodo cli")
 (depends
  (nodo-markdown :dev)
  (nodo-filesystem :dev)
  (nodo-cli-lib :dev)
  (dune
   (>= 2.0.0))))
