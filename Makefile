.PHONY: default release test man clean

default: man
	cargo fmt && cargo build

release: man
	cargo build --release
	cargo test --release
	strip target/release/metadata
	$(eval VERSION := $(shell grep -m 1 '^version = ' Cargo.toml | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+'))
	mkdir -p dist/v$(VERSION)
	cp target/release/metadata dist/v$(VERSION)
	cp man/metadata.1 dist/v$(VERSION)

test:
	cargo test

lint:
	cargo clippy -- -D warnings
	cargo fmt -- --check

man: man/metadata.1

man/metadata.1: man/metadata.1.adoc
	a2x --doctype manpage --format manpage $^

clean:
	cargo clean
