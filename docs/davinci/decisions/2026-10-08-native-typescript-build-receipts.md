# Strict native TypeScript source-build receipts

Tracking: [#6828](https://github.com/ubugeeei-prod/vize/issues/6828) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The source-build receipt provider moves alone in byte-exact commit `167fef15ef`
before typing. Its `BuildIdentity`, `BuildReceipt`, and closed `BuildRecipe`
types retain the original schema, manifest version, source revision, binary
path/hash, and all three build recipes. Unknown input is refined in place;
validation never rebuilds or projects a receipt. Existing assertion messages,
ordering, binary custody, and the immediately-after-build capture remain.

The provider previously imported the formatter manifest and its whole harness
merely to obtain `sha256`. The original Node `createHash().update().digest()`
body now lives in typed `tests/differential/sha256.ts`; the harness keeps its
existing public re-export and the provider imports the primitive directly.
This removes an unrelated formatter graph from the scoped type project without
an unchecked JavaScript boundary, serialization, or another pipeline stage.

`tsconfig.source-build-receipt.json` extends the strict, erasable Node foundation
and checks both actual provider roots with the existing native TypeScript 7
compiler. The existing repository check runs it after its original warning
gate. Current callers and build commands name `.ts`; Node 24.14.0 executes the
source directly. Historical `.mjs` fixtures, witnesses, hashes, output packets,
and earlier source/build identities retain their original bytes and authority.
Public package entrypoints and preload interfaces retain their contracts.

Local native TypeScript checking and 24 existing compiler, formatter, harness,
and LSP source-build custody laws pass. They include stale/forged identities,
closed legacy/shipping recipes, missing/corrupt/stale sources, and byte custody.
No native executable was built for these checks. Current exact-head Actions
and protected queue execution are still required before delivery; inherited
native proofs cannot qualify the new source extension.

This provider precedes the complete n8n typed qualification graph. Publication
must follow the genuine current TypeScript foundation chain with registered
native Stack membership before admission, or fresh signed `main` after actual
parent delivery. The source migration supplies no installed-package, complete
adoption, or performance qualification.
