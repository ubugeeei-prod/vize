# Temporary allocation JUnit evidence transport

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The independent allocation protocol PR is source `0252e59`, with original
successful required run `36966348830`, attempt 1. Patina's real named case
passed in shard 1/job `110711652033`; L0's passed in shard 4/job `110711652099`.
This temporary child reads only those two existing artifact IDs
`11210162486` and `11209886784`, the original source/merge metadata, complete
attempt job inventory, and original worker logs using Actions read permissions.

The collector verifies both service ZIP digests, every entry's CRC/content
hash, the original source tree/parent identity, and successful named worker
results against the actual corresponding non-skipped JUnit testcases.
It preserves literal ZIP bytes, every complete artifact entry including all
JUnit testcases/failures/skips, and authority records in hashed lossless frames.
The local receiver must authenticate the actual successful collector job and
verify every frame/content hash before accepting either JUnit observation.

This child has no PR and is never merged. It installs no dependencies and
runs no Cargo, test, product, benchmark or observer. It changes no protected
gate or allocation/instruction budget. Ordinary PR evidence cannot stand in
for fresh merge-queue full-profile worker/JUnit evidence or an actual merge.
Transport acceptance, manual full checks and queue validation remain pending
until their real terminal records are observed.
