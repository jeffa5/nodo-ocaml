.PHONY: build
build:
	dune build

.PHONY: watch
watch:
	dune build --watch

.PHONY: check
check:
	dune build @check
