# Cross-file CLI rule configuration

Issue: [#7935](https://github.com/ubugeeei-prod/vize/issues/7935).

The CLI's project passes currently bypass the resolved per-file linter config.
Pass their new diagnostics through the existing resolved rule map before merging
into ordinary lint results. This retains declarations and entry matching exactly
as resolved for single-file lint, without another config load or source parse.

Accept the four existing route-typing IDs, the \`cross-file\` group and the existing
60 published Croquis codes in config validation. Expose the four project route
rules through \`getPatinaRules()\` with no single-file preset membership. An explicit
individual code overrides its group; an explicit rule/group overrides category
warning/error. Category off keeps the ordinary disabled-category precedence.
Route findings follow \`ecosystem\` and the existing \`suspicious\` category; Croquis
findings follow \`cross-file\`. Do not reclassify every Croquis finding into a
coarse category. Keep default diagnostic IDs, messages, spans and defaults.

The fixture retains the whole reported issue/config and authored direct-router
controls for all four route diagnostics plus an unmatched inject. Real CLI tests
cover rule off/warn/error, full JSON field conservation, group and individual
code controls, category severity/disable, ordered entry resolution and exit
status. Exact-head Actions, protected merge qualification and released inclusion
remain required; local source preparation is not execution evidence.

Existing merged #8070 and #8091 already repair script browser positions and deep/
slotted CSS subjects. Their incomplete public/original invocation proofs and the
separate global-selector proposal are tracked independently; this change does
not claim those issues completed.
