# Publish the Vite plugin after its unplugin dependency

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

`npm/builder/vite/package.json` declares `@vizejs/unplugin` as a direct runtime
dependency. The Vite+ pack adapter imports `@vizejs/unplugin/rolldown` when it
configures compilation. Release previously waited for native and CLI publication
before publishing the Vite plugin, while unplugin published on a parallel branch.
A slower unplugin publication could therefore expose a Vite version whose exact
dependency was not yet available to a fresh installation.

Add `release-npm-unplugin` to `release-npm-vite-plugin.needs`. The existing unplugin
publisher verifies both the exact npm version and its dist-tag before succeeding.
This adds no cycle: unplugin depends on native publication, package artifacts,
installed-package smoke, and promotion. Every other dependency and the immutable
candidate, catalog, signed metadata, tag, and publication barriers remain intact.

The existing release-build contract reads the actual two package manifests and
requires this direct publication dependency. It fails before the added edge.
The focused workflow checks validate the repaired graph; they do not publish a
version or prove registry timing. The next official release supplies that evidence.
