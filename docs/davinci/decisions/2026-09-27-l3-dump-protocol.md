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
