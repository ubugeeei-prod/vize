# Documentation command choices

Tracks [#8366](https://github.com/ubugeeei-prod/vize/issues/8366).

Enhance rendered shell examples in every locale with the requested order:
`vp`, `npm`, `pnpm`, `yarn`, `bun`, `aube`, `jsr`. Keep the original server-rendered
code readable when JavaScript is disabled. Use keyboard arrow/Home/End selection,
one visible panel, an announced copy result, and a persisted manager choice.

Translate the operation, not just the executable name. Installing a dependency,
running an installed binary, fetching a one-off tool, and running a project script
are different operations. Vite+ tasks retain `vp run` through the selected
manager's local binary runner because the tasks need not be package.json scripts.

[Aube's documented add command](https://aube.sh/cli/add.html) and
[script/binary runners](https://aube.sh/package-manager/scripts) supply its actual
command spellings. Preserve comments and arguments. Workspace-specific flags,
shell programs, and unsupported operations display a notice and the original
command instead of inventing equivalent lockfiles or invocation semantics.

JSR is a registry, not a native CLI runner. Its tab explains unavailable
operations while [#8367](https://github.com/ubugeeei-prod/vize/issues/8367) owns
published package/import support. Do not advertise JSR CLI binaries or substitute
an npm import for an installed JSR consumer proof.

The source tests compare complete independent command vectors and audit the
existing guide/integration fences. The actual generated-site Chromium verifier
checks the five localized setup pages at desktop/mobile widths, no-JavaScript
fallback, all manager panels, keyboard wrapping, clipboard contents, and stored
selection. Source CI, protected merge, docs deployment, and live-site checks remain
required before closing #8366; local pure tests alone do not complete it.

The first generated-site run rejected extra translated code panels in the whole
inline-rule comparison. Retain the exact authored highlighted block in its own
manager panel and identify it explicitly; compare that complete block while the
existing SSR/source packets remain unchanged. A focused Chromium replay over a
separately built site plus these exact enhancement assets preserved all 1,453
inline code blocks across 276 widgets. Five locales passed 30 command checks,
keyboard, clipboard and reload persistence. Full exact-head Docs proof is still
required; this local injection does not establish deployment or publication.
