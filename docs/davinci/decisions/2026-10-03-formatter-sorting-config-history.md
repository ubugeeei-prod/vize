# Import-sorting CLI configuration history (#6882 / #7258)

Extend the separate import-sorting API pack with six source-built CLI scenarios,
using the existing current-source CLI build receipt. The original 300-case audit,
five historical CLI scenarios, input/options/output pins and old receipts remain
unchanged. The consumer uses the pinned API-provider manifest and complete UserCard
input/output references. It was prepared from that provider head, then rebased onto
fresh main only after #7550 actually merged; no dependent child PR existed before
the parent merge, so this is an independent queue leaf.

## Observable configuration behavior

Plan twelve source-built subprocess calls covering JSON-enabled check/dry/write/recheck, explicit
false, invalid-config bypass with `--no-config`, invalid leading-boundary write
protection, standalone TypeScript and an effectful MJS configuration. Every call
retains complete argv, status/signal/process error, raw stdout/stderr and all
workspace file bytes/hashes before and after execution. Check/dry leave source
bytes intact; write must match the whole public API reference; recheck confirms
that actual write. The standalone input/output is the exact script-body extraction
already used in the original CLI law, not an invented whole-SFC historical input.

The MJS fixture increments its original counter, chooses sorting only on the first
evaluation and applies original nested entry/ignore controls. A supplementary
observation writes the complete returned configuration object. The expected final
counter is exactly `1`, the selected component equals the whole sorted reference,
the ignored component retains all authored bytes and every other file retains its
expected complete state. These compare actual configuration effects and the raw
fixture-returned object. They do not claim a serialized private CLI effective-option
snapshot; that API does not exist. The independent API pack retains its own real
full native resolver probes. Configuration input, return trace, API probes and CLI
output are distinct evidence.

Expected CLI stream bytes are repository-authored from the existing CLI contract,
not captured observations. The MJS entry expansion prints an absolute temporary
path; only its declared workspace identity is interpolated into the full expected
stream. Actual stderr is retained without normalization, stripping or truncation.
Current Rust CLI source laws and all input/config/output/stream assets are pinned.
Unknown references, duplicate or omitted cases/steps, broken file-state chains,
invented native credit and altered full stream/file hashes fail closed. Store the
complete process frame before a fallible output-file snapshot. A broken snapshot
retains its typed error on that last call and cannot earn matched credit; cleanup
errors also retain the raw calls and fail the row.

## Qualified provider outcome

The API-provider PR #7550 actually merged at 2026-10-03T11:19:46Z as
`e0a3b75670e05f5d81a73eb8b4bd55de0c3af774`. Published-source Check
`37117512013` and protected candidate Check `37118060547` succeeded. The protected
planner ran the full 710/710 tooling inventory with zero deferred; instruction logs
measured all 100 registered ceilings three times without granting Glyph-specific
metric credit. Downloaded artifact `11273180157` admits all original 300 rows and
the separate 14-case sorting report (11 byte matches, three real typed errors, zero
failures), including 50 option/process calls. Its complete report SHA-256 is
`739a659616e891efb029a89d86313be6b6904bf6f0fac776ecd0243e0fa6a197`; the frozen
Linux observer SHA-256 is
`872a68f2dd32490a3c947520beec908e67f5c8e7a9b4efe60f8c413af8c8ee31`.
Normal report validators admitted all complete bytes and source/Cargo/frozen-artifact
hashes bind to the actual merge commit. Download validation does not re-execute the
Linux binary on macOS and grants no execution credit to this new CLI slice.

## Hosted execution

The existing mandatory T1 formatter execution appends this pack, checks the real
current-source CLI receipt and writes `import-sorting-config-report.json` through
the existing Actions evidence upload. Pure contract/selection checks stay in T0.
Four local admission tests pass, using explicitly synthetic rows only for negative
controls. No local real CLI capture is claimed: its attempted CI-profile build
failed with `No space left on device`, and stopped private Cargo caches were cleaned
while all earlier frozen observers, receipts and raw reports were preserved. Further
Rust validation runs in hosted Actions. Exact-head Actions, actual composed CLI
execution, native Stack registration when applicable and literal merge are pending.

## Remaining gates

This six-scenario slice does not close all #7258 configuration/schema/binding/Vite
obligations or the complete formatter-history gate. Additional malformed-key
variants, native/WASM and Vite+ controls, applicable Glyph instruction measurements
and the real OXC printer-error runtime arm remain unfinished. The pinned printer
currently reports internal invalid-document tag invariants; no genuine authored-input
printer-error witness has been established and no synthetic failure is substituted.
#6882 stays open and native handled/equivalent/paired counts remain zero.
