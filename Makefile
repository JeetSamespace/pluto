
# Define variables for binaries
GATEWAY_BINARY = pluto-gateway
ORBIT_BINARY = pluto-orbit

# Default target: build both binaries
all: build

# Build both binaries
build:
	cargo build --bin $(GATEWAY_BINARY) --release
	cargo build --bin $(ORBIT_BINARY) --release

# Run pluto-gateway
run-gateway:
	cargo run --bin $(GATEWAY_BINARY)

# Run pluto-orbit
run-orbit:
	cargo run --bin $(ORBIT_BINARY)

# Run tests
test:
	cargo test

# Run tests with detailed output
test-verbose:
	cargo test -- --nocapture

# Format the code using rustfmt
fmt:
	cargo fmt

# Check code formatting
check-fmt:
	cargo fmt -- --check

# Lint the code using clippy
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean the build artifacts
clean:
	cargo clean

# Build release versions
release:
	cargo build --release --bin $(GATEWAY_BINARY)
	cargo build --release --bin $(ORBIT_BINARY)

# Phony targets are not real files
.PHONY: all build run-gateway run-orbit test fmt check-fmt lint clean release