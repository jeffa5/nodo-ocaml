.PHONY: build
build:
	dune build

.PHONY: watch
watch:
	dune runtest --watch

.PHONY: check
check:
	dune build @check

.PHONY: test
test:
	dune runtest

.PHONY: format
format:
	dune build @fmt

.PHONY: promote
promote:
	dune promote

.PHONY: coverage
coverage:
	BISECT_ENABLE=yes dune runtest --force
	bisect-ppx-report html

.PHONY: completion
completion:
	cp zcomp ~/.zsh/functions/_nodo
