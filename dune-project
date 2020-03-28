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
  (nodo :dev)
  (ppx_expect :with-test)))

(package
 (name nodo-filesystem)
 (synopsis "A filesystem storage backend for Nodo")
 (depends
  (nodo :dev)
  (astring
   (= 0.8.3))
  (fileutils
   (= 0.6.1))))

(package
 (name nodo-git-filesystem)
 (synopsis "A git filesystem storage backend for Nodo")
 (depends
  (irmin-git
   (>= 2.0.0))
  (irmin-unix
   (>= 2.0.0))
  (nodo :dev)))

(package
 (name nodo-cli-lib)
 (synopsis "A cli frontend for Nodo")
 (depends
  (astring
   (>= 0.8.0))
  (cmdliner
   (>= 1.0.0))
  (fmt
   (>= 0.8.0))
  (logs
   (>= 0.7.0))
  (nodo :dev)))

(package
 (name nodo-cli)
 (synopsis "The Nodo cli")
 (depends
  (tls
   (>= 0.10.4))
  (ppx_expect
   (>= v0.13.0))
  (nodo :dev)
  (nodo-markdown :dev)
  (nodo-git-filesystem :dev)
  (nodo-cli-lib :dev)))
