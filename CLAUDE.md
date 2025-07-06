# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an interactive Fedimint wallet implementation in Rust that provides basic Lightning payment functionality. The wallet runs as a persistent interactive application, allowing operations to complete properly. It connects to a Fedimint federation and allows users to:
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

### Running the Wallet
```bash
# Set federation invite code (for fish shell, use set -x)
export FEDIMINT_INVITE_CODE="fed1..."

# Run the interactive wallet
cargo run

# Or provide invite code as argument
cargo run -- --invite-code "fed1..."
```

### Interactive Menu Options
1. **Get Wallet Balance** - Display current balance in millisatoshis and sats
2. **Create a Lightning Invoice** - Generate a new invoice with specified amount and description
3. **Pay a Lightning Invoice** - Pay a provided Lightning invoice
4. **Await Invoice Payment** - Monitor an invoice payment status using operation ID
5. **Exit** - Close the wallet application

## Architecture

### Core Components

1. **main.rs**: Interactive CLI interface using clap for initial configuration, provides menu-driven wallet operations
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