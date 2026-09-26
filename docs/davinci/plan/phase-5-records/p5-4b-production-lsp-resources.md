# Production 10k-file LSP resource observation

This measurement supplements the P5-4b isolated resident database probe. The
acceptance text in `phase-5-tasks.md` asks for a **synthetic** 10k-file session
under the P5-11a RSS preset. It does not specify 10,000 open production LSP
buffers. The stricter production workload below is recorded separately and
does not change the existing nine-file TS-44 or synthetic resident gates.

## Workload and protocol

- Exactly 10,000 generated SFCs on disk, with real Vue declarations and the
  repository-pinned native TypeScript runtime. Editor, ecosystem, lint and
  typecheck services are enabled throughout.
- All 10,000 SFCs are opened through stdio `textDocument/didOpen`, in batches
  of 32. Each must answer a structural `documentSymbol` request with script
  and template blocks, establishing actual document residency. No buffer is
  closed before the idle observation.
- Every SFC has real prop, emit, slot, model, exposure and component-option
  macros. Each has a distinct optional public prop; public interfaces cannot
  all collapse to one interned alpha page.
- Disjoint file pairs import each other. Sixteen evenly distributed files
  receive template prop completions, resolving sixteen distinct providers
  through production metadata and resident interfaces. Each completion must
  contain the required `label: string` prop. This establishes sixteen warm
  interface providers, not 10,000 alpha publications.
- Before bulk opening, 32 body edits on one open pair require unchanged prop
  completion replies. Controlled unsaved edits check a body-only TS 2322 error, a
  public prop change through completion detail and importer diagnostics, an
  emit payload change through imported-component hover, and an exposure type
  change through template-ref TS 2339 diagnostics. Restoring the interface
  must clear importer errors. These diagnostic checks run with two open files;
  the final residency observation has 10,000. Waits reject earlier publications.
- These are protocol outcomes and distinct provider request counts. Exact
  Salsa execution/reuse counts are established by the separate production
  interface tests; this probe does not infer counters from equal responses.

## Sampler and result scope

The Linux `/proc/<pid>/stat` sampler runs every 50 ms from server startup
through the final ten-second idle window. It sums RSS for Maestro and **all
live descendants**, including Corsa, and saves per-process RSS at peak and
idle. The Node harness and build processes are outside that process tree.
CPU retains each observed process identity's `utime + stime` counter after
exit, keyed by PID and birth ticks. Retiring a Corsa process therefore cannot
subtract its prior CPU. Child CPU is not counted twice through `cutime + cstime`.

The dispatch lane records one independent session using a `ci-opt` binary
(release, thin LTO, 16 codegen units) on `blacksmith-32vcpu-ubuntu-2404`.
`--runs` can request additional independent sessions; the artifact records
every per-run value and the maximum peak/idle RSS across completed runs. This broader scope
is **record only**: the 337/316 MiB ceilings belong to the separate nine-file
TS-44 baseline, and no new numeric ceiling is selected to fit this result.

```sh
vp exec node tools/support/compat/davinci/lsp-resources.mjs \
  --server target/ci-opt/vize --files 10000 --runs 1 --idle-seconds 10 \
  --warm-providers 16 \
  --out /tmp/production-lsp-resource.json
```

The optional `--open-files` flag supports an explicitly named workspace-only
scope. The Actions workload leaves it unset, so every generated SFC remains
open, with 10,000 structural witnesses and sixteen warm metadata providers.
The measurement command has a 45-minute deadline within the 60-minute job,
and artifact upload always runs. Partial progress and protocol failures are
saved; incomplete measurements are not reported as accepted.

## Recorded run

The production job is part of the manually dispatched
`davinci-resource-budgets.yml` workflow. Its artifact is named
`production-lsp-resource-linux-x64-ci`.

The bounded scope completed in [run
36240385542](https://github.com/ubugeeei-prod/vize/actions/runs/36240385542/job/108399647897),
resource `1fc3a8685c1dcfe3e00448b2ed2fd22a0953aa25`, producer
`ad5c60f710547d3f6431d9a12224616316b4449b`, artifact `10904744181`.
The [complete measurement JSON](./p5-4b-production-lsp-resources.json) is
retained with this record: one 85.510-second session, 10,000 open buffers and
10,000 successful residency replies, sixteen metadata provider replies,
32 body edits, and the controlled public-interface diagnostic oracle.

| Observation                      |  Summed RSS |   Maestro |                Corsa descendants |
| -------------------------------- | ----------: | --------: | -------------------------------: |
| Peak                             | 1,118.9 MiB | 640.9 MiB | 478.0 MiB across three processes |
| End of ten seconds without input | 1,005.9 MiB | 642.6 MiB |   363.3 MiB across two processes |

The input-idle window still had background work: cumulative sampled tree CPU
was **144.8501%**, summed across cores and retaining exited children's last
observed counters. This is not settled idle. An earlier successful run,
`36239737301`, used a net live-process CPU difference that undercounted checker
churn; its CPU result is superseded. Multiple checker children are observed,
not proof of a leak: the client owns native/editor transports and may briefly
overlap a replacement session with its predecessor during project reload.
No broader RSS or CPU ceiling is claimed. The prior corrected run
`36240038670` on producer `049358b20` recorded peak 1,120.8 MiB, input-idle
1,039.5 MiB, and sampled CPU 143.8348%; those remain historical measurements
of that exact earlier producer, rather than the final runtime observation.

The earlier strict attempt requested a native prop completion after every
open batch, intending to warm all 10,000 interface providers. [Run
36236392245](https://github.com/ubugeeei-prod/vize/actions/runs/36236392245/job/108388832165)
on resource `e1bee278790a2d78742d21f5000fb8c3b3932131`, producer
`56a65d3d314975ea29aa66f4f0f8cd942d67b8d0`, hit the 60-minute job timeout.
Its last logged checkpoint was **2,304 open/warm files, 7,138.5 MiB summed
RSS at 51.7 minutes**. This is an incomplete observation, with no final peak
or idle measurement. Increasing request cost and memory growth require a
separate production scaling investigation. The bounded workload above does
not establish the abandoned all-provider scope.

The same final runtime producer's unchanged nine-file TS-44 baseline passed
in [run 36240385542](https://github.com/ubugeeei-prod/vize/actions/runs/36240385542/job/108399647923),
artifact `10905870609`. Three runs recorded maximum peak **286.4 MiB**,
maximum idle **277.5 MiB**, maximum idle CPU **0.217%**, and keystroke p95
**29.9 ms**, within the existing pinned ceilings. Cold-start median was
5.6 ms and first-diagnostic median 1,116.7 ms. The isolated synthetic resident
guard also passed, artifact `10905254363`; it retains its separate scope.

## Scaling followup

Source inspection establishes repeated project-wide work in the strict sweep:

- Native prop completion opens a canonical project after obtaining structural
  interface metadata (`completion/template/component_native.rs`).
- Every open canonical request snapshots all open Corsa overlays, canonicalizes
  their paths, and sorts/hashes every overlay's bytes for a context fingerprint.
- The per-host Canon context cache holds eight entries. Distinct host requests
  miss it; rebuilding registers previous live project sources and regenerates
  their Vue projections (`vue_dependencies_alias/context/build.rs`).
- Each rebuilt context stamps its source closure, materializes the project
  union, and returns cloned materialized source text and mapping tables.

This repeated growing-project work is consistent with the observed time curve;
its individual contributions have not been profiled. The old strict log only
has summed RSS, so its 7,138.5 MiB cannot be attributed to Maestro or Corsa.
The bounded artifact records that attribution. Canon bridge profiling is
disabled by the current Maestro bridge configuration, and overlay counters
are test-only. A separate Canon/Maestro scaling change must preserve unsaved
dependency freshness and live project membership while reducing this work.
