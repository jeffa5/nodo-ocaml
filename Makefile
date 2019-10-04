.PHONY: build watch

build:
	cargo build

watch:
	cargo watch -x clippy -x test
