# Simple Fedimint Wallet

A minimal implementation of a Fedimint wallet with basic Lightning payment functionality.

## Features

- Connect to a Fedimint federation using an invite code
- Check balance
- Create Lightning invoices
- Pay Lightning invoices
- Monitor invoice payment status

## Prerequisites

- Rust 1.65 or later
- A Fedimint federation invite code

## Installation

```bash
git clone <repo>
cd simple-fedimint-wallet
cargo build --release
```

## Usage

Set your federation invite code as an environment variable or pass it as a flag:

```shell
export FEDIMINT_INVITE_CODE="fed1..."
```

### Check Balance

```shell
cargo run -- balance
```

### Create Invoice

```shell
cargo run -- invoice --amount 10000 --description "Test payment"
```

### Pay Invoice

```shell
cargo run -- pay "lnbc..."
```

### Check Payment Status

```shell
cargo run -- await-payment <operation-id>
```

### Data Storage

The wallet stores its data in `./wallet-data` by default. You can change this with the `--data-dir` flag.

## Security Notes

- This is a minimal implementation for learning purposes
- Store your invite code securely
- The wallet data directory contains sensitive information
