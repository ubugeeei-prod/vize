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

Reconstructed evidence accepts regular files and GNU tar hardlinks only. Source
collection still follows the original dependency aliases, while restored-tree
validation uses `lstat` and rejects file/directory symbolic links, including the
embedded manifest. Independent review reproduced an external raw-file symlink
substitution at the initial source below: identical external bytes could pass
its unchanged inner manifest with a recomputed outer archive hash. Three rebuilt
archive controls now require rejection for raw-file, directory and manifest
symlinks; external bytes cannot substitute for retained raw members.

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

## Hosted result and consumer review

Initial source/driver `24a8201fc568a8172b22b5f374a3877ae66377b4` passed every
original native gate in [run 38108847093](https://github.com/ubugeeei-prod/vize/actions/runs/38108847093).
The real manifest reconstructs all 685,199 files / 6,675,157,382 raw bytes, with
899,980,210 unique inode bytes. Inventory took 7.179 seconds, archive creation
2.060 seconds, and full reconstruction 12.637 seconds; internal packing and
verification totaled 22.001 seconds. The actual Actions pack step, including
cleanup, took 25 seconds, followed by a one-second upload. Artifact `11690688371`
contains 94,875,739 bytes, including the 94,873,950-byte tar archive.

Adjacent parent [run 38107509829](https://github.com/ubugeeei-prod/vize/actions/runs/38107509829)
at `024abf01387943cf14d21f437ddd3e4a41679c43` uploaded the same 685,199-file
workload in 536 seconds, producing 1,619,235,154 bytes. The observed artifact
phase decreases from 536 to 26 seconds and its size decreases by 94.1%. Fresh
capture timestamps/process/source receipts account for small raw-byte changes;
the original workload and eligibility remain intact. These are artifact results,
not isolated CPU/I/O/network attribution or type-checker execution improvements.

The downloaded actual envelope independently reconstructed all 685,199 files
and their full hashes/modes with GNU tar on macOS in 373.447 seconds. That local
filesystem measurement remains separate from the hosted Linux reconstruction.
The initial source's full Actions and seven regression laws passed; the stricter
symlink rejection must qualify again on a fresh exact head before delivery.

Consumer review at this source and authenticated main `db6e7805` finds no tracked
automatic consumer of `typechecker-native-phases`. All 54 `download-artifact`
steps select other artifacts or same-run release packages. Rust and JS release
preflight fetch raw JSON only from exact `real-project-matrix-{0..21}` artifacts;
they never select this native workflow/artifact. Those Matrix readers remain
unchanged and do not claim envelope support. Native evidence reconstruction is
documented above; future automatic native consumers must use the full verifier.
Protected merge qualification and actual delivery remain outstanding.
