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

The first composed source `da43c7ccd2` remains failed. Its original tooling run
38122423237 fixed #5823 but retained #5071's false Vue-module diagnostic and
introduced TS2528 plus an unresolved `__default__` in the two-block source.
Restoring the setup role exposed a second original parse boundary: the valid
classic block's default export was classified by reparsing the whole merged
script, so its malformed setup neighbor erased the export targets. The bounded
correction classifies the actual classic prefix with the same existing parser
only when descriptor-proved setup lacks analysis. For a descriptor-proved lone
setup, the existing unterminated-import layout retains the authentic section
at module scope and maps its following token to the actual setup token. Both
original syntax coordinates and full diagnostic vectors remain required;
successful setup and plain-only generation remain byte-exact controls. The
new lone-setup finder preserves ASI across block comments, semicolons, LF, CR
and Unicode line terminators. If a later setup error makes parsing panic, the
existing TSX retry parses only its genuinely terminated leading import prefix;
that returned AST remains the authority for module spans. No parser invocation
is added. Whole-SFC/native controls require only the genuine later TS1109 for
six valid comment boundaries, and preserve the real TS1005 for a same-line
unterminated import. The original two-block finder remains unchanged. The
new lone scanner also distinguishes quoted imported names from the genuine
module source: `import { 'ref' as ref } from 'vue'` is a valid terminated import,
not a missing terminator at `as`. Its complete corpus SFC and same-config native
vector retain only the genuine later expression error.
When the new lone section contains an already recovered leading import, its
contained AST spans are replaced by that encompassing section so the complete
authored declarations are emitted once. The two-import corpus/native control
requires only its genuine missing-terminator TS1005; old two-block planning
does not change.
The first full qualifier 38122497358 is also retained as failed, with no green credit.
Its original full App row passed all 15 negative diagnostic controls. Its
same-config native LSP #5071 before vector already contained exactly the two
original TS1005 diagnostics and no TS2307; the initial native test's once-TS2307
premise was false. That law now requires complete before/after equality for
#5071, while #5823 still requires a genuine before TS2304 for `T` and removal of
only that false row. Both pairs are saved before either verdict is asserted.
The public CLI's #5071 TS2307 remains a separate failing origin. Its generated
project lives outside the authored package tree and uses the existing import
rewriter, unlike the native control's original-tree paths/config. A lower
production-collector control retains complete before/after specifier packets;
the actual public checker separately must return only the original syntax
vectors, retaining its generated files/config, full result and independently
attested same-generated-config raw native vector. No CLI fix credit is taken
until that public control and unchanged original tooling corpus pass.

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

The fourth Stack layer first moves the complete existing ElementPlus native
projection control into its own test module without changing its 3,195-byte
function. The parent test file drops from 390 to 320 lines; the same native test
binary still runs all four controls.

The single composed qualifier also enables retention immediately after the
five original application JSON parses. It saves raw stdout, the unmodified
report and programs, authored argv/cwd, reported source/config bytes, config
extends, package contracts and physical generated-config observations before
unchanged assertions. Native execution-time config/options and dependency
reads remain explicitly unproved when the public report does not provide them;
source-derived cache paths are observations, not invented selection receipts.
The shared App suite keeps its existing behavior when the capture environment
variable is absent. No expected snapshot, diagnostic, count, deadline or
program selection changes in this layer.

The retention reader passes a bound TypeScript file-read callback, preserving
the complete config bytes while meeting the original zero-warning source gate.

The exact composed `888a4898a8` qualifier 38129574946 remains failed. Its
whole #5071 programs retain their real malformed import, so the existing OXC
whole-program parser refuses its AST both before and after; both actual import
collector vectors are empty. A nonempty AST premise was unsupported. The law
now records the parser panic, complete diagnostics and body length and requires
that exact refusal, while retaining the actual once-emitted module prefix,
authored source mappings and complete native/public import controls. Repaired
successful scripts require the same collector to see the genuine Vue specifier.
The genuine #5823 unused `info` TS6133 suggestion appears in both before/after
native vectors and in the public result; its complete public oracle preserves
that suggestion alongside TS1109 without filtering diagnostics.

The CR comment packet retained raw TS1109 while its mapped vector was empty:
the old source-map helper counted LF only and selected `void broken` instead
of `const broken = ;`. Public VirtualProject position mapping used that same
helper. The correction delegates to the existing Carton LSP/UTF-16 authority:
CR, LF and CRLF have their actual boundaries, LS/PS stay inside LSP lines,
and invalid UTF-16/surrogate or out-of-range positions remain refused. Compiler
diagnostics keep their existing separate TypeScript convention. The exact CR
SFC is retained as a corpus fixture and checked against the literal prior
converter; the public bidirectional mapping also has genuine positive and
invalid-position controls. Native comment vectors retain every original case.
A test-only accessor returns both actual materialized root and actual generated
config path from the existing project, avoiding guessed root config names.
Preparation is not source/native acceptance; fresh combined Actions remains
required and no projection snapshot or upstream baseline is changed here.

### Actual source compilation correction

The exact-source Rust gate at `59f6d6f63c` rejected two refusal assertions because
`Option<OriginalPosition>` does not implement the equality required by
`assert_eq!(..., None)`. Use `is_none()` for the same invalid UTF-16 and
out-of-range positions. Product code, coordinates and complete diagnostic
expectations remain unchanged; the failed job `114456639491` is preserved.
