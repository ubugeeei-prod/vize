# Canon original Program checker

Related: [#6849](https://github.com/ubugeeei-prod/vize/issues/6849),
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

`CorsaBridge::check_original_program` is an explicit product adapter for the
merged L4 whole-source Module projection. It takes the genuine completed File
and an absolute existing authored path. It keeps that same borrowed File,
original parse kind, complete projected bytes, canonical authored URI, original
source SHA-256 and complete ordered raw LSP report together in one result.
Every primary diagnostic keeps either an exact authored byte span or a typed
mapping refusal; related documents and all raw diagnostic fields remain intact.

The path must be inside the configured real project and its bytes must equal
the File source before submission, on entry to the worker, after checking and
cleanup, and before result publication. The root configuration bytes are also
checked around the worker job. Non-UTF8 transport paths, declaration-file
extensions, mismatched source kinds, a nested configuration or a different
explicit configuration refuse. The actual Corsa `parseConfigFile` endpoint
supplies normalized inherited options and root-file membership. Excluded files
and unsupported configured-project APIs refuse instead of using an inferred
project. Plain `.js`/`.ts` inputs require genuine import/export rows or actual
effective Force module detection; `.mjs`/`.mts` have an intrinsic Module goal.
Empty `export {}` currently has no public L2 export marker, so that input still
requires Force or an intrinsic Module extension.

An independent real-URI LSP overlay sends the exact L4 document to the
configured checker. This bypasses virtual-project construction and every
materialization/write path. It does not substitute the legacy Vue generator.
Comments, JSDoc and invocations remain supported whenever the actual lower
File is complete. Relative imports resolve from the actual authored path.
The same bounded bridge worker owns the request and always shuts down/reaps
the overlay session after either success or failure. A timeout abandons the
caller while the worker finishes transport cleanup.

The original ten L4 checker source fixtures remain exact. Canon's independent
full-vector law additionally checks the actual LSP unused-variable hints,
including severity, code, message and every serialized field. Further real
checker laws cover relative imports, a declared invocation, inherited Force,
nondefault `strict: false`, excluded sources, intrinsic Modules, declaration
kind and profile refusals. Source preservation and same-File retention are
asserted. A retained bridge reloads changed inherited options. Separate laws
cover CR/LF/CRLF and invalid UTF-16/generated ranges, plus actual failed and
timed-out overlay startup with process reaping and unchanged source bytes.
Exact-head Actions and protected queue acceptance remain required.

This is a root-member source snapshot contract, not a frozen complete
multi-file project revision: concurrent changes in dependencies, inherited
configuration or package metadata are not yet captured as one graph snapshot.
Vue/template/JSX projection, complete token maps, source edits, broader project
membership and whole fix-history comparison remain unfinished. The default
product route is unchanged and #6849/#6879 stay open.
