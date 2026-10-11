# Preserve the native config module loader in Nix sources

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The native CLI embeds `module-runtime.mjs` through
`crates/vize_carton/src/config/loader/js.rs:27`. Nix's explicit external-asset
list included the neighboring `vite-runtime.mjs` but omitted this required
module. Full Check at source `68c65d7db19a0cef486901725ccea69cce008848`
failed in job [114429471988](https://github.com/ubugeeei-prod/vize/actions/runs/38125688042/job/114429471988)
while building `checks.x86_64-linux.package`: the embedded file was absent.
The complete original job log is 2,858,496 bytes, SHA-256
`456e51959438051ff3c8185e06c2827409fe96467d299540de2c5b9cbc66508a`.

Add that exact file to `externalAssets` and extend the existing
`embedded-source-assets` check with `cmp` against its complete authored bytes.
Keep the Cargo filter, suffix list, other explicit assets, two existing byte
comparisons and excluded-file checks unchanged.

On current main `6ddbcd1df9167a00035ae4eb543a68ac193ac76d`, adding the
comparison before correcting the producer makes the actual filtered-source
derivation fail because the module is missing. This is the regression check;
it checks the real Nix source output rather than a textual allowlist pattern.
After adding the literal asset, the same
`nix build .#checks.aarch64-darwin.embedded-source-assets --no-link` succeeds,
including both original comparisons and both excluded-file checks.

Local filtered-source validation is distinct from building the Linux CLI.
The existing full Check campaign must confirm the package build on merged
source. Nix runs on scheduled and manually dispatched full Checks; ordinary
PR and merge-queue success does not establish Nix success. No workflow,
dependency, product runtime or publication change is part of this repair.
