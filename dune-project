(lang dune 2.1)
(name nodo)

(generate_opam_files true)

(authors "Andrew Jeffery <dev@jeffas.io>")
(maintainers "Andrew Jeffery <dev@jeffas.io>")
(source (github jeffa5/nodo))

(package
(name nodo)
(synopsis "A notes and todo organiser")
)

(package
(name nodo-markdown)
(synopsis "A markdown formatting backend for Nodo")
)

(package
(name nodo-filesystem)
(synopsis "A filesystem storage backend for Nodo")
)

(package
(name nodo-cli)
(synopsis "A cli frontend for Nodo")
)
