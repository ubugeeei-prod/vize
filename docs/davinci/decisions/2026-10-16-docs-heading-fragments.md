# Repair encoded documentation heading fragments before asset hashing

The pinned Ox Content 2.88.0 native runtime passes a URL fragment directly to
`querySelector`. Percent-encoded Japanese headings therefore produce an uncaught
`SyntaxError`, even when the browser's native fragment navigation reaches the
heading. This is independent of the documentation's Japanese wording and live
component examples.

Vize temporarily patches both published JavaScript entry points of
`@ox-content/vite-plugin@2.88.0` through pnpm. The integration runs immediately
before the provider's native asset externalization, so the provider still computes
asset filenames from their final bytes and writes their references. It does not
rewrite already hashed files or install a second browser navigation handler.

The replacement first looks up the decoded fragment as an element ID, then its
original spelling. Decoding errors are contained. The provider's existing CSS
selector fallback remains guarded, preserving escaped selectors without allowing
malformed or missing fragments to throw. Only the two lookups inside the owned
anchor handler are replaced. Exact pinned markers, boundaries, call counts and a
helper-name collision check inside the generated core fail the build when that
integration contract drifts. Ordinary documentation mentions are allowed.
Bare SSG output bypasses the repair because it contains no generated runtime.

`check:heading-links` participates in the existing docs build. It verifies the
real site's native asset digest and reference, then builds a small page through
the actual provider in its original, patched ESM and patched CJS forms. Chromium
must reproduce the original exception and pass initial-load, hash-change and
click controls for Japanese, ordinary, percent, space and punctuation IDs, escaped
selectors and missing targets. The build records the browser observations and a
Japanese anchor screenshot in `docs-render-evidence/heading-hashes` alongside the
other existing documentation evidence.

Remove this temporary integration when the pinned provider supplies equivalent
safe anchor lookup, retaining the browser controls. No upstream repository or
published provider release is modified by this change.
