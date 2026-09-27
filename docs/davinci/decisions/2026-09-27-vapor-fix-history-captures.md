# Complete Vapor fix-history capture

Tracks [#6880](https://github.com/ubugeeei-prod/vize/issues/6880), after the
[SSR cohort](./2026-09-27-ssr-fix-history-captures.md).

Nine exact authored inputs link seven fixes: destructured aliases #445, keys
#428, object index aliases, nested delegated events #6203 (three cases), named
slot fallback #1235, nested outlet #1262 and dynamic directive payload #1238.
Rust witness inputs and the two Pkl inputs are copied byte-for-byte. Delegated
if/loop/fallback cases preserve prefix=true; all other cases preserve default
`VaporCompilerOptions`. `tests/vize_test_runner::compile_vapor_template` uses
those actual defaults for Pkl inputs, distinct from its VDOM prefix=true path.

The Pkl fixtures already have full code references in
`tests/expected/vapor/v-for.snap`. Their runner normalizes CRLF, indentation,
blank lines, semicolons, quote style, multiline expressions and Vue import
grouping/helper order before comparison. Preserve those useful comparisons;
they cannot protect every raw module byte. These captures add the raw output
boundary without changing or replacing existing normalized, substring or
mounted-behavior assertions.

The test-only observer calls the original `compile_vapor` public entrypoint.
Its complete result has four fields, all preserved through an exhaustive
pattern: raw code (including imports), ordered template strings, nullable raw
source-map string, and ordered public error messages. This entrypoint exposes
messages rather than structured diagnostics; no code/location payload is
invented or borrowed from another entrypoint. Every compiler and experimental
option field is read from the actual values with exhaustive patterns. Standard
syntax/default matcher/no SFC scope belong to the original wrapper contract.

## Actual local evidence

Tracked-clean committed source `941ff5b2d` used the same exclusive target
owned by the compiler-history task; no other agent's target was touched:

```sh
CARGO_TARGET_DIR=/tmp/vize-ssr-fix-history-target-20260927 \
  cargo build --locked --profile ci -p vize_atelier_vapor \
  --example vapor_fixture_observer --message-format=json-render-diagnostics
```

The narrow build succeeded in 11.47 seconds. The actual Cargo-selected example
uses opt-level 0, no debug info and test=false. It was copied off target before
execution. Binary SHA256:
`092af26624bd0a41d224157b45595b8966daa38c13aae5c77ab709fd6798f785`.
Its receipt binds clean source commit/tree, exact recipe, lock/toolchain,
selected Cargo event, successful build and raw logs. Both executions retain
actual stdout/stderr/status and binary/output hashes.

All nine complete results repeat equally. Template arrays contain actual
ordered values (two for aliases, one for the other eight). All maps are null
and public error arrays empty, so neither populated maps nor error-message
production is certified by this workload. Per-case expected JSON views are
value-identical copies of the measured outputs/options; raw first/repeat
captures remain unchanged.

The ordinary Rust test compares every complete result and option value with
the immutable capture. Only JSON object-key order is immaterial. Strings,
terminal whitespace, CRLF, null/presence, scalar types and array order remain
exact. Inputs use `.input.txt` and are explicitly compiled; no implicit global
Vue-corpus membership or L2/L3 baseline is changed.

The reviewed frozen observer source remains reachable at
[`provenance/vapor-history-capture-941ff5b2d`](https://github.com/ubugeeei-prod/vize/tree/provenance/vapor-history-capture-941ff5b2d),
pointing exactly to the receipt's source SHA. This preserves historical local
source identity separately from fresh publication-head Actions evidence.
Original raw Cargo/process/output archives remain byte-identical after replay.

## Remaining gates

The nine-case ordinary product test, strict example/test Clippy, formatting,
source-length ratchet and all 29 linked archive hashes pass locally. Fresh
Actions must pass before merge.
This is local macOS arm64 profile-ci evidence, not Actions or a native route
certificate. Public selected output is observed without claiming zero fallback.
Native acceptance remains zero; shared compiler adapter registration and full
history/target/dialect review remain unfinished. Continue raw hydration,
KeepAlive and structural-slot captures while preserving their runtime traces.
#6880 stays open, with its historical 609-fix denominator unreconciled.

Publication uses ordinary module discovery for both the integration target and
Cargo's automatically discovered example directory. A separate test-only copy
of the observer helper has identical logic after input include-path adjustment;
this avoids a new dependency crate or path-attribute bypass. The example entry
moves in a move-only commit, then wiring changes follow. The generator updates
only the Vapor consumer shard. Archive bytes and all 29 indexed hashes remain
unchanged; fresh latest-head Actions validate the adjusted source.
