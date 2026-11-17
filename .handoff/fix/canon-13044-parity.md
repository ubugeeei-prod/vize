# Handoff: #13044 real-project typecheck parity (agent p0-13044)

This branch is a superseded local WIP snapshot: every change in it already
landed through #13236, #13240 and #13241. It is kept only for this note.
Do not open a PR from it.

## Merged

| PR | branch | merge SHA | item |
| --- | --- | --- | --- |
| #13236 | fix/canon-13044-class-later-static | 8a4e5f6e | Vben 5x TS2322 class fixed; 8x TS2578 + 1x aria documented in tests/_fixtures/compat-documented-differences.json |
| #13240 | fix/canon-13044-installed-file-mirror | af961eee | Mealie 9x TS2307 (vuetify, @nuxtjs/i18n) fixed |
| #13241 | fix/canon-13044-self-package-link | 2de08e47 | self-pointing package link never planned |

Scoped matrix runs:

- Vben 44050066114: typecheck divergence passed, 10 vize diagnostics, all 10 documented, 0 FP / 0 FN.
- Mealie 44050242933: typecheck surface success. The only TS2307 left is `~/types/vuetify`, which plain tsc also reports.
- File Browser 44051560175: typecheck surface success.

The progress comment is on #13044 (issuecomment-6944517611). The issue was deliberately left OPEN.

## Open: pending user decision

File Browser reports 3x TS2339 `Property 'fromNow' does not exist on type 'Dayjs'` in
ListingItem.vue:130, Share.vue:379 and settings/Shares.vue:153. vue-tsc reports 0 on
`frontend/tsconfig.app.json`. The matrix has no vue-tsc comparison for File Browser,
so its verdict passes anyway.

Cause: `src/main.ts` imports `dayjs/plugin/relativeTime`, whose d.ts does
`declare module 'dayjs' { interface Dayjs { fromNow ... } }`. Nothing checked
imports main.ts, and two places drop that augmentation:

1. `crates/vize/src/commands/check/tsconfig_inputs/ambient.rs` `augments_modules` keeps only
   sources that augment directly, so main.ts is not a program root.
2. `crates/vize_canon/src/batch/executor/cli/partition.rs` (`declares_program_wide_types`) shares
   only directly augmenting sources across CLI shards, so a root main.ts can land in another shard.
   Probes confirmed this: `vize check zz-use.ts zz-setup.ts` still errors, and a .vue file plus
   main.ts in one shard is clean.

Proposed fix: treat a source whose bare imports resolve (for example via
`type_references::resolve_type_reference_declaration_files_reading`) to a declaration
that augments a module as program-wide in both places. Cache resolution per
specifier. Measure cold-pair perf, because many main.ts files import vue-router or
pinia, which augment `vue`. The user decides whether to do this.

## Other remaining items

- Mealie run status stays `configuration` (`tools/support/fixtures/typechecker_qualification.rs`
  treats any TS2307 as setup failure), because of the genuine `~/types/vuetify` import.
- Vben with an isolation overlay lacking `#/*` now fails on a different overlap. An owned
  installation `.../form-ui/node_modules/@vben-core/composables/node_modules/@vben-core/shared`
  sits under a whole-directory link `.../composables/node_modules` -> `packages/@core/composables/node_modules`.
  The real matrix config never reaches it.
- vize does not check static attribute values on native elements (`<input aria-expanded="visible">`),
  while vue-tsc does. This is a separate false negative.

## Gotchas

- Typechecker `manifest.json` cases must stay sorted by `pack ?? id`
  (`differential-dialect-coverage.test.ts`).
- `workspace_aliases.rs` sits near the 500-line limit.
- Local repro needs corepack: `npm i -g corepack --prefix <dir>`. vite-plus then auto-links a global
  shim into `~/.vite-plus/bin`; remove it.
- The typecheck prep script needs `GITHUB_SHA` and an existing `--output-dir`.
- Matrix project shards (shard count 150): vue-vben-admin 0, filebrowser 30, mealie 50.
- On macOS, set `CORSA_PATH=node_modules/@typescript/typescript-darwin-arm64/lib/tsc`.
- Monorepo-5000 cold-pair cells are very noisy (single runs 9-33 s). Judge by vue-1000 and by the
  vize syscall counts.
