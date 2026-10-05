# Nuxt Oxlint configuration root (#7983)

## Problem and boundary

The complete original report reproduces `../**/dist` in the default generated
`.nuxt/oxlint.config.json`. Oxlint rejects parent segments in global ignores;
removing ignores leaves the parent-relative overrides unmatched. This is the
regression of #7251, not a compiler or native-linter semantic change.
The [paired Issue decision](https://github.com/ubugeeei-prod/vize/issues/7983#issuecomment-5991355930)
records the original corpus and mandatory execution gates.

The [Oxlint configuration contract](https://oxc.rs/docs/guide/usage/linter/config-file-reference)
roots ignores at the config directory and cannot match files outside it.
Pinned Oxlint1.78 `config_store.rs:118–136` uses a config-relative path inside
that directory and the actual absolute path outside it for overrides. A blanket
`**/` prefix would broaden overrides while leaving outside global ignores broken.

## Selected repair

Generate `.oxlint.vize.json` in the Nuxt project root by default. Keep the root
plan globs unchanged when both directories coincide, and prepend only the exact
relative plan-root namespace when the config is in an ancestor. Outside-layer
overrides/exclusions use their actual absolute paths, matching Oxlint's original
path selection. Outside global ignores and below/outside-root config locations
are rejected before writing an invalid artifact. This explicit migration
replaces silently ineffective settings; it does not widen the lint target set.

The reserved default file contains the recognized settings marker
`settings.vize.generatedBy: "@vizejs/nuxt"`. Exclusive initial creation cannot
replace an authored collision; later writes require the exact ownership marker
and a regular file. Malformed, unowned and symlink collisions remain unchanged.
Ownership follows the resolved reserved root path even when explicitly named
with a relative or absolute option; peer review identified and closed the
[option-presence bypass](https://github.com/ubugeeei-prod/vize/issues/7983#issuecomment-5991453040). Other explicit custom files retain the existing
regular-file/atomic write contract and are unmarked. Add the generated file
to the project's `.gitignore`.

Auto-init still preserves existing project/ancestor Oxlint and Vite configs.
Only the complete exact former generated default `oxlint.config.mts` loader
can migrate automatically to the new artifact; any authored edit or symlink
is preserved. A custom ancestor config with a new loader needs `autoInit: false`
so that a root loader cannot reinterpret the config's different glob base.
Existing explicit plugin specifier URL resolution remains unchanged.

## Original source and execution gates

The new linter differential corpus retains the whole original Issue body
SHA256 `b8f105a611cf70a62e9e10526592c5dcb53c7ac018e65ceb4a618cfdb5637769`,
renderer, failed JSON, two original paths and identical full SFC
SHA256 `aacf0733b25709e6b813e6af1909e701762d80c2aaa5e7c6f8f0dcf8f4548ff0`.
The `.vue.txt` is storage only: the mandatory automatic recipe copies these
exact bytes to physical `.vue` inputs and requires authentic source-NAPI calls.
Original upstream whole-artifact recordings and all unrelated assets stay
unchanged; current default generation adds only the explicit ownership marker
to the original root-relative plan, with the incorrect rebasing removed.

The existing Nuxt3 automatic job builds the source native addon, current Nuxt
integration, lint plan and Oxlint bridge. It retains the old compiler
client/SSR/browser controls and #7999 manifest/prefetch packet, then uses the
unchanged Nuxt3.19.3 and Nuxt4.5.2 dependency cohorts for finite lint probes.
The unchanged workspace Oxlint is1.78.0; reporter1.75/1.86 are not separately
executed or credited. Each cohort runs actual module default generation and
seven CLI cases: default, original startup failure, original unmatched
overrides, original root renderer, page exclusions, ancestor namespace,
and outside-layer/neighbor controls. The old invalid config remains a failure.

Full raw configs, CLI process status/stdout/stderr/reports, current whole package
dist files and source-NAPI load/input/options/return events are retained.
Complete native inline-style results pin the original message/help and authored
attribute span76–94, line6 columns7–25. Full CLI diagnostic identity/message
and exact visited-file/warning/error counts enforce ignores and ordered
overrides; nondeterministic timing fields are retained without speed claims.
Close Nuxt before sealing native events and remove only the probe's borrowed
dependency links/project after its finite child completes. Original projects
and lockfiles remain unchanged. Nuxt2's old root-placement workaround is
removed so its existing complete process-flag controls use the actual default.

Local configured formatting/lint, original corpus laws and emitter laws pass.
The emitter run includes the existing dummy-JS Oxlint test; it gives no source
NAPI credit. Local generation laws cannot load the absent workspace dist and
remain unexecuted. Private source/recipe peer review, fresh exact-source Actions,
authentic whole packets, unchanged protected100+4/full suites, actual signed
reporter-credited merge and supported publication are still required.
No native/compiler/default-history migration, browser/SSR expansion or
performance result is claimed by this lint-config repair.
