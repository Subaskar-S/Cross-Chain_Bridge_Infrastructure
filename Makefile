.PHONY: help install build test clean dev deploy docs

# Default target
help:
	@echo "Cross-Chain Bridge Development Commands"
	@echo "======================================"
	@echo "install    - Install all dependencies"
	@echo "build      - Build all components"
	@echo "test       - Run all tests"
	@echo "clean      - Clean build artifacts"
	@echo "dev        - Start development environment"
	@echo "deploy     - Deploy contracts and services"
	@echo "docs       - Generate documentation"
	@echo "lint       - Run linting and formatting"

# Install dependencies
install:
	@echo "Installing Rust dependencies..."
	cargo fetch
	@echo "Installing Node.js dependencies for contracts..."
	cd contracts/ethereum && npm install
	@echo "Installing Foundry..."
	curl -L https://foundry.paradigm.xyz | bash || echo "Foundry already installed"
	@echo "Installing Substrate dependencies..."
	rustup target add wasm32-unknown-unknown

# Build all components
build:
	@echo "Building Rust workspace..."
	cargo build --release
	@echo "Building Ethereum contracts..."
	cd contracts/ethereum && forge build
	@echo "Building Substrate contracts..."
	cd contracts/substrate && cargo build --release

# Run tests
test:
	@echo "Running Rust tests..."
	cargo test
	@echo "Running Ethereum contract tests..."
	cd contracts/ethereum && forge test
	@echo "Running Substrate tests..."
	cd contracts/substrate && cargo test
	@echo "Running integration tests..."
	cd tests && cargo test

# Clean build artifacts
clean:
	cargo clean
	cd contracts/ethereum && forge clean
	cd contracts/substrate && cargo clean

# Start development environment
dev:
	@echo "Starting local blockchain networks..."
	docker-compose up -d
	@echo "Starting relayer service..."
	cargo run --bin relayer &
	@echo "Starting API server..."
	cargo run --bin api &
	@echo "Development environment started!"

# Deploy contracts and services
deploy:
	@echo "Deploying Ethereum contracts..."
	cd contracts/ethereum && forge script script/Deploy.s.sol --broadcast
	@echo "Deploying Substrate contracts..."
	cd contracts/substrate && cargo contract build
	@echo "Starting production services..."
	docker-compose -f docker-compose.prod.yml up -d

# Generate documentation
docs:
	cargo doc --no-deps --open
	cd contracts/ethereum && forge doc

# Linting and formatting
lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
	cd contracts/ethereum && forge fmt --check
