# Audit Status

Last updated: 2026-11-09

## Reports

| Auditor   | Report                                                                                       | Review period            | Scope                                           | Findings                                     |
| --------- | -------------------------------------------------------------------------------------------- | ------------------------ | ----------------------------------------------- | -------------------------------------------- |
| Zenith    | [2025-zenith](2025-zenith-solana-foundation-solana-record-service-audit.pdf)                 | 2025-09-09 to 2025-09-16 | `d98ba1e` on `master`                           | 3 Critical, 1 High, 6 Medium, 12 Low, 8 Info |
| Accretion | [2026-accretion](2026-accretion-solana-foundation-solana-record-service-audit.pdf) (A25SFR3) | 2025-10-09 to 2026-02-10 | `ba0f649` on `master`                           | 1 High, 3 Medium, 4 Low, 7 Info              |
| OtterSec  | [2026-ottersec](2026-ottersec-solana-foundation-solana-record-service-audit.pdf)             | 2026-11-04               | `32a5654` and `ee7b214` on `release/program-v1` | None                                         |

## Current Baseline

- Auditor: OtterSec
- Branch: `release/program-v1`
- Audited-through commit: `ee7b2148c818ec28144f3caf29fec9a351ff8e82`
- Compare unaudited delta on the release branch: https://github.com/solana-foundation/solana-record-service/compare/ee7b2148c818ec28144f3caf29fec9a351ff8e82...release/program-v1

## Branch and Release Model

- `release/program-v1` carries the deployed v1 program and its hotfixes. Commits after the audited-through SHA on this branch are unaudited until a new audit or mitigation review updates this file.
- `master` diverged from `release/program-v1` at `90d8c5b` and is not covered by the OtterSec audit.
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

1. Add the new report to `audits/` as `<year>-<auditor>-solana-foundation-solana-record-service-audit.pdf`.
2. Add a row to the Reports table.
3. Update the branch, audited-through commit and compare link if the audit sets a new baseline.
4. Tag audited release commit(s) (for example `program-vX.Y.Z`).
5. Update the README Security section if needed.
