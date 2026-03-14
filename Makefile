.PHONY: build build-debug test lint bench clean install serve

CARGO_FLAGS ?= -j 1

build:
	cargo build --release $(CARGO_FLAGS)

build-debug:
	cargo build $(CARGO_FLAGS)

test:
	cargo test $(CARGO_FLAGS)

lint:
	cargo fmt -- --check
	cargo clippy $(CARGO_FLAGS) -- -D warnings

bench:
	cargo bench $(CARGO_FLAGS)

clean:
	cargo clean

install:
	cargo install --path crates/agentic-workflow-cli $(CARGO_FLAGS)
	cargo install --path crates/agentic-workflow-mcp $(CARGO_FLAGS)

serve:
	cargo run --bin agentic-workflow-mcp $(CARGO_FLAGS)
