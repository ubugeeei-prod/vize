# Lint execution reader flow

Refs: #8365 (More guides) and #8372 (natural Japanese).

## Decision

Explain when to use the existing lint execution settings and provide practical
CLI and cross-file report next steps. Retain the complete original English body
and author the complete Japanese page with its original English heading anchor.
The reviewed-translation marker preserves the authored input on regeneration.

Keep the whole existing Vite+ example, every public option and rule name, native
CLI file selection, explicit rule/scoped-off precedence, warning default and
invalid-negative boundary unchanged. Vite+ forwards only paths and --fix; its
native settings belong to lint.vize rather than Oxlint's options.

Source authority is vize_l0 config/model/linter_execution.rs and linter.rs,
the CLI lint/config_load.rs and lint/entry_rules.rs, and Vite+'s runner.ts and
lint-options.test.ts. Existing source tests cover configuration forwarding,
strict-reactivity rule activation, explicit disable and negative limits.
These source contracts do not establish a new installed release or broader
third-party project acceptance.

## Delivery boundary

Update only the finite /guide/lint-execution More entry. Preserve all other
routes and coverage classification. Native English/Japanese reading, every
original example and source anchor, actual next-step links, and stable public
source qualification remain required before counting this bounded guide as
delivered. The full More and Japanese editorial issues remain open.

Prepared independently while the casing repair was in its protected queue.
Publication follows its actual merge on fresh main, using the real worker/profile
PR head as the native Stack parent. Preserve every incoming foundation clause,
including that parent record. No casing product or browser helper changes.
