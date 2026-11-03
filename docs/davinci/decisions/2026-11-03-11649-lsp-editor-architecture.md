# LSP editor architecture: one Vue layer over three TypeScript backends (2026-11-03)

Tracking: [#11649](https://github.com/ubugeeei-prod/vize/issues/11649).
Measured with the `lsp-vs-volar` harness
(`tools/benchmarks/scripts/lsp-vs-volar*.ts`, `.github/workflows/lsp-vs-volar.yml`)
on Blacksmith (8 pinned cores) and on a local 300-SFC project.

## Decision

`vize lsp` keeps one Vue layer and one request/merge policy over a single
`TypeBackend` interface with three first-class TypeScript backends:

- **A**: TypeScript native ≥ 7.1, `tsc --lsp` with the Vize content mapper;
- **B**: TypeScript native 7.0 through Corsa, which is today's path;
- **C**: the project's own TypeScript 5.7–6.x tsserver with
  `@vizejs/typescript-vue-plugin`.

All three consume the same content-mapper projection. The backend is chosen
once per workspace from the project's TypeScript. Every backend must meet the
same targets, and every maestro test and LSP response fixture runs against all
three with identical expected results.

The reason is the maintainer's requirement that Vize work in every
environment: TypeScript 6 and earlier, TypeScript native without
content-mapper support, and TypeScript native with it. So this is not a
migration from one engine to another. Nothing listed as a backend is deleted.

## Target

At least 10× better than Vue's official tools (`@vue/language-server` plus
tsserver with `@vue/typescript-plugin`) on misskey and n8n, for each of:

- session CPU;
- diagnostics after an edit;
- hover, completion and definition p99;
- peak memory.

Per-edit cost must depend on the edited SFC and its real dependents, never on
project size. Correctness must not regress.

Accuracy is judged against the correct answer, not against Volar. Vize does
not aim for vue-tsc or Volar compatibility. Volar is a reference, not the spec.
Hover shows the instantiated type. Definition, references and rename find every
authored location, even where Volar finds fewer.

## Status

Backend A wins big on cold open on both fixtures (7.7× misskey, 8.1× n8n)
and on completion (21–25× p50). Steady state is another story:

- misskey: A is ahead on every row, but only 1.1–2× on session CPU,
  diagnostics after an edit, hover and memory.
- n8n: A ties Volar on diagnostics after an edit (447 vs 496 ms p50) and on
  memory. It is behind on session CPU (36.0 vs 19.5 CPU-s), CPU per edit (1.13
  vs 0.56 CPU-s) and hover p99 (188 vs 87 ms).

A has no Vize-side preparation left on an edit: on n8n, tsgo spends 93% of the
session CPU and the mapper process 7%. What remains is tsgo's own work on the
projected code: re-checking the edited SFC's virtual file and printing large
inferred types for hover. So 10× on per-edit diagnostics, CPU and hover depends
on projection size. That is a shared typechecker/LSP goal: measure per-SFC
projected bytes against the edited file's tsgo check time, then slim the
largest constructs first.

Backend B, today's default, is 18–220× slower than Volar per session. Its edit
cost is Vize-side preparation that grows with the reachable graph (see the
traced critical path below). C gets its harness row in #11651.

## Where B stands (main after #11571)

| misskey `MkNote.vue`             |                    vize lsp (B) |        Volar |
| -------------------------------- | ------------------------------: | -----------: |
| session CPU (3 × 5 cycles)       |                     193.5 CPU-s |   10.8 CPU-s |
| peak PSS                         | 8168 MiB (tsgo 6088, vize 1894) |     2501 MiB |
| diagnostics after edit p50 / p99 |                  2488 / 2645 ms | 262 / 320 ms |
| hover p50 / p99                  |                   199 / 2194 ms |   10 / 46 ms |
| references p99                   |                        23846 ms |       331 ms |

| n8n `ResourceLocator.vue`        |    vize lsp (B) |        Volar |
| -------------------------------- | --------------: | -----------: |
| session CPU (3 × 5 cycles)       |      4364 CPU-s |   19.7 CPU-s |
| peak PSS                         |       22325 MiB |     5628 MiB |
| diagnostics after edit p50 / p99 | 9701 / 13988 ms | 503 / 554 ms |
| hover p50 / p99                  |  198 / 42146 ms |  9.5 / 81 ms |

Sources: run 41633245394 (misskey) and run 41620021649 (n8n).

### Critical path of one edit in B (traced)

Each row is time since the edit's type pass started. Source: run 41629280780
with `VIZE_TRACE_EDITOR_PREPARATION=1`.

| misskey (2125 ms total)                     |            | n8n (8483 ms total)                           |            |
| ------------------------------------------- | ---------: | --------------------------------------------- | ---------: |
| alias fingerprint + cache miss              |      19 ms |                                               |            |
| `alias_reachable_dependencies`              |     767 ms | `alias_reachable_dependencies`                |    3405 ms |
| `alias_package_routes`                      |     355 ms | `alias_package_routes`                        |    1610 ms |
| rest of `alias_build`, stamp, materialize   |     596 ms | rest of `alias_build`                         |    2393 ms |
| `vue_project_build` → `project_synchronize` |     111 ms | `vue_project_build` ×2, `project_synchronize` |     772 ms |
| `editor_mirror` + readiness                 |     106 ms |                                               |            |
| **tsgo diagnostics**                        | **170 ms** | **tsgo diagnostics**                          | **303 ms** |

So 92% (misskey) and 96% (n8n) of an edit is preparation the problem does not
need. Every keystroke rebuilds a whole `VirtualProject` for the edited SFC's
reachable graph, because the alias context's cache key hashes the full host
text (#11595). That cost grows with the reachable graph: exactly the
O(project) step the scaling target forbids. Requests pay a ~200 ms floor of the
same preparation; `vue_project_build` averages 391 ms over 69 calls, while
tsgo's own hover averages 18 ms.

Memory has the same shape. tsgo holds 6.1 GB on misskey because the editor
transport keeps two programs, one per URI identity (#11586), over virtual code
that grows with the SFC graph (`MkNote.vue` projects to 192 KB with 1061
mappings). The Vize server holds 1.9–5.8 GB of mirrors, virtual documents and
alias-context `VirtualProject` copies.

## Backend A measured: tsgo 7.1.0-beta `--lsp` + Vize content mapper

An edit needs two steps: re-map the one edited SFC, then let tsgo check it
incrementally. Batch `tsc --runExternalCode` with the Vize content mapper
already works that way. TypeScript 7.1's `tsc --lsp` loads the same mappers
(`runExternalCode` initialization, `contentMappers` in tsconfig,
`custom/setContentMapperContributions`). 7.0.2 has none of these.

Blacksmith, misskey `MkNote.vue` (run 41633245394, 3 rounds × 5 cycles):

|                                  |            A |       B (main) |        Volar |  A vs Volar |
| -------------------------------- | -----------: | -------------: | -----------: | ----------: |
| session CPU                      |    6.3 CPU-s |    193.5 CPU-s |   10.8 CPU-s |        1.7× |
| peak PSS                         |     1274 MiB |       8168 MiB |     2501 MiB |        2.0× |
| open → diagnostics               |       536 ms |        4868 ms |      4149 ms |        7.7× |
| diagnostics after edit p50 / p99 | 136 / 175 ms | 2488 / 2645 ms | 262 / 320 ms | 1.9× / 1.8× |
| hover p50 / p99                  |  9.5 / 32 ms |  199 / 2194 ms |   10 / 46 ms | 1.1× / 1.4× |
| completion p50 / p99             | 0.4 / 5.8 ms |   199 / 303 ms |   10 / 30 ms |  25× / 5.2× |

n8n `ResourceLocator.vue`, same run:

|                                  |            A |         B (main) |        Volar |    A vs Volar |
| -------------------------------- | -----------: | ---------------: | -----------: | ------------: |
| session CPU                      |   36.0 CPU-s |       4315 CPU-s |   19.5 CPU-s |         0.54× |
| CPU per edit (median)            |   1.13 CPU-s |       0.67 CPU-s |   0.56 CPU-s |         0.50× |
| peak PSS                         |     5749 MiB |        22223 MiB |     5624 MiB |         0.98× |
| open → diagnostics               |      1955 ms |         13424 ms |     15760 ms |          8.1× |
| diagnostics after edit p50 / p99 | 447 / 748 ms | 10378 / 14575 ms | 496 / 749 ms |   1.1× / 1.0× |
| hover p50 / p99                  |  16 / 188 ms |   283 / 48899 ms |   12 / 87 ms | 0.75× / 0.46× |
| completion p50 / p99             | 0.4 / 0.6 ms |     165 / 609 ms |  8.2 / 50 ms |     21× / 83× |

On n8n the tsgo process itself spends the edit CPU: 38.9 of the session's 41.9
CPU-s against 3.0 for the mapper process.

A has no mirror, no session directory, no second program and no per-revision
project copy. tsgo calls the mapper for changed files only.

Accuracy gaps of A on this run. They are fixed in the shared content mapper,
which also improves batch checking:

- definition on a component tag returns nothing (#11635);
- references and rename on a setup binding miss its template uses (#11636);
- hover on a component tag shows generated `__vize*` members (#11637);
- template hover on a `const` binding prints `var` (#11638).

## Structure

```
editor ──LSP──▶ vize lsp
                 ├─ Vue layer (one): template directive/tag/prop/event/slot completion,
                 │  HTML/CSS, SFC structure, patina lint, glyph format, code actions,
                 │  Musea, router/i18n/Pinia
                 ├─ request/merge policy (one): which requests go to the backend,
                 │  how backend and Vue-layer answers merge, versions and cancellation
                 └─ TypeBackend (one interface: open/change/close, diagnostics, hover,
                    completion(+resolve), definition family, references, rename,
                    signature help, semantic classification, code fixes)
                      ├─ A: TypeScript native ≥ 7.1 `tsc --lsp` + Vize content mapper
                      ├─ B: TypeScript native 7.0 through Corsa
                      └─ C: the project's TypeScript 5.7–6.x tsserver + plugin
```

**One projection.** All three backends consume Canon's content-mapper
projection: the same virtual code, spans, feature masks and directive table. A
reaches it through the mapper process tsgo spawns. C reaches it through the
napi projection the plugin already uses (#11538 aligned the directive table).
B gets it by moving its editor codegen onto that projection. A mapping or
accuracy fix then lands once for every backend and for batch checking.

**Selection.** The backend is chosen once per workspace folder at
initialization, from the project's own TypeScript: ≥ 7.1 with content-mapper
support → A, 7.0 → B, 5.7–6.x → C. The setting `vize.typescript.backend`
(`auto` | `content-mapper` | `corsa` | `tsserver`) overrides the choice. The
choice and its reason are logged and shown in the status item. It never
changes mid-session: a TypeScript change in the project asks for a restart
instead of switching silently.

**Project TypeScript.** A and C run the project's own TypeScript, so their
semantics match what users run, as Volar's do. A prefers the project's
TypeScript native when it is ≥ 7.1 and otherwise uses the 7.1.0-beta pin Vize
ships. B keeps the 7.0.2 pin.

**Parity gate.** Every maestro LSP test and every LSP response fixture
(`tests/_fixtures/differential/lsp`) runs against all three backends in CI with
identical expected results. A backend-specific expectation is a defect in that
backend, never a second fixture.

**Targets apply to each backend.** Each one must reach ≥ 10× Volar and flat
scale curves. For B this makes #11595 (the alias context rebuilt per
keystroke) and #11586 (duplicate programs) defects to fix, whatever A does. C
is measured against Volar on the same harness. It is the like-for-like
comparison, since both use the project's tsserver.

**Composition.** `vize lsp` is the one server the editor talks to, in every
editor. For A it runs `tsc --lsp` as a subprocess; for C, the project's
tsserver with the plugin; for B it keeps the in-process Corsa session. The
forward hop for A and C is measured in its first PR and must stay below 1 ms
p50. VS Code can still use TypeScript native preview's own content-mapper
discovery (`editors/vscode/src/content-mapper-discovery.ts`). The default
extension path is the proxy, so behavior and merge policy match in every
editor.

## Supported TypeScript range

| Backend | Range                    | Why this is the floor                                                                                                                                     |
| ------- | ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A       | ≥ 7.1 (7.1.0-beta today) | `tsc --lsp` content-mapper support first ships in 7.1; 7.0.2 has none.                                                                                    |
| B       | 7.0.x                    | Corsa's pinned runtime (7.0.2).                                                                                                                           |
| C       | 5.7–6.x                  | 5.7 is the lowest TypeScript the plugin's CI qualifies (`VIZE_TYPESCRIPT_PLUGIN_TYPESCRIPT=5.7.3`, #10680). Lowering it needs a CI lane at the new floor. |

The lowest supported version overall is **TypeScript 5.7**.

## TypeScript 7.1 pin

- `vize check` stays on 7.0.2 until 7.1 is stable.
- A ships behind the backend setting on 7.1.0-beta. `auto` picks A only when
  the project's TypeScript native is ≥ 7.1.
- The editor path joins the content-mapper conformance workflow, with one LSP
  session per backend against the qualified host revision.
- Risk: the beta's content-mapper and LSP protocol may still change. The
  conformance workflow pins and qualifies the host revision, so a protocol
  break fails there before a release.

## What stays and what is deleted

Every backend stays. What goes is the duplicated projection and session
machinery inside B:

- B's own editor codegen, once B consumes the shared projection;
- the per-revision alias-context `VirtualProject` (#11595);
- the second program per identity (#11586);
- request-view switching (already gone in #11571).

The per-feature request mapping in `crates/vize_maestro/src/ide/corsa_support`
(10675 lines) and `ide/diagnostics/corsa` (3399 lines) moves behind the
`TypeBackend` interface as B's implementation. The Vue layer stops knowing
which backend answers.

## Order of work

Every PR is measured on Blacksmith against Volar, per backend.

1. Harness rows: B, Volar, A (#11623), and C (tsserver + plugin, #11651). Scale
   fixtures (1k/5k/20k SFCs, a project-reference monorepo) with size curves.
2. One projection for B. In parallel: the mapper accuracy gaps (#11635–#11638)
   and projection size against tsgo check time. Both land once for every
   backend.
3. The `TypeBackend` interface in maestro, with B behind it and no behavior
   change. The parity gate starts here, with B only.
4. Backend A through the interface, then backend C. Each joins the parity gate.
5. Selection by project capability, the override setting, logging and status.
   `auto` becomes the default once A and C pass the gate.
6. B to target: #11595 and #11586, measured on the same curves.

Done means every backend is ≥ 10× Volar on misskey and n8n for session CPU,
diagnostics after an edit, hover/completion/definition p99 and peak memory,
and flat on the scale curves.
