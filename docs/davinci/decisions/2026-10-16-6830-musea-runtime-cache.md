# Restore Musea runtime and dependency verification caches

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Both Musea jobs now restore the existing Vite+ bootstrap cache before
`setup-vp`, then restore pnpm's verification cache immediately before their
existing filtered install. The actions use their default restore-only mode.
pnpm still validates its own record against the lockfile, pinned version,
policy and registry configuration; a missing or invalid record runs normal
verification. Changes to that cache helper select the product integrations.

## Observed baseline

Both runs below passed before these additional restores. Durations include
workflow startup, the selector and the native job; they exclude no waiting.

| Measurement                              | [`986d6cd`, run 38893209415](https://github.com/ubugeeei-prod/vize/actions/runs/38893209415) | [`307e85b`, run 38896572839](https://github.com/ubugeeei-prod/vize/actions/runs/38896572839) |
| ---------------------------------------- | -------------------------------------------------------------------------------------------: | -------------------------------------------------------------------------------------------: |
| Whole workflow                           |                                                                                307 s (5m07s) |                                                                                233 s (3m53s) |
| Creation to native job start             |                                                                                         50 s |                                                                                         27 s |
| Native job                               |                                                                                        257 s |                                                                                        206 s |
| Vite+ setup                              |                                                                                         29 s |                                                                                          8 s |
| Filtered dependency install              |                                                                                          4 s |                                                                                          3 s |
| Native binding and gallery builds        |                                                                                         81 s |                                                                                         58 s |
| Cargo `ci` compilation within that build |                                                                                         63 s |                                                                                      45.37 s |
| Native browser assertions                |                                                                                         50 s |                                                                                         48 s |
| Hosted CLI and snapshot assertions       |                                                                                         51 s |                                                                                         45 s |

The later run also passed the separately selected Chromium lane. These are
different runs with different source and startup conditions. The table does
not measure the effect of the new cache restores or establish a five-minute
guarantee. Fresh Actions must measure that effect.

All product test commands, source-native builds, fresh native build tokens,
Cargo profiles, cache namespaces, artifacts and limits remain unchanged.
The hosted CLI tests remain sequential: their helper rebuilds the shared
`npm/builder/vite-musea/dist/gallery` with `emptyOutDir: true`, so simply running
the commands concurrently would introduce a race.

## Native cache follow-up

Both logs mounted `ubugeeei-prod/vize-native-inline-browser-target-Linux-X64`
read-only, then reported `Rust source mtimes left as checked out: no manifest`.
Each logged 344 distinct package names being compiled, including 34 Vize
packages. A mounted disk alone does not demonstrate usable compiled artifacts.

TODO: provide a trusted main-push seed for the existing `native-inline-browser`
namespace using the same source-native `build:ci` command, `ci` profile and
`napi,legacy` features. At these revisions only the Musea workflow references
this namespace; its schedule or trusted main dispatch can write it, but there
is no main-push seed. Preserve the source mtime manifest and fresh native
identity checks. Measure reuse before claiming a build-time reduction; the
`test-js-packages` dev-profile binary is not a substitute for this build.
