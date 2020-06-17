LIBAVCODEC_VERSION=$(shell pkg-config --modversion libavcodec)
$(info detected lavc $(LIBAVCODEC_VERSION))
LIBAVCODEC_VERSION_MAJOR := $(word 1,$(subst ., ,$(LIBAVCODEC_VERSION)))
LIBAVCODEC_VERSION_MINOR := $(word 2,$(subst ., ,$(LIBAVCODEC_VERSION)))
ifeq ($(LIBAVCODEC_VERSION_MAJOR),)
  $(error cannot determine libavcodec version with pkg-config)
else ifeq ($(shell test $(LIBAVCODEC_VERSION_MAJOR) -gt 58; echo $$?),0)
  $(warning unknown libavcodec version, possibly from FFmpeg >4; use at own risk)
  FEATURES += ffmpeg43
else ifeq ($(LIBAVCODEC_VERSION_MAJOR),58)
  ifeq ($(shell test $(LIBAVCODEC_VERSION_MINOR) -ge 91; echo $$?),0)
    FEATURES += ffmpeg43
  else ifeq ($(shell test $(LIBAVCODEC_VERSION_MINOR) -ge 54; echo $$?),0)
    FEATURES += ffmpeg42
  else ifeq ($(shell test $(LIBAVCODEC_VERSION_MINOR) -ge 35; echo $$?),0)
    FEATURES += ffmpeg41
  else
    FEATURES += ffmpeg4
  endif
endif

CARGO_BUILD_FLAGS += --no-default-features
ifneq ($(FEATURES),)
  $(info enabled features: $(FEATURES))
  CARGO_BUILD_FLAGS += --features "$(FEATURES)"
endif

.PHONY: default release test man clean

default: man
	cargo fmt && cargo build $(CARGO_BUILD_FLAGS)

release: man
	cargo build $(CARGO_BUILD_FLAGS) --release
	cargo test $(CARGO_BUILD_FLAGS) --release
	strip target/release/metadata
	$(eval VERSION := $(shell grep -m 1 '^version = ' Cargo.toml | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+'))
	mkdir -p dist/v$(VERSION)
	cp target/release/metadata dist/v$(VERSION)
	cp man/metadata.1 dist/v$(VERSION)

test:
	cargo test $(CARGO_BUILD_FLAGS)

man: man/metadata.1

man/metadata.1: man/metadata.1.adoc
	a2x --doctype manpage --format manpage $^

clean:
	cargo clean
