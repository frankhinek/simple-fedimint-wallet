# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a minimal Fedimint wallet implementation in Rust that provides basic Lightning payment functionality. The wallet connects to a Fedimint federation and allows users to:
- Check balances
- Create Lightning invoices
- Pay Lightning invoices
- Monitor payment status

## Key Commands

### Build and Development
```bash
# Build the project
cargo build --release

# Run in development mode
cargo run -- <command>

# Check code for errors without building
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

### Wallet Commands
```bash
# Set federation invite code
export FEDIMINT_INVITE_CODE="fed1..."

# Check balance
cargo run -- balance

# Create invoice (amount in millisatoshis)
cargo run -- invoice --amount 10000 --description "Payment"

# Pay invoice
cargo run -- pay "lnbc..."

# Check payment status
cargo run -- await-payment <operation-id>
```

## Architecture

### Core Components

1. **main.rs**: CLI interface using clap, handles command parsing and orchestrates wallet operations
2. **wallet.rs**: Core wallet implementation that:
   - Manages Fedimint client initialization and recovery
   - Handles Lightning invoice creation/payment through the Lightning module
   - Manages gateway selection for invoice routing
   - Provides operation tracking for async payment flows

### Key Dependencies

- **fedimint-client**: Core client functionality for federation interaction
- **fedimint-ln-client**: Lightning network integration module
- **fedimint-mint-client**: E-cash mint operations
- **fedimint-rocksdb**: Persistent storage backend
- **tokio**: Async runtime for handling concurrent operations

### Data Flow

1. Client connects to federation using invite code
2. Wallet data persists in `./wallet-data` (configurable via --data-dir)
3. Lightning operations go through federation gateways
4. All operations return operation IDs for async tracking

### Error Handling

The wallet uses a custom `WalletError` enum for domain-specific errors:
- `InvoiceAmountZero`: Prevents creation of zero-amount invoices
- `InvoiceCanceled`: Handles canceled invoice scenarios
- `InsufficientBalance`: Pre-flight balance checks for payments

## Development Environment

### Nix Setup
The project includes a Nix flake for reproducible development environments:
```bash
# Enter development shell
nix develop

# Includes: rust toolchain, cargo tools, openssl, pkg-config
```

### Logging
Set log level via environment variable:
```bash
RUST_LOG=debug cargo run -- balance
```