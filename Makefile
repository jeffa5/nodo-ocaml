.PHONY: build
build:
	cargo build

.PHONY: watch
watch:
	cargo watch -x clippy -x test

.PHONY: test
test:
	cargo test

.PHONY: completion
completion:
	cp zcomp ~/.zsh_functions/_nodo

.PHONY: install
install: completion
	cargo install --path . --force

