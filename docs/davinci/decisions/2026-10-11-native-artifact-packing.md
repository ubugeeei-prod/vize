# Lossless native evidence packing

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830). This storage
change follows [metadata event routing](./2026-10-11-native-phase-metadata-edits.md)
and changes only the final native workflow artifact transport.

## Measured cause

Successful native [run 38101861881](https://github.com/ubugeeei-prod/vize/actions/runs/38101861881)
at source `dee6d6e5b77090ee1be1569891ae498e2a6faa09` uploaded 685,199 files in
555 seconds, producing artifact `11688093783`, a 1,619,229,991-byte ZIP.
Its complete ZIP central directory was inspected with HTTP byte ranges, without
downloading the complete payload. The inventory contains 6,675,178,577 raw
bytes; compressed file content occupies 1,406,501,743 bytes and per-file ZIP
headers plus the central directory occupy another 212,728,248 bytes.

The JSX slot capture alone contributes 671,725 files, 6,044,616,465 raw bytes and
1,324,958,884 compressed bytes. Twenty-one case `node_modules` trees each expand
about 31,986 files and 288 MB. The unchanged JSX CLI fixtures create directory
symlinks to the same complete installed dependencies. The pinned upload action
follows those symlinks, processing each logical path as a separate ZIP file.
Repeated TypeScript and Ant Design payloads share CRC32 and size across all
21 paths; that inventory observation is not an independent SHA identity claim.

The current cache PR's successful native [run 38106167242](https://github.com/ubugeeei-prod/vize/actions/runs/38106167242)
at `c4828443ddf10b6579a3024c7882da2c7671eb49` repeats the same 685,199 files,
produces 1,619,219,962 bytes and spends 525 seconds in the upload step. Its
native workload finished before that upload. The historical and current upload
action, original path selection, hidden-file policy and retention are identical.
These figures separate artifact work from native diagnostic execution. They
do not isolate ZIP CPU time from file I/O and network time.

## Decision

Retain every original native execution, input, oracle, result, source receipt and
budget. Move the exact original 22 upload patterns into a source-owned path
list. Match the pinned action's eligibility: follow directory/file symlinks,
include hidden files, omit broken links and cyclic directory traversal, retain
all distinct alias paths, and exclude paths outside the original selection.

Pack the complete logical file tree with GNU tar and four-worker Zstandard.
Dereferencing the aliases produces regular file entries and hardlink records
for repeated inodes, retaining every path while encoding shared content once.
A real 21-directory-symlink fixture with an underlying file link count of one
produces one data record and twenty hardlink records with GNU tar 1.35.
Packing does not add source hardlinks or change source inode, mode, modification
time, change time or link count. No dependencies or fixture data are omitted.

Embed a manifest containing every original relative path, byte length,
permission mode and SHA256, bound to the immutable source and driver SHA. Each
physical file hash is reused only for the same device/inode/size/mode/time
revision. The transport receipt records the full archive and manifest hashes,
logical raw bytes, unique inode bytes, archive size and separate inventory,
archive creation and reconstruction timings. Archive creation includes file I/O
and compression rather than claiming isolated CPU time. SHA hashes replace no native gate.

Extract into a fresh directory and require exact full manifest equality before
uploading. The artifact keeps its name `typechecker-native-phases` and retention
of 30 days, with a two-file envelope: `native-phase-evidence.tar.zst` and
`receipt.json`. Upload the already compressed archive at compression level zero.
If packing fails, the job remains failed and uploads the entire original raw
selection with the original compression level and hidden-file behavior.

## Reconstructing evidence

Download the artifact and reconstruct its complete raw tree:

```sh
gh run download RUN_ID --name typechecker-native-phases --dir capture
vp node tools/support/compat/github/native-phase-artifact.mjs reconstruct capture restored
```

The destination must be fresh. Reconstruction verifies the archive, embedded
manifest, full file inventory, every byte hash and modes; it rejects damaged,
missing, changed or additional raw payloads. The restored tree is self-contained
and retains all original case paths. Raw JSON and receipt bytes remain exact.

## Verification and outstanding measurement

The source fixture reconstructs all 21 actual shared-inode aliases and checks
the full independent filename/byte/SHA inventory, including binary, Unicode,
newline, hidden, executable and original failed-result files. Damaged transport,
missing raw payload with a recomputed transport hash, and changed reconstructed
bytes are rejection controls. The workflow law retains every original pattern,
all preceding qualification steps and the failed-pack raw fallback.

Local fixture checks establish lossless transport, not hosted speed. Fresh
current-source diagnostics, full source/protected gates and real artifact
packing must pass on Actions. Compare the same retained 685,199-file workload
against the 525–555-second baseline using the new phase timings and actual
upload duration. Report CPU/storage/network effects separately where measured;
no speed improvement, protected delivery or type-checker acceptance is claimed
before those witnesses complete.
