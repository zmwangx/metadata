.PHONY: default release test man clean

default: man
	cargo fmt && cargo build

release: man
	cargo build --release
	cargo test --release
	$(eval VERSION := $(shell grep '^version = ' Cargo.toml | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+'))
	mkdir -p dist/v${VERSION}
	cp target/release/metadata dist/v${VERSION}
	cp man/metadata.1 dist/v${VERSION}

test:
	cargo test

man: man/metadata.1

man/metadata.1: man/metadata.1.adoc
	a2x --doctype manpage --format manpage $^

clean:
	cargo clean