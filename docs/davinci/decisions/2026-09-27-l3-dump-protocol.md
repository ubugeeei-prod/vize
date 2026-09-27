# Canonical L3 graph protocol (#6832, partial slice)

The level crates have no compatibility users according to AGENTS.md. Current
L3 APIs therefore expose only sixteen `l3.*` operation mnemonics; no old opcode
aliases or compatibility decoder are added to the level implementation.

The graph document explicitly identifies its new grammar with `[l3-dump-v2]`.
Its regions/ops/edges/effects sections use that same versioned header. Records,
variant discriminants, IDs, phases, effects, spans and lowering algorithms retain
their shape and order. The current Rust and Lean readers require the new grammar;
old headers and opcode vocabularies are negative test inputs only.

A deterministic bounded script updates thirty-eight active consumers, including six
snapshots and twelve formal reference fixture files. Its inverse recovers the
exact frozen parent bytes before formatting; four source/docs files also have
layout-only formatter output. Those fixtures are current executable reference
inputs, separate from historical raw compiler/profile captures. Immutable f59,
e28 and ede1 captures, source hashes, options, receipts and logs stay unchanged.

Level dump parse-error labels use L3. Legacy product emitted code and diagnostics
are unchanged by the implementation; actual differential Actions still verify
that boundary. Local source proof does not establish compiler/runtime parity.

Cargo features, counters and current-source strict profiling move in a separate
slice with authenticated v2 observer recipes. Formal namespaces/paths, other
concern-page headers, feed negotiation, cache hash domains and the shared actual
production-stage generator remain outstanding. #6832 stays open; Stage1 does
not begin. No extra compiler stage, serialization between levels, or budget
relaxation is introduced.

## Current source replay evidence

The prepared source follows clean parent `5bdaa46ec`. All 43-file migration
hunks outside the central record have the same stable patch id as the
original (`4e856be2ebd1a1eb7e9e7375f3aed6e863e9c26a`); the central record
retains registry, Dump and CLI decisions. The script accepts this parent via
`--base-ref` and verifies all 38 transformed consumers, including inverse
byte identity and layout-only formatting. Forty-two existing Node contracts
pass at this composition, alongside Cargo format, whitespace and changed
350-line caps. The historical diagnostic fixture is reused explicitly; no
new CLI, Rust, Lean, WASM or Actions execution is claimed.

The decisions and remaining coupling are mirrored in the
[#6832 record](https://github.com/ubugeeei-prod/vize/issues/6832#issuecomment-5854177277).

## Published protocol assertion repair

The published graph slice in [#6945](https://github.com/ubugeeei-prod/vize/pull/6945)
contributes one assertion-lint failure observed in the descendant #6947 Actions
tooling job: `dump_protocol.rs:28` compares only the `[l3-dump-v2]` prefix.
The repair compares the entire normalized printed graph with an explicit
`CANONICAL_GRAPH` constant while retaining the structural roundtrip and every
old-header/unknown-opcode rejection control. No allowlist entry is added.

The original input `GRAPH` remains byte exact. It includes empty edge/effect
sections that the derived printer omits; it is accepted input rather than
the normalized full-output oracle. Source review of Page field declaration
order, Region/Op value printers and `PagePrinter::list` establishes the exact
expected rows and blank lines without changing the printer or protocol.

The existing wire replay script also preflights the bounded assertion repair
against published parent `ab21c5ceb`. It proves the source inverse before any
write and supports `--assertions-only --verify` independently of the earlier
38-consumer wire migration. Parent owns the central record/issue update and
publication. TODO: rerun the Rust protocol test and required Actions gates at
the restacked immutable head; source/lint evidence is not Rust execution.

Preparation verification passes all eight protocol/assertion-lint Node
contracts, including the actual standalone Rust assertion linter over this
published-parent tree and its bad-fixture/expiry/path controls. The invocation
adds `/Users/ubugeeei/.cargo/bin` and `/opt/homebrew/bin` to PATH so spawned
`rust-script` and Node resolve. The linter allowlist stays byte exact.
The bounded assertion replay and repeated replay pass; the full wire verifier
also passes all 38 consumers with `--base-ref 5bdaa46ec` and explicit Node.
Standalone Rust formatting, documentation formatting, whitespace and the
350-line source policy pass. No full Rust product build or new Actions run is
claimed. The descendant log's two Vitrine cache-prefix findings belong to
its later hash-domain layer and are absent from this parent tree.

Cross-layer replay tooling note: prepending `/opt/homebrew/bin` selects its
old `rustfmt 1.5.1-nightly`, which changes token order and fails the strict
formal source check before any current path-map comparison. The preserved
formal proof passes with explicit `--node` and `--rustfmt` paths targeting
Node 25.8.1 and the project toolchain's `rustfmt 1.9.0-stable`
(`88d9e12ae1`, 2026-08-18). Pin those tools when diagnosing later composition;
do not relax the token or frozen-byte invariants to accept a formatter mismatch.
