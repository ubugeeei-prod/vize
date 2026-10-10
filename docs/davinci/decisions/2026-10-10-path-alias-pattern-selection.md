# Select a path alias before probing targets

Issue: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).
Base: actual signed main `e5702be6d0119a716499827fa7b918ce8c9dcbae`.

The Canon dependency resolver mixed pattern ranking with successful file
probing. A wildcard could outrank an exact key or win its score tie; a missing
preferred target could fall through to a different, less-specific pattern.
The editor and lowered JSX projections then rewrote the import to that wrong
physical module. Ordinary CLI input collection can mask the registration bug,
so a direct TS-only CLI success does not qualify these producer paths.

## Decision

Select one pattern before consulting files: a matching exact key first,
otherwise the longest matching wildcard prefix. Equal wildcard prefixes keep
their incoming pattern order. Probe only that pattern's targets in their
existing order and stop at the first resolved target. Record its unsuccessful
probes too, so creation of a formerly missing candidate remains an invalidation
input. A missing selected pattern does not authorize another alias pattern.

The existing parser, native engine, source mapper, candidate extensions,
physical workspace/package ownership, `.vue.ts` folding and single-wildcard
prefix/capture/suffix substitution remain in place. Malformed multiple-star
keys/targets remain refused. This adds no provider or compiler pipeline stage.
It changes neither default feature gates nor Maestro configuration ownership.

## Reproduction and required qualification

The [authored corpus](../../../tests/_fixtures/differential/typecheck/path-alias-precedence-3984/README.md)
uses `@x` and competing wildcard targets with distinct exported primitive types.
Authentic public v0.439.0, with its actual native Corsa 7.0.2, reports a false
TS2322 on clean opt-in TSX and on the same SFC through public `check-server`.
The server's complete virtual TypeScript names the wrong physical wildcard
mirror. Stock TypeScript 6.0.3 and the actual native compiler agree on the
clean original TSX project. The inverse also reproduces a missing required
TS2322. Exact-key ties and ordered target arrays reproduce the same two
failures; missing selected exact/longer-wildcard targets replace the required
TS2307 with a wrong-module TS2322. All six original-engine expectations agree.

Before production changes, real Rust 1.99.0 executed the five added resolver
laws: one passed and four failed. The maintained native corpus driver also
failed its independent complete clean expectation against public v0.439.0.
Neither failure is an optional-source/native early return.

The existing `type-snapshot-bench.yml` now selects this source/corpus change
and requires all five selector laws, all six existing suffix laws, all eight
alias rewrite laws and the required-native App-only suffix diagnostic/repair.
Its existing source builds and three fixed CLI corpora retain all complete
diagnostic/program fingerprints, max/1T modes, two warmups and nine alternating
pairs. The paired workload, timing protocol and budgets are unchanged.

The same job additionally runs identical alias inputs through literal base
and head CLIs plus stock TypeScript 6.0.3 and native Corsa 7.0.2. It retains
full CLI reports, all check-server replies including generated documents,
input/executable/driver hashes, exact source SHAs and successful shutdowns.
Both exact-only controls require whole CLI report parity. Intended corrected
diagnostics stay separate from unchanged benchmark diagnostics. Timing is
reported only after that unchanged workload's own gates; no 10x improvement
is claimed. Fresh Actions, protected full native/corpus/instruction gates,
actual signed merge and released installed replay remain required.

## Remaining scope

#3984 remains open for full project/build/watch/reference/declaration/LSP
parity. #7698 remains open for throughput and shared graph obligations.
This change preserves incoming equal-prefix wildcard order, but the existing
config loader's `serde_json::Map` can lose authored JSON declaration order.
A typed ordered loader with its own original-engine corpus must address that
separate TODO; workspace-wide serde serialization order is not changed here.
Wildcard keys whose targets contain no `*` retain their previous unsupported
substitution boundary and need a separately scoped native regression.
Persistent edit/invalidation, broader wildcard shapes and public publication
are not credited by this cold selection corpus. Upstream remains read-only.
