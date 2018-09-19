LACV_VERSION=$(shell pkg-config --modversion libavcodec)
$(info detected lavc $(LACV_VERSION))
LAVC_MAJOR_VERSION := $(firstword $(subst ., ,$(LACV_VERSION)))
ifeq ($(LAVC_MAJOR_VERSION),)
$(error cannot determine libavcodec version with pkg-config)
else ifeq ($(shell test $(LAVC_MAJOR_VERSION) -lt 58; echo $$?),0)
# Disable feature ffmpeg4
CARGO_BUILD_FLAGS += --no-default-features
endif

.PHONY: default release test man clean

default: man
	cargo fmt && cargo build $(CARGO_BUILD_FLAGS)

release: man
	cargo build $(CARGO_BUILD_FLAGS) --release
	cargo test $(CARGO_BUILD_FLAGS) --release
	$(eval VERSION := $(shell grep '^version = ' Cargo.toml | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+'))
	mkdir -p dist/v$(VERSION)
	cp target/release/metadata dist/v$(VERSION)
	cp man/metadata.1 dist/v$(VERSION)

test:
	cargo test

man: man/metadata.1

man/metadata.1: man/metadata.1.adoc
	a2x --doctype manpage --format manpage $^

clean:
	cargo clean
