# Original event rename qualification

The event reports [#8010](https://github.com/ubugeeei-prod/vize/issues/8010)
and [#8011](https://github.com/ubugeeei-prod/vize/issues/8011) share a bound
`defineEmits` component whose `change` key must link to `emit("change", true)`
and its parent's `@change` listener. Existing static event-form native controls
exercise declarations and listeners, but omit that original bound emit call.
Their successful execution does not qualify the complete original transaction.

Preserve both complete reports and the literal Toggle/App source blocks in
`tests/_fixtures/differential/lsp/event-rename/`. Independently authored repaired
files change exactly those three event names to the reported `update`. Six real
stdio sessions cover declaration, call, and parent origins with LF/CRLF, retaining
the reported key `2:3` and call `6:9` cursors. Each compares complete references
and WorkspaceEdit values, applies actual edits to all original files, checks
disk bytes and empty version-2 diagnostics, then independently installs the
goldens and checks complete empty version-3 diagnostics. No response filtering,
deduplication, library allowance, or generated-coordinate substitution is used.

Use the existing mandatory pinned-native CLI fixture and retain its actual
configuration and process identity. The original reports describe configuration
options without literal config bytes; the helper's project-root Vue layout is
an explicit harness choice. All previous original/native event, model, alias,
shadow, whole-transaction refusal, and source-ownership assertions are retained.
Production stays unchanged until actual hosted execution demonstrates a defect.
If a repair is needed, retain its failed full packets and correct only the proven
producer/ownership path after coordinating shared dispatchers with the slot lane.

Current source Actions, required native execution, fresh protected gates, signed
actual delivery, and public release replay remain unqualified at preparation.
This event slice does not complete #8010 configured-runtime assets physically
inside the authored root or bare-script exporter routing, #8011 slots, or the
stock unequal camel/kebab Content Mapper contract in
[#4075](https://github.com/ubugeeei-prod/vize/issues/4075). Upstream is read-only;
Vize bridge success must not be presented as stock protocol acceptance.
