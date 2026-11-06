# Audit Status

Last updated: 2026-11-06

## Current Baseline

- Auditor: OtterSec
- Report: `audits/2026-ottersec-solana-foundation-solana-record-service-audit.pdf`
- Branch: `release/program-v1`
- Audited-through commit: `ee7b2148c818ec28144f3caf29fec9a351ff8e82`
- Compare unaudited delta on the release branch: https://github.com/solana-foundation/solana-record-service/compare/ee7b2148c818ec28144f3caf29fec9a351ff8e82...release/program-v1

The audit was performed against commits `32a5654` and `ee7b214` on `release/program-v1`, which sit on top of the `program-v1.0.0` tag (`90d8c5b`). No findings were reported.

## Branch and Release Model

- `release/program-v1` carries the deployed v1 program and its hotfixes. Commits after the audited-through SHA on this branch are unaudited until a new audit or mitigation review updates this file.
- `master` diverged from `release/program-v1` at `90d8c5b` and is not covered by this audit.
- Program releases are immutable tags (for example `program-v1.0.0`).

## Verification Commands

```bash
# Commits on the release branch after the audited baseline
git log --oneline ee7b2148c818ec28144f3caf29fec9a351ff8e82..origin/release/program-v1

# Program source changed on the release branch since the audited baseline
git diff --name-status ee7b2148c818ec28144f3caf29fec9a351ff8e82..origin/release/program-v1 -- program
```

## Maintenance Rules

When a new audit is completed:

1. Add the new report to `audits/`.
2. Update the branch, audited-through commit and compare link.
3. Tag audited release commit(s) (for example `program-vX.Y.Z`).
4. Update the README Security section if needed.
