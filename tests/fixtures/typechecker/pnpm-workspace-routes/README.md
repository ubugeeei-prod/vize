# Workspace package route regression

These five authored source files, manifests, tsconfig and raw relative links
preserve the reproduction in Vize issues #6982 and #7834. `links.json` records
both the root-link case and the additional legitimate pnpm per-package links.
Tests create those links without flattening the workspace or replacing package
exports with tsconfig aliases.

The package manifest preserves the reported Vue 3.5.38/TypeScript 5.9.3 request.
The Actions oracle uses the repository's installed Vue type provider and pinned
Corsa runtime; it does not claim to install those historical package versions.
Typed component/event and delete/recreate variants are separate regression
controls, not literal issue input. This fixture makes no performance claim and
does not address the unrelated Pinia TS2322 errors also mentioned in #7834.
