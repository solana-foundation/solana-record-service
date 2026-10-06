# Changelog: `@solana/record` (TypeScript client)

TypeScript SDK for the Solana Record Service program. Published to npm; tagged `ts-client-vX.Y.Z`.

Versioning follows [Semantic Versioning](https://semver.org/).

Releases before 2.0.0 were published as `srs-lib` and predate this changelog; see the git history and [npm](https://www.npmjs.com/package/srs-lib) for that period.

## [Unreleased]

### Changed

- The package is renamed from `srs-lib` to `@solana/record`. `srs-lib` receives no further releases.

## [2.0.0]

_Supersedes 1.0.0. The package is renumbered so the program, the Rust client and the TypeScript client share one version number._

### Changed

- **Breaking:** generated for `@solana/kit` 8 instead of Umi. `@solana/kit` is a peer dependency, and `@metaplex-foundation/umi`, `bn.js`, `borsh` and `borsher` are no longer dependencies.
- Ships ESM and CJS builds with `accounts` and `instructions` subpath exports, and is marked side-effect free.
- Generated sources under `src/generated/` are no longer committed. They are produced by `just generate-clients` at build and publish time.

### Added

- Publishing is manual (`workflow_dispatch`), gated on a green test run, a branch guard and a dry-run default. It authenticates to npm over OIDC instead of a long-lived token, and pushes a git tag and a GitHub Release.
- PDA helpers `findClassPda`, `findRecordPda`, `findMintPda` and `findGroupPda`, and PDA defaults for the class, record, mint and group accounts in the async instruction builders.
