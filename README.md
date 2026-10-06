# Solana Record Service

Solana program and clients for publishing on-chain records under named classes, with optional Token-2022 tokenization.

## Overview

An authority creates a **Class**: a named namespace with free-form metadata. A class can be **permissioned**, in which case every new record needs the class authority's signature and the class authority can act on any record in it, and it can be **frozen**, which stops new records from being created.

A **Record** lives under a class at an address derived from a caller-chosen seed. It has an owner, an expiry and a data payload. The owner can transfer or delete it; in a permissioned class the class authority can too. Only the class authority can rewrite a record's data or expiry, or freeze it. A frozen record cannot be transferred and its expiry cannot change. The program stores the expiry but does not enforce it.

A record whose data is Token-2022 metadata (`name`, `symbol`, `uri` and additional key/value pairs, as written by `createRecordTokenizable`) can be **tokenized**. `MintTokenizedRecord` creates a Token-2022 mint for the record with that metadata, adds it to a group mint shared by the class, and mints one token to the owner. From then on the record's owner is the mint, and whoever holds the token controls the record. The mint is its own permanent delegate, mint authority, freeze authority and close authority, so the class authority can freeze the token and the program can move or burn it. `BurnTokenizedRecord` burns the token, closes the mint and hands the record back to the holder.

This repository contains:

- A Rust Solana program built with [Pinocchio](https://github.com/anza-xyz/pinocchio)
- IDL and client generation via [Codama](https://github.com/codama-idl/codama)
- A TypeScript client (`@solana/record`) in `clients/typescript`
- A Rust client (`solana-record`) in `clients/rust`

## Program ID

```
srsWjm76StJucL7atFyPSdXFaVLNPFqEt1uFEDPrZsn
```

Print it from source at any time with `just program-id`.

## Accounts

| Account  | Address                                  | Contents                                                            |
| -------- | ---------------------------------------- | ------------------------------------------------------------------- |
| `Class`  | PDA `["class", authority, name]`         | Authority, permissioned and frozen flags, name, metadata            |
| `Record` | PDA `["record", class, seed]`            | Class, owner type, owner, frozen flag, expiry, seed, data           |
| Mint     | PDA `["mint", record]`, Token-2022 owned | The token of a tokenized record                                     |
| Group    | PDA `["group", class]`, Token-2022 owned | Group mint of the class, created by the first `MintTokenizedRecord` |

## Instructions

| Instruction               | Signer                                          | Purpose                                           |
| ------------------------- | ----------------------------------------------- | ------------------------------------------------- |
| `CreateClass`             | Class authority                                 | Create a class                                    |
| `UpdateClassMetadata`     | Class authority                                 | Replace the class metadata                        |
| `UpdateClassAuthority`    | Class authority                                 | Hand the class to a new authority                 |
| `FreezeClass`             | Class authority                                 | Freeze or thaw the class                          |
| `CreateRecord`            | Owner, plus the class authority if permissioned | Create a record                                   |
| `UpdateRecord`            | Class authority                                 | Replace the record data                           |
| `UpdateRecordExpiry`      | Class authority                                 | Set the record expiry                             |
| `TransferRecord`          | Owner, or the class authority if permissioned   | Give the record a new owner                       |
| `DeleteRecord`            | Owner, or the class authority if permissioned   | Close the record and refund its rent              |
| `FreezeRecord`            | Class authority                                 | Freeze or thaw the record                         |
| `MintTokenizedRecord`     | Owner, or the class authority if permissioned   | Mint the record as a Token-2022 token             |
| `FreezeTokenizedRecord`   | Class authority                                 | Freeze or thaw the token                          |
| `TransferTokenizedRecord` | Holder, or the class authority if permissioned  | Move the token to another token account           |
| `BurnTokenizedRecord`     | Holder, or the class authority if permissioned  | Burn the token, close the mint, return the record |

The clients also expose `createRecordTokenizable` and `updateRecordTokenizable`. They are `CreateRecord` and `UpdateRecord` with the data typed as Token-2022 metadata, so the record can be minted later.

## Project structure

```text
solana-record-service/
├── program/                 # Rust Solana program (Pinocchio)
│   ├── build.rs             # Writes the IDL when GENERATE_IDL is set
│   └── src/
│       ├── instructions/    # Instruction handlers and the Codama instruction enum
│       ├── state/           # Class and Record accounts, Token-2022 metadata types
│       ├── token2022/       # Token-2022 account readers and extension CPIs
│       └── lib.rs           # Entrypoint and discriminator routing
├── idl/                     # Codama IDL generated from the program (committed)
├── clients/
│   ├── typescript/          # @solana/record, built on @solana/kit
│   └── rust/                # solana-record
├── integration_tests/       # litesvm integration tests
├── scripts/                 # Client generation
├── .githooks/               # pre-push: format and lint checks
└── justfile                 # Task runner
```

## Quick start

```bash
git clone git@github.com:solana-foundation/solana-record-service.git
cd solana-record-service
just setup
just build
just test
```

### Prerequisites

`just setup` checks for `pnpm`, `cargo`, and `cargo-build-sbf`.

| Tool       | Install                                                                                                |
| ---------- | ------------------------------------------------------------------------------------------------------ |
| Rust       | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh`                                      |
| Solana CLI | `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"`                                        |
| pnpm       | `curl -fsSL https://get.pnpm.io/install.sh \| sh -`                                                    |
| Just       | `curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh \| bash -s -- --to ~/.local/bin` |

Rust is pinned in `rust-toolchain.toml`, Node.js in `.nvmrc`, and pnpm in the `packageManager` field of `package.json`.

## Build and test

`just --list` shows every recipe.

| Recipe                  | Description                                            |
| ----------------------- | ------------------------------------------------------ |
| `just build`            | Program `.so` plus the TypeScript client               |
| `just build-program`    | Compile the SBF program                                |
| `just generate-idl`     | Regenerate `idl/solana_record_service.json` via Codama |
| `just generate-clients` | Regenerate both clients from the IDL                   |
| `just test`             | Rust unit and integration tests                        |
| `just integration-test` | Rust integration tests against the built `.so`         |
| `just check`            | Format and lint checks, run by the pre-push hook       |
| `just check-generated`  | Fail if the committed IDL is out of date               |

Generated client sources under `clients/*/src/generated/` are not committed. They are produced from the IDL by `just generate-clients` and bundled into the published packages.

## Clients

TypeScript:

```bash
pnpm add @solana/record
```

```typescript
import { findClassPda, findRecordPda, getCreateRecordInstructionAsync } from '@solana/record';
```

The package exports the Codama-generated instruction builders, account decoders and PDA finders (`findClassPda`, `findRecordPda`, `findMintPda`, `findGroupPda`). It is built on `@solana/kit` v8, declared as a peer dependency.

Rust:

```bash
cargo add solana-record
```

```rust
use solana_record::instructions::*;
```

## CI

| Workflow       | Description                                                    |
| -------------- | -------------------------------------------------------------- |
| **Build**      | Compile the program and the TypeScript client                  |
| **Test**       | Rust unit and Rust integration tests                           |
| **Format**     | Rust and TypeScript formatting                                 |
| **Lint**       | Clippy and oxlint                                              |
| **IDL Check**  | Fail on drift between the program and the committed IDL        |
| **Security**   | `cargo audit` and `pnpm audit`                                 |
| **PR hygiene** | Linked issue, commit signatures, AI disclosure and attribution |

Publishing is manual. `Publish Rust Client` and `Publish TypeScript Client` are `workflow_dispatch` only, gated on a green test run and a branch guard, default to a dry run, and authenticate to the registries over OIDC. `Release` builds the program with `solana-verify` and upgrades devnet directly, or exports a single Squads transaction (verify PDA, IDL metadata and program upgrade) for mainnet.

## Security

Report vulnerabilities privately through the process in [SECURITY.md](SECURITY.md), not in a public issue.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. Changelogs are kept per artifact: the [program](CHANGELOG.md), the [Rust client](clients/rust/CHANGELOG.md), and the [TypeScript client](clients/typescript/CHANGELOG.md).

## License

[MIT](LICENSE).
