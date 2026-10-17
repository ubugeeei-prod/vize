# Reuse each freshly verified native preparation identity

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The JS builders test step took 207 seconds in [run 38935551261](https://github.com/ubugeeei-prod/vize/actions/runs/38935551261/job/117090136978)
and 182 seconds in [run 38936385902](https://github.com/ubugeeei-prod/vize/actions/runs/38936385902/job/117092960110).
Its public Vite CLI formatter test took 94.595 and 90.205 seconds; the public
NAPI formatter test took 74.103 and 70.472 seconds. These test files ran
concurrently. Those durations include all test work, not just hashing, and
both runs preceded this change. Actual saved seconds remain unmeasured.

Each of the 27 CLI passes previously hashed the local addon in
`nativePreparationIsActive`, then immediately hashed it again in `currentAddon`.
Return the freshly verified preparation identity from a shared reader and
retain the existing boolean API as its wrapper. Subsequent observations reuse
that call's verified hash, removing exactly 27 redundant whole-addon hashes.
Every call still reads the current receipt and validates the live owner,
source HEAD/tree/diff, single binding name and full addon bytes. There is no
cross-call cache or admission based only on file metadata.

Keep the independent initial post-load hash. Subsequent observations require
the exact initially loaded parent modules, data-valued exports and public/local
API descriptors to remain identical. A scoped verifier checks those handles
before each fresh hash and never invokes `require` or an export getter afterward.
Eviction, module replacement and accessor substitution fail closed. Each call
still rereads the preparation receipt; the final child has the same source and
receipt checks as every earlier child.

The child CLI loader is unchanged: each actual load retains its own full
before/after hashes, exact provider, export descriptors and errors. All nine
CLI cases and 27 check/write/recheck passes remain, including whole streams,
files and temporary-config cleanup. All nine NAPI cases and 25 passes, their
build/toolchain/source checks, and the retained artifacts are unchanged.

All 18 focused preparation and public formatter contract tests passed locally,
with no skips. Four new controls cover fresh per-call hashes, changed addon
bytes with restored mtime, source edits, stale owners, mismatched receipts or
bindings, caller mutation, module eviction and changes during initial loading.
They also reject replacement module objects and mutating export/API getters
without executing those getters.
Synthetic loader controls claim no native execution credit; hosted product
tests must still pass before this change qualifies.
