# Simple Fedimint Wallet

A minimal Fedimint wallet implementation with basic Lightning payment functionality. The wallet runs
as a persistent application with a menu-driven interface.

## Features

- Connect to a Fedimint federation using an invite code
- Check balance
- Create Lightning invoices
- Pay Lightning invoices
- Monitor invoice payment status

## Prerequisites

- A Fedimint federation invite code

## Installation

```bash
git clone <repo_url>
cd simple-fedimint-wallet
cargo build --release
```

## Usage

### Setting the Federation Invite Code

You can provide your federation invite code in two ways:

1. **Environment Variable** (recommended):
```bash
# For bash/zsh
export FEDIMINT_INVITE_CODE="fed1..."

# For fish shell
set -x FEDIMINT_INVITE_CODE "fed1..."
```

2. **Command Line Argument**:
```bash
cargo run -- --invite-code "fed1..."
```

### Running the Wallet

Start the interactive wallet:

```bash
cargo run
```

The wallet will display a menu with the following options:

```
Simple Fedimint Wallet
=====================

1. Get Wallet Balance
2. Create a Lightning Invoice
3. Pay a Lightning Invoice
4. Await Invoice Payment
5. Exit

Select an option:
```

### Interactive Operations

1. **Get Wallet Balance**: Displays your current balance in millisatoshis and satoshis

2. **Create a Lightning Invoice**: 
   - Enter the amount in millisatoshis
   - Optionally provide a description
   - Receive a Lightning invoice and operation ID

3. **Pay a Lightning Invoice**:
   - Paste the Lightning invoice when prompted
   - The wallet will process the payment and show the contract ID and fees

4. **Await Invoice Payment**:
   - Enter the operation ID from invoice creation
   - The wallet will wait for the invoice to be paid

5. **Exit**: Safely close the wallet application

### Data Storage

The wallet stores its data in `./wallet-data` by default. You can change this with the `--data-dir`
flag.

## Security Notes

- This is a minimal implementation for learning purposes
- The wallet data directory contains sensitive information
