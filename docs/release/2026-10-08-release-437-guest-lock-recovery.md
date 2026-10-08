# v0.437 pre-pin guest-lock recovery

The official `vp run release minor -y --pin` invocation on 2026-10-08 stopped
before creating a metadata PR, pin, hosted Release run or tag. Source
[#8312](https://github.com/ubugeeei-prod/vize/pull/8312) retained
C `9ce571fb9ab649def7f8246c67d4e9d1979c5c98` and
H `57d58dca15bf2e31c599d9c6f916460de80c568e`.

The real offline `cargo update -p vize_guest` resolver downgraded unrelated
`anyhow` 1.0.104 to 1.0.103, `bitflags` 2.13.2 to 2.13.1 and `cfg-if` 1.0.5
to 1.0.4, changing their checksums in all four standalone SDK locks. The
unchanged strict metadata comparator correctly rejected the first affected
lock. Unchanged-H resume independently reproduced that refusal.

The producer now uses that existing exact version transformation to update
only the in-tree SDK version. The real Moon preparation regression retains
all four complete locks and compares each entire output against the original
bytes with only `vize_guest.version` changed. Its simulated offline downgrade
path remains armed if preparation accidentally invokes Cargo for a guest lock.
The comparator, other preparation operations and release gates stay intact.

After the repair actually merges, the release owner will verify no hosted
Release, pin, tag or publication exists, archive frozen H under
`release-archive/v0.437.0-pr8312`, verify the archive's exact SHA and close
#8312 as abandoned/unpublished. Delete only the owned `release/v0.437.0` ref
with a `57d58dca15bf2e31c599d9c6f916460de80c568e` lease. Preserve both failed
generated worktrees, raw refusal logs and commit objects. This narrow
pre-pin recovery is authorized for this failed source; it adds no retirement
framework and never changes an immutable tag or the old source commit.

Then invoke the official minor pinned release again from fresh genuine main,
creating fresh C/H/R and running every original source, full Check, Fuzz replay,
Miri, Matrix, Docs, build, preflight, protected metadata and publication gate.
PR-only ELOOP investigations, supplementary custody or cryptographic preparation
are separate work and add no pre-cut gate. Public GitHub Release, all configured
registries/assets and both editor channels still require terminal verification.
No source-green, archived source or retired v0.436 artifact is publication credit.

The primary local evidence is `/private/tmp/vize-v0437-8312-prepin-failure/`:
API source/run inventory, exact remote refs, all four C/H locks and deltas, the
whole unchanged-H resume log and a hashed receipt. This change also records
its decision on [#6239](https://github.com/ubugeeei-prod/vize/issues/6239) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
