# Full Playground provider hydration

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Paired [issue decision](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6115023842).

## Observed failure

Full Check runs [38182228212](https://github.com/ubugeeei-prod/vize/actions/runs/38182228212)
and [38182228207](https://github.com/ubugeeei-prod/vize/actions/runs/38182228207)
completed dependency installation but failed before browser tests started.
The actual `svg-namespace-corpus-plugin.ts` resolves `vue-vapor-runtime/package.json`
with `createRequire` from `npm/ui/package.json`; the failure is
`MODULE_NOT_FOUND` with that UI manifest as the require stack.

The filtered install selected Playground, Vite integration, CLI and native
workspaces. It omitted UI, which owns the existing `npm:vue@3.6.0-rc.9` alias.
Building and downloading the shared JS artifacts does not hydrate that alias
in the consumer job.

## Decision

Append `--filter './npm/ui...'` to the same full Playground installer.
Preserve the four existing filters, frozen lockfile, installer producer,
Node version, aliases, source pins, build artifacts, browser command, VRT
command, native compiler producer and every browser oracle.

The focused regression runs the actual MoonBit `github/vp_install` command
twice against five isolated tiny workspace manifests. It copies the UI alias
specifier and exact lock entry, all original lock package/snapshot bodies,
and the locked root Node runtime entry without resolving new versions.
The original four filters reproduce the missing UI provider; adding UI
resolves the unchanged provider declarations from the real SVG plugin,
the pinned Vue package and intrinsic compiler, and its actual Vapor browser
runtime path. No Rust/native build or fixture compiler replaces the browser gate.

## Qualification

The two local laws pass with no skips; the original four-filter failure is
retained as an authored negative. The system Nix Moon compiler lacked its
native prelude; verification used the existing repository Moon cache without
changing source or weakening the law.

TODO: require fresh exact-head source and native Actions, the unchanged full
Playground browser and VRT jobs, protected queue checks and actual merge.
The isolated dependency regression alone grants no product, browser, full
repository health or release acceptance.
