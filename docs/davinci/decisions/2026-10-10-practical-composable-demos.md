# Practical composable documentation demos

Tracking: [#8374](https://github.com/ubugeeei-prod/vize/issues/8374),
[#6101](https://github.com/ubugeeei-prod/vize/issues/6101).

## Decision

Keep the existing source-owned UI basic examples and add six complete Vue SFC
examples beside the composable package: disclosure, bounded quantity, debounced
local search, blur validation, undo/redo, and offset pagination. Each demonstrates
an observable purpose, relevant transitions, public imports, and accessible
native controls. Every UI/composable entry gets minimal installation and project
context; the English/Japanese composable hubs link the practical examples.

The generator displays the entire same SFC packet that the preview compiles.
SHA-256 binds its source to the live iframe. The preview build typechecks these
SFCs against the current workspace source, compiles them with the existing UI
Vue/Vite compiler configuration, renders them through Vue's server renderer,
and hydrates the matching source in Chromium. SSR and browser builds use fresh
compiler-plugin instances; a dev-server plugin instance must not be reused by a
production build. Hydration must preserve all element identities and the full
initial markup, with no browser warnings or errors.

The browser checks assert all six examples' actual initial/interacted behavior
at 720px and 360px, including Enter/Space, keyboard blur, boundaries, delayed
search, flush/cancel, error/valid/reset, batching, redo invalidation, and page-size
clamping. Virtual browser time makes the real debounce timer deterministic.
Twenty-four actual PNGs and their hashes accompany the source hashes and
individual interaction receipts. These captures are illustrations, not VRT
baselines or a complete accessibility audit.

After SSG, the existing docs acceptance entrypoint checks sixteen rendered
English/Japanese routes and local assets, exact complete displayed source,
iframe source/hydration identity, and the same six interaction laws. It supports
`--site` for the deployed site. Actions runs these checks in the existing Docs
build, with strict preview TypeScript inside the existing Check job; no new
workflow job or product pipeline stage is added.

## Delivery and remaining acceptance

- Local observations are not hosted Actions, protected merge, or deployment
  receipts. Obtain fresh exact-head Docs and Check runs, actual queue merge,
  and deployed browser acceptance before calling this slice delivered.
- Live interaction checks cover these six composable entries and the existing
  eight featured UI families. The other source-generated entries remain outside
  those interaction checks. Their complete practical/state coverage is still
  required by #8374; keep that issue open until its full acceptance is met.
- #6101 remains open for the broader compile/typecheck/lint/format/render,
  unsupported-claim/default coverage, tooling/editor metadata, and verified
  release-note matrix across all named products.
- No upstream n8n or provider writes are part of this change.
