# L0 owns its foundation laws (2026-10-01)

Decision for [#6833](https://github.com/ubugeeei-prod/vize/issues/6833).

Move all remaining foundation integration laws and key fixtures from the
compatibility substrate to L0, beside their implementation. The move-only
commit relocates 68 files: 22 harnesses, their support modules and frozen
fixtures. Integration imports canonical `vize_l0::diag` and its witness
verifier, keys, fact/pass managers and Dump APIs directly. All key hashes,
schema assertions, witness cases and diagnostic/remark bytes stay fixed.

The key laws continue to invoke the same real L1/L2 producers and SFC oracle.
These are dev-only, version-less path dependencies: they introduce no reverse
normal/build edge and are removed when the foundation is packaged. Higher-level
native implementation and product acceptance remain separate work.

`remark_zero_cost` keeps `harness = false` and the exact standalone allocation
assertions, so libtest's reporter cannot pollute process-wide measurements.
Captured compiler stdout/stderr, receipts, one-shot source and fixed vectors
move byte-for-byte; their historical package/path strings are not rewritten.
The Linux/macOS TS-43 and unused-bindings fact commands use L0, and the latter
workflow also triggers on the foundation it now tests. Source witnesses and
compiled-doc inputs use the actual moved paths.

Replay `python3 tools/support/levels/move-l0-runtime-laws.py moves`, commit only
the file moves, then run `integrate`, format Rust, normalize Cargo metadata
and regenerate inventories. Collision checks cover every move before index
mutation; integration buffers every path/manifest rewrite before writing.

Local verification passed all 342 L0 unit tests and all 22 integration
harnesses, including the standalone zero-cost executable. The explicit
capture-only vector observer remains ignored as before. Fifty-five tooling
laws passed, and all moved fixture bytes match the move-only commit. Actions
and actual protected-queue merge remain required. #6833 still needs the final
legacy consumer imports, benchmark ownership and compatibility package deletion.
