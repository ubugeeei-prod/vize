# Supplemental semantic completion corpus contract

For [#6879](https://github.com/ubugeeei-prod/vize/issues/6879), the required
native semantic completion fixture uses a separate supplemental contract.
It supplements the source-built acceptance laws described in
[the original gate record](./2026-10-12-native-semantic-completion.md).

The original `batch-start-diagnostics-v1` registry keeps its 32 historical
projects, 112 physical inputs, all original pack bytes, checker options,
diagnostic vectors and capture/report contracts. The frozen escaped-import
case does not have that pack structure or that diagnostic position contract.
Registering it as a legacy pack made the strict loader reject its unknown test
body in four tooling shards at source `041d5df16fb7071aa8d429050c2f2f25efb992c7`.
The loader retains that refusal for every unknown original pack.

The new `supplementalCases` registration uses
`source-canon-semantic-completion-v1`, with `source-cli` and
`configured-lsp-owner` targets. Its separate validator requires the one
registered test identity and the complete independently pinned frozen oracle.
It authenticates eight physical files, five reported program members, both
full TS1260 and TS2304 tuples, complete program/config/options membership and
the original stdout, failing stderr, command and provenance bytes. Original
source, repair source and composed artifact source identities remain distinct.
Changing a source, raw packet, vector, identity or registration, including
rehashing a changed oracle, refuses qualification.

The supplement has no legacy/native differential adapters. Original reports
still contain exactly the same 32 historical rows and grant zero whole-product
native credit. A successful fixture load proves immutable custody and planned
source contracts only. Actual source-built CLI and same-owner native process
execution remain mandatory in the existing hosted native gate; loading the
fixture or passing synthetic refusal tests cannot replace their receipts.

The four original tooling logs also identify separate warm-terminal manifest
length, shallow-history Cargo.lock custody and raw artifact-fallback failures.
Their corrections belong to their existing delivery owners. The source-length
budget and vendor policy require no change for this corpus correction.
