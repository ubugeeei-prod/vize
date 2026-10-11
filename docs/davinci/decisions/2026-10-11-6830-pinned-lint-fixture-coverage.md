# Preserve original sources in full App E2E lint

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Full App E2E [38111801880](https://github.com/ubugeeei-prod/vize/actions/runs/38111801880)
at `605429c660b328bb03bec5bbfef15961e59fcaf0` failed its original
`lint:all` job `114388842067`: five stale whole snapshots and four real
configuration-loading failures. Release 0.445.0 was retired; this evidence
does not qualify a later release. All original suites, cases, source pins,
patterns, managers, rule presets and budgets remain required.

## Configuration dependencies

Lint now uses dedicated, authenticated source exports. Browser fixture
rewrites remain confined to their existing consumers. Setup verifies the
owning Gitlink, every original file's complete Git blob, its file type and
Unix executable mode before and after lint. Nested Gitlinks are authenticated
and exported too; an empty submodule cannot inherit its parent's identity.
Environment commands bind the native CLI installed by `setup-vp` directly,
since package scripts prepend a local `vp` that lacks `env`. The first
source run [38115298910](https://github.com/ubugeeei-prod/vize/actions/runs/38115298910)
proved this setup defect: eleven cases passed, while all four preparations
failed before install. The corrected CLI path and bytes join runtime receipts;
manager versions, Node runtime and complete fixture inputs stay explicit.

| Fixture  | Original pin                               | Authored pnpm | Effective config Vite          | Setup                                                                                 |
| -------- | ------------------------------------------ | ------------- | ------------------------------ | ------------------------------------------------------------------------------------- |
| Naive UI | `b26496a26536918204722cb9b9e5dd9bfe31c893` | 11.10.0       | 8.3.4                          | Upstream has no lock; commit a generated frozen lock bound to the original manifests. |
| VOICEVOX | `0a7fcc8523410ad294b51b70a46575c4d21371ac` | 10.28.2       | 8.0.13                         | Original frozen lock, authored Electron installation and `.env.example`.              |
| Misskey  | `810faa8e5db4bbf9d6d408a61df148559340ee99` | 10.30.1       | 7.3.1                          | Original frozen workspace, authored i18n build and `.config/example.yml`.             |
| npmx.dev | `c9e8be91e61eeeae956e2068722caa09e293cc68` | 11.20.0       | Authored vite-plus 0.2.7 graph | Separately reviewed frozen dependency migration described below.                      |

The complete original factories pass the exact embedded native configuration
script at Carton producer `4262809614210a9531c585c1e40db4f42275206e`.
Naive/VOICEVOX/Misskey return the whole projection
`{"typeChecker":{"jsxTypecheck":true}}`. npmx returns
`{"typeChecker":{"jsxTypecheck":true},"formatter":{}}`. No factory is
skipped, evaluated a second time by setup, or replaced with a stub. Original
configuration errors continue to propagate. The Carton producer change is a
separate prerequisite; installing dependencies alone does not repair native
JSON/extensionless TypeScript/`.js`-to-`.ts` module loading.

### npmx dependency migration

The original integrity-pinned Lunaria preview URL
`https://pkg.pr.new/lunariajs/lunaria/@lunariajs/core@904b935` returns 404.
Its original source commit is `904b9353f5002a7278d23ba5464c8ef106e21024`;
the successful preview publication no longer has a recoverable Actions artifact.
The original package manifest, lock and integrity remain explicit inputs and
the original frozen install remains a failed observation.

The mutable browser helper's Lunaria 0.1.1 replacement was rejected because
it lacks `createLunaria`, which the original authored consumer imports.
Official stable 0.2.0 provides that API. Its registry tarball integrity is
recorded in the sidecar. The complete original `lunaria/lunaria.ts` entry
successfully generated its real dashboard and status using the original
configuration and 2,103 authenticated source-history commits. This verifies
the authored API use; it does not reconstruct the missing preview's bytes.

Only a separate installer export changes `@lunariajs/core` to exact 0.2.0.
Its frozen lock preserves every other original importer/package/snapshot
entry and adds eight necessary nodes. Original patches, settings, catalogs
and overrides remain intact. The original lint source export keeps the
original package and lock byte for byte and attaches the installed graph.
Both exports are verified, including their full remaining source files.

The original and corrected native producers return exactly identical whole
configuration stdout, exit code and stderr on this migrated graph. The
effective lock SHA256 is
`6cdfe35228adac418da3832edec9a3c3a82430202799fadb55d87e9466ed46c5`.
Receipts explicitly state `originalFrozenClosure: false`; neither this
migration nor Naive's newly generated lock is original upstream lock evidence.

## Whole snapshot review

The original artifact `11691797306` ZIP has SHA256
`d792e1ab18405030b693aae1bd75ad22b485ed0ec1bee22ecb24bae67d01298b`
and exactly six entries: its lint log and five `.snap.actual` files.
The job log has SHA256
`1296699948acf980fa2caf974cb0391bc5c7979bc78ccdb4ad0c92a83f12065b`.
The new snapshots copy complete authenticated provider bytes after reviewing
all 1,977 ordered file identities and all 395 changed messages; no field,
severity, help, coordinate, file or rule is stripped. 1,661 file packets
are identical and 316 changed. Every owning fixture pin matches the pin at
its preceding snapshot commit.

| Existing policy                         |                                 Whole messages reviewed | Owning source change                                                                                                      |
| --------------------------------------- | ------------------------------------------------------: | ------------------------------------------------------------------------------------------------------------------------- |
| Duplicate default img-alt alias         |          10 removed; original alt-text packets retained | `1d7ff3b2b05e1e93a68d37c291d1683e653df5ba`                                                                                |
| Unsafe URL limited to active sinks      | 88 non-active image/media/component diagnostics removed | `85cc05b89b5b16b8d302cd44ca124bab2d17c9ea`                                                                                |
| Physical filename casing                |                                    183 added, 6 removed | `341fd1ba1d28391ed9973f2675727e27c3d3c731`                                                                                |
| Lexical/direct-call SSR globals         |                                    64 removed, 29 added | `bfb7c96b81041903f4feb3164433fde717fc45b9`, `c30f1a02da763e0086c489b8e17b9aa558187cff`                                    |
| Unused-component definition coordinates |           3 removed and 3 added, other fields identical | `00bceccf4baa4fe46eb395673677fc01ef1be197`                                                                                |
| Used slot variables                     |           3 removed; original event call sites retained | Complete original source witnesses; no single later collector commit attributed                                           |
| Keyed conditional template roots        |                                               1 removed | `c0e86a7d0eaa5ed440405d67b6aa01fcf00aa51e`                                                                                |
| Explicit ref factory imports            |                                               2 removed | `1b7c6d88b0be6e9036a17a75b0d5b2fc8a5c5edc`                                                                                |
| Deliberate SFC table-wrapper exemption  |                                               1 removed | `bfb7c96b81041903f4feb3164433fde717fc45b9`                                                                                |
| Nonfocusable separator ARIA contract    |                                               1 removed | `8f6f963b0cc0a78987ac84ae35960b90d0504fa5`                                                                                |
| Unused MenuItemProps.id                 |                                                 1 added | Shared collector `094739fbcda25f3b4c41d03c72f04a6a1242aa2d`; full source proves no use, no single later commit attributed |

The strict casing policy intentionally diagnoses dotted `.story`/`.client`
physical filenames and underscores. It does not have an implicit story-file
exemption. The SSR policy has lexical/direct-invocation bounds: external
`isClient` bindings, arbitrary interprocedural flow and full Options API/SSR
execution are not proven by this corpus. A runtime constructor such as
`node: HTMLElement` is not a type annotation. The SFC table-wrapper exemption
is deliberate policy, not evidence that arbitrary invalid tables are valid.

All 316 changed source files were also captured with the original explicit
51-rule adoption projection and official Vue base, using ESLint 10.4.1,
eslint-plugin-vue 10.9.2, vue-eslint-parser 10.4.1 and TypeScript parser 8.65.0.
All providers loaded and all 316 complete captures succeeded. Whole raw
responses, serialized packets, options, versions and producer hashes are
retained. Official Vue confirms the used-slot/key witnesses and the three
unused components. Native warning severity and official adoption error
severity are separately preserved. Native SSR/security/a11y/physical-filename
policies have no identical rule in this 51-rule projection; this capture
does not claim full native/official rule equivalence.

Durable evidence lives in
`tests/_fixtures/_projects/lint-review/2026-10-11/`. Gzip files contain complete
JSON/NDJSON, not summaries with omitted message fields. `lint-realproject-review`
checks their complete packets, original provider bytes, file ordering and
policy accounting. The configuration dependency helpers emit runtime receipts
under `.vize/artifacts/lint-fixtures/` for the existing App E2E artifact upload.

## Required follow-through

Run the original full lint suite and all required App E2E rows on the composed
new source head after the Carton prerequisite merges. Passing these retained
local source/config/oracle receipts alone does not admit a release. Keep the
separate original npmx route-metadata failure open until its owning fix and
original consumer acceptance pass; this change does not edit that expectation.
