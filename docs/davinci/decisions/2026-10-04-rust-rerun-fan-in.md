# Rust worker reconciliation after a failed-only rerun

Refs: #6830, #6861. This repairs the existing full Rust report without changing
the workspace archive, test partitions, product stages, coverage or timeouts.
The prospective #6863 tooling latency proposal remains separate.

## Genuine failure and GitHub reuse contract

The original #7743 candidate was
`7cebaf4aadf87586f2970fb35a9aef0c251d5d65`, source
`596183affa97edd5da922fa8c62017f2bcf56d66`. Its
[Check attempt 1](https://github.com/ubugeeei-prod/vize/actions/runs/37182670251/attempts/1)
cancelled worker 1, job `111380002920`, after the 20-minute deadline during
the 3,641,699,748-byte archive download. Archive verification and source tests
had not executed in that worker. This is infrastructure failure, not Rust
test success. The archive `11295508245` is genuinely named by run ID and source
SHA; schema 3 permits reuse across attempts of that same source.

A later read-only observation found
[attempt 2](https://github.com/ubugeeei-prod/vize/actions/runs/37182670251/attempts/2)
terminal failure. This repair did not trigger either attempt or mutate a queue.
The official job API reports successful worker IDs
`111383872160`, `111383872888`, `111383872899`, `111383873044`, all with
`run_attempt: 2`. Only worker 1 executed anew, 07:03:07–07:05:31 UTC.
Workers 2–4 retain their original execution windows and complete step/runner
records from jobs `111380002178`, `111380002111`, `111380002125` respectively.
Their immutable artifacts still have their original attempt-1 names. Selecting
the newest reported attempt alone would incorrectly require missing artifacts.

The actual attempt-2 report job `111384220007` passed its complete-tier gate
and artifact download, then failed the unchanged observation aggregate: its
attempt-only pattern supplied only the newly uploaded worker 1 artifact.

The actual four artifacts are `11296686359` (worker 1, attempt 2),
`11296081419`, `11296415584`, `11295409064` (workers 2–4, attempt 1).
Official artifact creation times fall inside each original successful upload
step. This demonstrates metadata selection only; it does not turn that failed
workflow into acceptance or transfer its runtime proof to a new source.

Read-only downloads of those four small ZIPs independently verified every
official digest, source/tree, genuine JUnit hash and schema-3 runtime envelope.
All four retain the identical complete archive receipt and capture binary SHA;
the 14 actual registered fixture bodies are distributed 3/6/3/2. The large Rust
archive was not downloaded and no new source tests or aggregate were executed.

## Selection and refusal policy

1. Bind the official run to the exact checked-out SHA, run ID, current attempt,
   repository, `merge_group` event and `.github/workflows/check.yml` caller.
2. Fully paginate official jobs with `filter=all` and the run's artifacts.
   Require four exact worker identities, unique job IDs and one job per worker
   per reported attempt. Foreign source/run and future attempts fail.
3. For each worker, inspect its latest reported outcome first. It must have
   completed successfully and executed archive verification, unfiltered shard
   tests, real observation verification and upload, in that order. Failure,
   cancellation, skip and incomplete execution never fall back to older success.
4. A carried success may use an earlier execution only when its complete
   execution windows, outcome, all step records and runner metadata exactly
   match the earlier official source-bound job. Retain both job IDs and both
   attempts. Any changed execution requires its own correctly named artifact.
5. Require exactly one unexpired, nonempty artifact from that original
   successful upload, with exact source/run/repository identities and a SHA-256
   digest. Retain original artifact IDs, names and digests; rewrite none.
6. Download the four immutable IDs through pinned download-artifact v8 with
   explicit `digest-mismatch: error`, read-only token and run ID. Keep separate
   original artifact directories. Re-read official metadata after downloading;
   require the same selection and exactly those four directories before the
   unchanged full observation aggregate executes.

`worker-selection.json` remains alongside the original corpus report and
acceptance artifact. Read-only Actions permission propagates only through the
two existing reusable-workflow caller jobs and the Rust report job. The
unconditional needs gate remains first, and the outer required report still
propagates failures. Ordinary PR archive selection is unchanged.

The existing typechecker aggregate retains exactly four distinct shards,
exact source commit/tree and manifest, identical complete archive receipts and
binary identities, genuine JUnit/test/capture identities, full registered
counts, duplicate rejection and all original failure guards. Rebuilding an
archive while reusing workers from a different build still fails its identical
receipt/binary checks. No archive, worker receipt, artifact or test is relabelled.

## Source validation and pending delivery

The isolated implementation starts at literal main
`912d311cc378db3506eb7ce78a6f51b0a37f91ff` after #7743 merged through its distinct
healthy candidate. The historical `7ceb` failures stay separate. Thirteen pure
laws cover mixed and full reruns, carried execution identity, latest failures
and pending outcomes, absent/duplicate/foreign workers and artifacts, strict
digest/upload windows, later-page failures, identity drift and four-directory
inventory. Existing workflow and archive-reuse laws are retained and extended
only for the explicit read permission and source-bound selection wiring.

The first publication source `3e04d07cf60817c605d2f3cafa4352ed81f88367`
failed `check-js` in Check `37187992781`: exactly 13 floating Node test
registration promises violated the existing zero-warning rule. Marking those
registrations with `void` retains all callbacks and assertions; no production,
worker selection or coverage contract changes. The failed source is retained
separately, and corrected-source hosted validation remains required.

Pure laws and metadata checks are not hosted source execution. Root and peer
source reviews cleared the frozen contract and authorized publication. Post the paired issue decision,
require exact-head Actions, full protected validation and literal merge; retain
all actual selected worker IDs/attempts, source/build receipts, named full counts
and required contexts. A genuine failed-only hosted execution of this new
workflow remains pending and must be reported separately from synthetic laws.
Never deliberately fail or retry a healthy queue candidate to manufacture proof.
