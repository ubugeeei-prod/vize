# Keep authored setup metadata after script parse failure (#6879)

Paired decision: [#6879 comment](https://github.com/ubugeeei-prod/vize/issues/6879#issuecomment-6106696196).

The complete native semantic phase exposed two producer mistakes in the pinned
Vue Language Tools corpus. With the original malformed script, `_failed_#5823`
reported TS2304 for an authored `generic="T"`; `_failed_#5071/withoutScript.vue`
reported TS2307 for its installed Vue import. Their original TS1109 and TS1005
diagnostics remain required. This is a bounded fourth slice of native Stack
8598, following its anchor, JSX, and native semantic completion fixes.

The descriptor retains `<script setup>` and its `generic` attribute even when
script analysis returns no setup scope. Canon previously read both facts only
from Croquis: missing scope made it treat setup as a normal script, and missing
generic made `__setup<T>()` become `__setup()`. The correction passes immutable
descriptor metadata through the existing internal generation options. Missing
analysis scopes use the authored block role and attribute. A successful setup
analysis retains its complete original signature, including async. No recovered
AST facts, async inference, additional pipeline stage, or diagnostic suppression
is introduced. This does not replace a product with Davinci; #6879 remains open.

The regression corpus at
`tests/_fixtures/differential/typechecker/malformed-setup-metadata/` retains all
three complete original SFCs, both original project configurations and their
complete base, hashes, license, revision
`88e8500c1e5f1b29d80f42c8ca065cc9cbd56899`, and the exact failing vectors from
the prior hosted tooling log. Both the upstream manifest and documented
differences stay unchanged. The existing split-script glued-import layout and
its `withScript.vue:1:19` TS1005 coordinate remain strict controls.

The internal `vue_codegen::malformed_setup_tests::` controls use the actual
descriptor analysis and the unchanged prior generator option set, with no
descriptor witness, for the before projection. The after projection uses the
production whole-SFC generator. Successful repaired and async scripts require
complete emitted code, mappings, and semantic links to stay equal. Both sides
retain whole descriptor/Croquis/source/code/map packets. Required native tests
use the same binary, file paths, complete configuration chain, and pinned Vue
3.6.0-rc.6. The native client that returns the full diagnostic vectors also
attests its selected configuration and complete normalized compiler options.
The syntax-only CLI output is retained separately and cannot establish semantic
correctness. The negative control retains the malformed source while adding a
real undeclared name and a number/string assignment error; both must be diagnosed.

Local formatting, module size and generated inventory checks are preparation
evidence. Production compilation, the complete authored/reference corpus,
paired native diagnostic controls, strict Actions, protected queue, merge and
installed release qualification remain pending until their exact composed
source runs finish. The App E2E owner publishes the fourth child and runs one
complete composed qualifier. No second parent build or separate full observer
is added for this slice.
