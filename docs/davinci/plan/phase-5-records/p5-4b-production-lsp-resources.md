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
  of 32. No buffer is closed before the idle observation.
- Every SFC has real prop, emit, slot, model, exposure and component-option
  macros. Each has a distinct optional public prop; public interfaces cannot
  all collapse to one interned alpha page.
- Disjoint file pairs import each other. Each of the 10,000 files receives a
  template prop completion request, which resolves its distinct imported
  provider through the production component metadata and resident interface
  path. Each completion must contain the required `label: string` prop.
- Controlled unsaved edits on one pair check a body-only TS 2322 error, a
  public prop change through completion detail and importer diagnostics, an
  emit payload change through imported-component hover, and an exposure type
  change through template-ref TS 2339 diagnostics. Restoring the interface
  must clear importer errors. Diagnostic waits reject earlier publications.
- These are protocol outcomes and distinct provider request counts. Exact
  Salsa execution/reuse counts are established by the separate production
  interface tests; this probe does not infer counters from equal responses.

## Sampler and result scope

The Linux `/proc/<pid>/stat` sampler runs every 50 ms from server startup
through the final ten-second idle window. It sums RSS for Maestro and **all
live descendants**, including Corsa, and saves per-process RSS at peak and
idle. The Node harness and build processes are outside that process tree.
CPU is the sum of live processes' `utime + stime`; child CPU is not counted
twice through `cutime + cstime`.

Three independent sessions use `ci-opt` binaries (release, thin LTO, 16
codegen units) on `blacksmith-32vcpu-ubuntu-2404`. The artifact records all
per-run values and the maximum peak/idle RSS across runs. This broader scope
is **record only**: the 337/316 MiB ceilings belong to the separate nine-file
TS-44 baseline, and no new numeric ceiling is selected to fit this result.

```sh
vp exec node tools/support/compat/davinci/lsp-resources.mjs \
  --server target/ci-opt/vize --files 10000 --runs 3 --idle-seconds 10 \
  --out /tmp/production-lsp-resource.json
```

The optional `--open-files` flag supports an explicitly named workspace-only
scope. The Actions workload leaves it unset, so every generated SFC remains
open and metadata-warmed. Partial progress and any protocol failure are saved
to the artifact; incomplete measurements are not reported as accepted.

## Recorded run

The production job is part of the manually dispatched
`davinci-resource-budgets.yml` workflow. Its artifact is named
`production-lsp-resource-linux-x64-ci`. The first execution exposed a stable
Rust compilation error in the production type-world integration before any
LSP measurement; its isolated synthetic resident job passed. A successful
production observation and its exact workflow URL will be recorded here.
