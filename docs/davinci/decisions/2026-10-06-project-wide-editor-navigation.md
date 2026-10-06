# Project-wide editor navigation (#8013)

The public issue's four complete original sources are retained under
`tests/_fixtures/differential/lsp/project-navigation-8013`. Its described
package/TypeScript settings are made explicit in separate authored controls;
Vue 3.5.43 and native 7.0.2 are the qualified providers. No private project is
used. This legacy repair does not replace any Davinci provider.

References are semantic project operations, independent of the cross-file
**lint** switch. Imported bindings use the existing canonical configured
project. Independent script-setup locals keep the existing local/open-importer
surface. Shorthand object bindings also identify their source property and use
the project surface; their non-exported local declaration alone cannot establish
namespace isolation. The ownership filter still
rejects a configured project that excludes the queried document and respects
nearest-directory TS-before-JS discovery and JavaScript admission.

Request-time discovery adds ordinary JS/TS source paths to the existing
canonical requested-source surface, so reverse TS importers can participate.
The inventory is lazy and caches only paths, keyed by workspace roots and file
notification generation. It performs no initialization walk. Closed contents
are read afresh; open buffers override them. File create/delete/rename and
watched-source notifications invalidate the snapshot. Discovery failures are
not cached: the same walk retains its error status and still returns available
sources. Cached reads recheck the generation and roots after background I/O,
retrying if file membership or configured folders changed in flight. Missing-root
recovery and controlled root/folder/create/delete races have complete-state laws.
The existing native project/session, source mapping, cancellation
and private-URI guards remain authoritative.

The original regular-script `export const shared` integration law already
asserted the complete five-location project result with `crossFile: true`.
Its former false-flag condition excluded the unopened named-export importer;
that expectation encoded the #8013 bug, rather than a setup-local namespace.
Both lint values now require the same existing complete exported-symbol oracle,
with original SFCs, configuration, request, ordering and declaration flags intact.
The separate setup-local, lexical-shadow and dirty/close laws are unchanged.
This correction follows the public project-wide API contract and pinned native
`ProvideReferences` declaration/group semantics, not a Vize output recapture.

Workspace symbols use that same demand-loaded inventory. Existing Vue symbol
production and ranking remain; ordinary script declarations use their
original AST identifier spans and UTF-16 ranges, without descending into
nested functions or parsing comments as declarations. Symbol collection is
structural and does not require a new checker process.

Hosted qualification retains complete inputs, provider/binary/source identities,
raw framed protocol, decoded responses, stdout/stderr and terminal status.
The original references are checked with declarations on/off under both lint
switch values, with all four files open and with importers unopened. Dirty TS
importers and close-to-disk restoration, original shorthand property linkage,
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

The current cb180 mandatory native failure retains all four original files:
its additional App-only `show` oracle incorrectly classified
`const { show } = useToast()` as an independent lexical binding. Pinned native
`findallreferences.go` lines 426–430 explicitly gives a shorthand object binding
project scope; lines 2586–2600 and `utilities.go` lines 372–376, 989–1001 resolve
its source-property identity. The separate stock process now queries this exact
original binding and asserts the three complete script endpoints before Vize.
Vue's existing semantic links recover the two authored template uses; no native
location is discarded. The classifier reuses its original single analysis parse
for the AST pattern and Croquis metadata, preserving simple and explicitly
aliased local bindings and all existing shadow/dirty/close controls. The native
whole-vector, fresh source and protected qualification of this correction remain
pending; the authentic previous failure is retained. `utilities.go` is pinned to
Git blob `8dab6bce9b867e6b6fca9d0b197fa5c117feb296` at the same native commit.

The same-PR successor incorporates signed actual main `97d5c26a153944e66439a3108746c19d7bf62220` (#8120) after the real generated Maestro inventory conflict prevented Actions. All incoming shared declaration-option producers and whole controls stay byte-exact; only the composed consumer census is regenerated. All four original sources and the shorthand successor's independently authored whole expectations remain unchanged, with fresh current-source qualification required.
