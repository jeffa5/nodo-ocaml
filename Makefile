.PHONY: build
build:
	cargo build

.PHONY: watch
watch:
	cargo watch -x clippy -x test

.PHONY: install
install:
	cargo build --release
	mv target/release/nodo ~/bin/nodo
