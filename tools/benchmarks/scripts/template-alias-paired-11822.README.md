# Template alias paired campaign

This manual-only campaign compares published Vize 0.679.0, an approved exact current-main source, an approved candidate source, and the Vue mapper on the immutable `MatesEngineer/vue-typecheck-benchmark` commit `c9a59cb07c236cbf7e0a97aa06ca3d02756415ea` (2,500 SFCs, twelve planted diagnostics).

All repository orchestration, parsing, staging and tests are typed TypeScript, run directly by Node's stable type stripping. The sole `.mjs` asset is the protocol tee copied verbatim into consumer `node_modules`; it is executed there by the external mapper host. Source-pinned TypeScript 6 performs strict `noEmit` before native source builds. The benchmark's TypeScript 7.1 native compiler is a separate toolchain.

The workflow must exist on the default branch before its first dispatch. Dispatch targets the reviewed candidate ref containing the harness; the executed workflow SHA must equal that source SHA. Root controls source pushes, exact source approval and dispatch. Preparation does not authorize a run.

## Admission and retained evidence

- Each source produces one NAPI build with a fresh token, matching `ci-opt` flags/features, generated NAPI SDK declarations, normal CLI packing, normalized publish manifests and a closed runtime capsule. Every Vize cohort uses the normal Node CLI and NAPI package endpoint. A `.node` image is hashed and observed as a mapped shared image; it is never spawned as a command.
- Five untimed native/wrapper probes and eight untimed captures precede forty primary processes: four systems, two default hosts and five alternating repetitions. Missing diagnostics, messages, locations, codes, duplicate errors, status differences, unexpected `TS6142`, incomplete transforms or changed authored directive authority fail admission.
- Declaration roots are explicitly listed `.d.ts`/`.d.mts`/`.d.cts` files, top-level types and every conditional types export. All relative imports and reference paths are resolved recursively to physical declaration files. Missing globals, slots, listeners or unused conditional targets fail independently of `skipLibCheck`. Unreachable private implementation declarations remain in the full inventory; reaching an out-of-package declaration fails.
- Every primary invocation uses a fresh process/workspace and empty compiler/mapper cache. OS page cache is not flushed. Complete provider/capsule/image custody hashes run outside GNU time, equally around each observation. Raw streams, protocol, transform text, metadata, directives, source identities, manifests, SDK closure and CPU-width receipts are retained.
- Checker `Memory used` remains separate from GNU-time maximum RSS; the latter is not simultaneous process-tree RSS. Pinned expected SRI is recorded alongside installed payload SHA and pre/post equality, without claiming independent tarball verification.

## Budget and incomplete campaigns

One Blacksmith 32-vCPU job has a 45-minute hard limit and a shared 39-minute processing deadline. The final six minutes are reserved for a small raw artifact, a bounded full archive and its upload. Each source install/build opens original stdout/stderr files and persists argv, allowlisted environment, source SHA and start metadata before spawning. GNU timeout supervises the stage; end status is stored separately. A timeout or partial sample population fails the campaign.

Three installs capped at five minutes each and two NAPI builds capped at twelve minutes each already sum to 39 minutes before setup, packing, captures and measurements. Completion within 45 minutes is **not established**. The current cap preserves bounded cost and failed evidence; a larger one-campaign ceiling requires a new explicit root decision. No medians or speed claims are accepted from truncated runs.

Pure controls can run with an existing source-pinned JS API, without installing or building anything:

```sh
TEMPLATE_ALIAS_TYPESCRIPT_API=/absolute/source/npm/cli/node_modules/typescript/lib/typescript.js \
  node --test tools/benchmarks/scripts/template-alias-paired-11822.test.ts \
  tools/benchmarks/scripts/template-alias-paired-11822-declarations.test.ts
```

Passing controls or a green workflow is not diagnostic/performance acceptance. Shared emit SDK semantics, canonical baseline qualification and large real-project regression checks remain separate gates. Historical 679 counters and M5 Pro timings must not be presented as current-main measurements or Blacksmith reproductions.
