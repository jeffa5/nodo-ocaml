.PHONY: build watch

build:
	cargo build

watch:
	cargo watch -x clippy -x test

install:
	cargo build --release
	mv target/release/nodo ~/bin/nodo
