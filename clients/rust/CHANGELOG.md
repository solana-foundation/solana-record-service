# Changelog: `solana-record-service-client` (Rust client)

Rust SDK for the Solana Record Service program. Published to crates.io; tagged `rust-client-vX.Y.Z`.

Versioning follows [Semantic Versioning](https://semver.org/).

Releases before 2.0.0 predate this changelog; see the git history and [crates.io](https://crates.io/crates/solana-record-service-client) for that period.

## [Unreleased]

## [2.0.0]

_Supersedes 0.1.0. The crate is renumbered so the program, the Rust client and the TypeScript client share one version number._

### Changed

- **Breaking:** generated against the Solana 3.x component crates instead of `solana-program`. Public keys are `solana_address::Address` rather than `Pubkey`.
- **Breaking:** u8-prefixed and trailing fields use the `spl-collections` types (`U8PrefixedStr`, `U8PrefixedVec`, `TrailingStr`, `TrailingVec`) instead of `kaigan`.
- **Breaking:** `Record::owner_type` is the `OwnerType` enum instead of a `u8`.
- The crate version is inherited from the workspace, so a single edit in the root `Cargo.toml` bumps the program and the client together.
- Generated sources under `src/generated/` are no longer committed. They are produced by `just generate-clients` and shipped to crates.io through an explicit `include` in `Cargo.toml`.

### Added

- Publishing is manual (`workflow_dispatch`), gated on a green test run, a branch guard and a dry-run default. It authenticates to crates.io over OIDC instead of a long-lived token, and pushes a git tag and a GitHub Release.
- `fetch` feature: `fetch_class`, `fetch_record` and their `fetch_all_*` variants over `solana-rpc-client`.
- `Class::find_pda` / `create_pda` and `Record::find_pda` / `create_pda` derive the class and record addresses.
