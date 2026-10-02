# Changelog: Solana Record Service program

On-chain program `srsWjm76StJucL7atFyPSdXFaVLNPFqEt1uFEDPrZsn`, versioned by `program-vX.Y.Z` git tags.
SDK client changelogs are tracked separately: [`clients/typescript`](clients/typescript/CHANGELOG.md) and [`clients/rust`](clients/rust/CHANGELOG.md).

Versioning follows [Semantic Versioning](https://semver.org/).

Releases before 2.0.0 predate this changelog and were never tagged; see the git history for that period.

## [Unreleased]

## [2.0.0]

_Not yet deployed. The currently deployed binary predates this version. The program, the Rust client and the TypeScript client are unified on a single version number as of this release._

### Changed

- Built on Pinocchio 0.11 and the pinocchio-token-2022 CPI builders. The instruction and account layouts are unchanged, so existing classes and records stay readable.
- The release profile enables `overflow-checks` and fat LTO. Arithmetic that silently wrapped in release builds now aborts the instruction, and the next deployment will not reproduce the build hash of any prior deployment.

### Added

- The IDL is generated from the program source with Codama derive macros and committed under `idl/`.
- `just` is the single task runner for build, test, lint, format and client generation.
- CI gates on every pull request: build, integration and client tests, formatting, clippy, IDL drift, `cargo audit` and `pnpm audit`.
- A `Release` workflow builds the program with `solana-verify`, upgrades devnet directly, and exports a Squads transaction for mainnet.
