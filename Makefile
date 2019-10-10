.PHONY: build
build:
	cargo build

.PHONY: watch
watch:
	cargo watch -x clippy -x test

.PHONY: install
install:
	cargo install --path . --force
