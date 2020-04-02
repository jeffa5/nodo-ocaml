.PHONY: build
build:
	dune build

.PHONY: watch
watch:
	dune build @runtest --watch

.PHONY: check
check:
	dune build @check

.PHONY: test
test:
	dune runtest

.PHONY: format
format:
	dune build @fmt --auto-promote
	dune format-dune-file dune-project > dune-project.bak && mv dune-project.bak dune-project

.PHONY: promote
promote:
	dune promote

.PHONY: install
install: build
	dune build @install
	dune install
	cp zcomp ~/.zsh/functions/_nodo

.PHONY: coverage
coverage:
	BISECT_ENABLE=yes dune runtest --force
	bisect-ppx-report html

.PHONY: completion
completion:
	cp zcomp ~/.zsh/functions/_nodo

.PHONY: ci
ci:
	dune exec ci/nodo_ci.exe

.PHONY: docker-build
docker-build:
	docker build .
