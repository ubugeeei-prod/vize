# Project-wide editor navigation (#8013)

The public issue's four complete original sources are retained under
`tests/_fixtures/differential/lsp/project-navigation-8013`. Its described
package/TypeScript settings are made explicit in separate authored controls;
Vue 3.5.43 and native 7.0.2 are the qualified providers. No private project is
used. This legacy repair does not replace any Davinci provider.

References are semantic project operations, independent of the cross-file
**lint** switch. Imported bindings use the existing canonical configured
project. Top-level script-setup bindings keep the existing local/open-importer
surface because another module cannot import them. The ownership filter still
rejects a configured project that excludes the queried document and respects
nearest-directory TS-before-JS discovery and JavaScript admission.

Request-time discovery adds ordinary JS/TS source paths to the existing
canonical requested-source surface, so reverse TS importers can participate.
The inventory is lazy and caches only paths, keyed by workspace roots and file
notification generation. It performs no initialization walk. Closed contents
are read afresh; open buffers override them. File create/delete/rename and
watched-source notifications invalidate the snapshot. Discovery failures are
not cached. The existing native project/session, source mapping, cancellation
and private-URI guards remain authoritative.

Workspace symbols use that same demand-loaded inventory. Existing Vue symbol
production and ranking remain; ordinary script declarations use their
original AST identifier spans and UTF-16 ranges, without descending into
nested functions or parsing comments as declarations. Symbol collection is
structural and does not require a new checker process.

Hosted qualification retains complete inputs, provider/binary/source identities,
raw framed protocol, decoded responses, stdout/stderr and terminal status.
The original references are checked with declarations on/off under both lint
switch values, with all four files open and with importers unopened. Dirty TS
importers and close-to-disk restoration, setup-local namespace isolation,
unopened component/export symbols and dirty/closed exports are full controls.

The separate stock-native project preserves the complete original script
texts and offsets; an explicit ambient Vue-import declaration supplies the
unrelated SFC component import. All four whole native diagnostic reports and
both complete reference vectors are asserted before the Vize controls.
Reference group order is compared after a complete location sort; the raw
native response is retained, while Vize's public sorted vector is exact.
The native source authority is pinned at `2bd066d87f5bafd315be9f40889d0a60b9e58e0b`
(`internal/ls/findallreferences.go`, Git blob `28043f859dd5b655bacbad9c0a472a7551701e3b`).

Execution, unchanged-budget protected qualification, actual signed merge and
public release are still pending. Existing corpus expectations are retained;
no benchmark threshold, optional campaign, SDK or diagnostic filtering changes.
