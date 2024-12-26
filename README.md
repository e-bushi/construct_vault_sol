# Construct Vault Sol

A Solana smart contract implementation for creating time-locked token vaults for $KUZA tokens.

## Overview

The Construct Vault smart contract enables users to create secure vaults that lock their $KUZA tokens for a 30-day period. This contract implements a Program Derived Address (PDA) system to manage vault ownership and token custody.

### Key Features

- Create time-locked vaults for $KUZA tokens
- 30-day locking period
- Secure token custody through PDAs
- Associated Token Account management
- User activity tracking

## Architecture

The contract consists of three main components:

1. **The Construct** (`construct_vault_sol`): Main contract logic that handles vault creation and token locking
2. **The Vault PDA**: Stores user details, token amounts, and locking timestamps
3. **Associated Token Account**: Holds the actual locked tokens, owned by the Vault PDA

## Prerequisites

- Rust 1.83.0
- Solana Program v2.1.6
- Borsh v1.5.3
- SPL Token v7.0.0
- SPL Associated Token Account v6.0.0

## Dependencies

The project uses the following crates:
- `borsh = "1.5.3"` - Binary Object Representation Serializer for Hashing
- `solana-program = "2.1.6"` - Core Solana program crate
- `spl-associated-token-account = "6.0.0"` - SPL Associated Token Account handling
- `spl-token = "7.0.0"` - SPL Token program integration

### Dev Dependencies
- `solana-program-test = "2.1.6"`
- `solana-sdk = "2.1.6"`
- `tokio = "1.42.0"`

## Installation

```bash
git clone https://github.com/e-bushi/construct_vault_sol
cd construct_vault_sol
cargo build
```

## Building

```bash
cargo build-bpf
```

## Testing

```bash

cargo test-bpf

```

## Usage

### Program ID

After building, you can find your program ID in the target/deploy directory. You'll need this ID to interact with the program.

### Client Integration

Example of creating a vault transaction using the solana-program-test library:

```rust

let create_vault_ix = Instruction::new_with_borsh(

    program_id,

    &VaultInstruction::Initialize { amount },

    vec![

        AccountMeta::new(payer.pubkey(), true),

        AccountMeta::new(vault_pda, false),

        AccountMeta::new(token_account, false),

        AccountMeta::new_readonly(spl_token::ID, false),

        AccountMeta::new_readonly(system_program::ID, false),

    ],

);

```

### Checking Vault Status

You can fetch vault data using the Solana RPC API:

```rust

let account_data = connection

    .get_account_data(&vault_pda)

    .await?;

let vault_data = VaultState::try_from_slice(&account_data)?;

println!("Lock expires at: {}", vault_data.lock_expiry);

```

## Security Considerations

- The contract uses PDAs for secure token custody

- Implements checks for token ownership and amounts

- Validates lock duration compliance

- Ensures proper account ownership

## Contributing

1. Fork the repository

2. Create your feature branch

3. Commit your changes

4. Push to the branch

5. Create a new Pull Request

## Disclaimer

This smart contract is provided as-is. Users should conduct their own security audit before deploying in production.

## Contact

For questions and support, please open an issue in the GitHub repository.

