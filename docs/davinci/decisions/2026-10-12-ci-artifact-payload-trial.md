# Complete artifact payload transport trial

This decision pairs [#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6111600216).

The [payload baseline](./2026-10-12-ci-artifact-payload-baseline.md) identifies
repeated candidate bodies in the formatter corpus and distinct historical native
binaries in the oxlint evidence. CRC and size never authorize deduplication.

The existing CI walltime comparison workflow gains a manual-only `artifact-payload`
component. Its default source is the complete formatter artifact `11709550349`,
from main `a49936db24d9540e0630b42474654190d575294b`. That original Check was
cancelled; the transport experiment cannot convert it into passing qualification.
The producer authenticates the source SHA, repository, full ZIP size and API
SHA-256 digest before reading any entry. It never records redirect credentials.

Every complete decoded file body receives a SHA-256 digest. The experimental
envelope stores one body per whole-file digest with deflate level 6 and a complete
manifest of original paths, file/directory types, modes and ZIP permission
metadata. Different historical binaries remain different objects. Source ZIP
bytes, source SHA, driver SHA, artifact/run identity and manifest/archive digests
remain in the receipt. Unsupported links, duplicate or unsafe paths fail closed.

A separate job downloads the exact experimental envelope and reconstructs every
path into a fresh tree. Files receive independent inodes, including equal bodies;
the verifier rejects missing or extra paths, changed modes, symlinks, hardlinks,
damaged objects or substituted source/driver/artifact identity. Empty files,
opaque raw streams, hidden names, Unicode, newline names and explicit empty
directories remain covered. Controls also run with Python optimization enabled.

Receipts record original and packed bytes, complete entry count, unique object
bytes, read/hash/compress wall time and full restore/verify time. GNU time records
CPU and peak resident memory for pack and restore; Actions step timestamps supply
actual upload/download durations. The jobs retain the complete manifest and
both resource reports independently of the envelope transfer.

This trial changes no original producer, default artifact upload, corpus fixture,
raw stream, native qualification, compiler pipeline stage or resource budget.
The `all` selector also leaves the expensive trial off. The initial trial starts
from a historical ZIP, so its timing includes original decompression and cannot
establish the cost of packing a live source tree. Global adoption remains TODO
until producer integration, physical native acceptance, original consumer
equivalence and a complete same-source cost comparison pass. Report measured
bytes and times separately from any target or unmeasured CI speedup.
