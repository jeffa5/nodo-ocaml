.PHONY: build
build:
	dune build

.PHONY: watch
watch:
	dune build --watch

.PHONY: check
check:
	dune build @check

.PHONY: test
test:
	dune runtest

.PHONY: promote
promote:
	dune promote
