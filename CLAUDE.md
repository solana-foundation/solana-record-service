# CLAUDE.md

Solana program (Pinocchio) for records published under named classes, with
optional Token-2022 tokenization. Codama derive macros describe the program,
`program/build.rs` writes the IDL, and Codama renderers generate the Rust and
TypeScript clients. Build/test recipes: `just --list`. Program overview and
instruction list: [`README.md`](./README.md).

## Gotchas

**Wire format is not Anchor's.** Instruction discriminators are one byte,
routed in `program/src/lib.rs`, and account discriminators are one byte at
offset 0 (`Class` is 1, `Record` is 2). Variable-length fields are a mix of
u8-prefixed and trailing: a class name and a record seed carry a u8 length,
class metadata and record data run to the end of the buffer. Anything that
assumes Anchor or Borsh defaults will mis-decode both.

**`createRecordTokenizable` and `updateRecordTokenizable` exist only in the
clients.** They reuse discriminators 4 and 5 and retype `data` as Token-2022
`Metadata`; `scripts/generate-clients.ts` adds them to the IDL before rendering.
The program cannot tell the two encodings apart, so instruction parsers in the
clients identify discriminator 4 and 5 as the plain variants.

**The IDL is written by a build script, not a CLI.** `program/build.rs` emits
`idl/solana_record_service.json` only when `GENERATE_IDL` is set, which
`pnpm run generate-idl` does; an ordinary `cargo build` leaves the file alone.
The IDL embeds the workspace version, so bumping `version` in the root
`Cargo.toml` is not complete until `just generate-clients` has run and the IDL
diff is committed. `just check-generated` catches it, and CI fails on it.

**Codama reads the source, so annotations are the contract.** Account lists,
argument types, PDA seeds and account defaults come from `#[codama(...)]`
attributes on `instructions/mod.rs`, `state/` and `constants.rs`; the
instruction enum is never used at runtime. `type = string(utf8)` or
`type = bytes` with no `size_prefix` renders a trailing field, which is what
the program reads. `Mint` and `Group` in `constants.rs` are empty structs that
exist only to declare PDA seeds; the generate script drops accounts with no
fields so they keep their PDA helpers without getting decoders.

**The Rust client depends on `spl-collections`.** The Rust renderer maps
u8-prefixed and trailing fields to `U8PrefixedStr`, `TrailingVec` and friends
from that crate. It has no serde support, so the client has no `serde`
feature. The Rust renderer only emits PDA helpers on accounts, so the Rust
client has `Class::find_pda` and `Record::find_pda` but nothing for the mint
and group PDAs; the TypeScript client has `find*Pda` for all four.

**Generated client sources are gitignored.** `clients/*/src/generated/` is
produced by `just generate-clients`. Never hand-edit it, and never commit it.
Because `cargo package` honors `.gitignore`, `clients/rust/Cargo.toml` carries
an explicit `include` list; dropping it publishes a crate with no source.

**Integration tests need the built `.so` on disk.** `cargo test` alone does
not build it. Run `just integration-test`, which runs `cargo-build-sbf` first
and sets `SBF_OUT_DIR` to `target/sbpf-solana-solana/release`.

**`AccountView` is `Copy`.** Instruction structs hold account views by value
and `execute` takes `&mut self`. A copy points at the same runtime account,
so a lamport or data write through any copy is visible through all of them,
and the runtime borrow flags still apply.

**`#![no_std]` still links `std`.** The program's own code avoids `std`, but
the `codama` dependency is not `no_std`, so `std` ends up in the binary and
`entrypoint!` uses its panic handler. `nostd_panic_handler!` fails with a
duplicate `panic_impl`. SAS and subscriptions use the same setup.

**Rent comes from `lamports_per_byte` alone.** Pinocchio 0.11's `Rent`
ignores the exemption threshold, which matches the live clusters (threshold
1.0). Test runtimes that still report a 2.0 threshold under-fund every account
the program creates; litesvm 0.12 does not.

**Token-2022 metadata and group CPIs are hand-written.** `pinocchio-token-2022`
has no builders for `InitializeTokenMetadata`, `UpdateField`, `InitializeGroup`
or `InitializeMember`, so they live in `token2022/extensions.rs`. The metadata
bytes are passed through from the record as stored. `test_tokenization.rs`
re-parses the minted state with the SPL interface crates, so an encoding change
fails there.

**The release profile changed the binary.** `overflow-checks` and fat LTO are
on, so arithmetic that used to wrap now aborts the instruction, and the next
deployment will not reproduce any prior deployment's build hash.

**`declare_id!` in `program/src/lib.rs` is parsed twice by text.** The
`program-id` recipe seds it, and Codama only recognizes the unqualified macro
call. Keep it a single literal line.

## Conventions

- Pinocchio, never `anchor-lang`.
- Each instruction lives in `instructions/` as an accounts struct validated in
  `TryFrom<&[AccountView]>`, an instruction struct parsed in
  `TryFrom<Context>`, and `process` / `execute`.
- Ownership, signer, discriminator and PDA derivation are all checked
  explicitly; there is no framework doing it.
- Crate versions are inherited from the workspace, so one edit in the root
  `Cargo.toml` moves the program and the Rust client together. The TypeScript
  client's version lives in `clients/typescript/package.json` and must be
  bumped separately.
