.PHONY: build run fmt lint test docker e2e e2e-init e2e-all clean

build:
	cargo build --release

run:
	cargo run -- --config config/example.yaml

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

test:
	cargo test

docker:
	docker buildx build --platform linux/amd64,linux/arm64 -t bus-exporter .

e2e:  ## Run native E2E tests (Rust-based Modbus simulator, no Docker needed)
	cargo test --test e2e_modbus -- --nocapture

OTELCOL_VERSION ?= 0.120.0

e2e-init:  ## Install otelcol-contrib for OTLP e2e tests
	@echo "Installing otelcol-contrib v$(OTELCOL_VERSION)..."
	@ARCH=$$(uname -m | sed 's/x86_64/amd64/' | sed 's/aarch64/arm64/'); \
	curl -fsSL "https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download/v$(OTELCOL_VERSION)/otelcol-contrib_$(OTELCOL_VERSION)_linux_$${ARCH}.tar.gz" \
		| sudo tar xz -C /usr/local/bin otelcol-contrib
	@otelcol-contrib --version
	@echo "Done. Run 'make e2e-all' to execute all e2e tests."

e2e-all:  ## Run all E2E tests including ignored ones (requires otelcol-contrib — install via 'make e2e-init')
	cargo test --test 'e2e_*' -- --nocapture --include-ignored

clean:
	cargo clean
