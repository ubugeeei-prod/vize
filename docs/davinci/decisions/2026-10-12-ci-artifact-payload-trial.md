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

Full Check `38182228212` exposed source pollution from the new control launcher:
Python wrote `__pycache__` beside imported helpers, and the existing jsconfig
clean-source oracle correctly refused that checkout. Every payload Python
launcher now uses `-B`, including normal and optimized controls, pack/download
and restore. A real temporary Git inventory reproduces the untracked cache
without `-B` and stays unchanged with it under the Linux source-local cache
policy. No ignore rule, cleanup, original assertion or qualification is weakened.
Fresh exact-head full Actions remain required after this repair.

Peer review reproduced a silent inventory omission in the original verifier:
`os.walk` skipped an unreadable mode-000 directory and could miss an unexpected
opaque child. Walking errors now refuse full verification, and the restored root
must itself be an independent directory. A real permission control fails against
the retained original helper and passes after the fix; root runners exercise it
under an unprivileged child identity. Existing bytes and mode checks stay intact.

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

## Measured complete historical formatter round trip

[Actions run 38159309222](https://github.com/ubugeeei-prod/vize/actions/runs/38159309222)
passed both jobs at driver `cbd123f8eb84b57f77fb1c1db33dadfd280c89b6`, with
original source `a49936db24d9540e0630b42474654190d575294b`. Both the complete
original ZIP API digest and all 142,033 decoded file bodies were verified. The
fresh consumer restored all 142,033 files, independent inodes, modes and paths.

| Measurement                                 |                      Observed value |
| ------------------------------------------- | ----------------------------------: |
| Original ZIP                                |                 1,758,920,917 bytes |
| Uploaded complete envelope                  |                   474,358,605 bytes |
| All three experimental retained artifacts   |                   480,761,556 bytes |
| Unique objects                              |                              67,948 |
| Decoded original / unique bodies            | 7,807,255,030 / 3,345,896,436 bytes |
| Original download and digest                |                      16.248 seconds |
| Pack receipt / GNU wall time                |              48.417 / 49.94 seconds |
| Pack user / system CPU                      |                42.32 / 6.42 seconds |
| Pack peak resident memory                   |                         338,060 KiB |
| Envelope upload / consumer download step    |               2.664 / 3.885 seconds |
| Complete restore and verify / GNU wall time |              28.798 / 28.84 seconds |
| Restore user / system CPU                   |               18.24 / 10.52 seconds |
| Restore peak resident memory                |                         245,644 KiB |

The complete envelope is 73.03% smaller than the original ZIP; including the
separate provenance and consumer artifacts, this experiment retains 72.67% fewer
new bytes. The existing original remains retained, so this trial itself increases
total storage. Packing and verification add about 79 seconds of work; this is a
measured transfer-size result, not an end-to-end CI speedup or source acceptance.

All 90 native TypeScript `lib/tsc` bodies now have complete SHA-256 evidence,
each equal to `4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`.
The trial also retains all other distinct objects; no oxlint historical binary
was sampled, substituted or removed. The local regression fixture additionally
contains different equal-size bodies with equal CRC32, which remain distinct
SHA-256 objects. The authenticated producer manifest digest is
`61e378a67fe68c1b716bd12bed4aa52e240efcd1d6b72a726d8407d511a37cdd`;
envelope artifact `11709409248`, producer `11709549012` and consumer
`11709643770` retain the complete proof. Actual queue qualification remains
separate from this historical round trip.
