# Original Oxlint path scope

Issue: [#7903](https://github.com/ubugeeei-prod/vize/issues/7903).

The original wrapper moves every Vue file before Oxlint evaluates ignore rules
and per-file overrides. The qualified explicit JSON route already selects files
with Oxlint on their original paths and projects ordered overrides into an owned
mirror. Extend that route to discovered root JSON and TS/MTS configurations.

TS/MTS configuration uses the same native Node import and JSON serialization as
Oxlint. Preserve object `extends` rather than flattening settings, ignores or
rules ourselves, and project each inherited override in its existing order.
Keep the borrowed module beside its owner so plugin resolution retains its
namespace. Verify the entire projected file set with the actual engine before
linting, then restore source spans with the existing location mapping.

Retain the reported SFC and independent inherited-config controls in
`tests/_fixtures/differential/lint/oxlint-original-path-scope-7903`. The real CLI
controls compare complete core diagnostic objects with stock Oxlint and require
scriptless template positives, directory and filename overrides, VCS/config/CLI
ignores, immutable authored bytes and cleanup of every owned temporary file.

This first slice does not qualify nested configuration or relocated import and
type-aware project graphs. Discovered nested configurations fail with an
actionable explicit-config instruction. The n8n editor config enables the import
plugin and type-aware rules, so adoption acceptance remains open: core and custom
rules must execute on their original files while the Vize-only bridge supplies
template diagnostics. JSONC and other engine versions need separate evidence.
No upstream state is changed, no native level is admitted and no timing result
is claimed. Exact-head Actions, unchanged merge-queue gates, actual merge and
installed release replay remain required.
