# Original Vue descriptor observations (#6837)

The L1 container now offers `Vue.observe_descriptor` alongside the unchanged
capture API. Both use the same block-emission loop. At each actual block event,
policy records its original index, raw attributes and source-checked content.
There is no second source scan, JavaScript parse, pipeline stage or serialization.
The original complete `Container` retains every block, attribute, diagnostic and
byte between blocks, including rejected and unsupported input.

`DescriptorObservation` owns that container, the explicit `DescriptorOptions`,
a checked root when addressable, selections and typed policy issues. Its fields
are private and its getters are read-only. No constructor accepts a caller-built
`Container`, and no mutable capture access is provided. `admitted()` borrows a
private `AdmittedDescriptor` only when all original splitter diagnostics and
policy issues are absent. Refusal exposes both original diagnostics and typed
policy evidence, while keeping the complete owner available.

An admitted script or template view borrows this original owner. Each yields the
actual source-order container index and exact `SourceBlock`. Script views also
report the authentic ordinary/setup role and `Lang`; consumers cannot choose a
role or substitute a block. The root is the complete authored file. The required
script input is `EmbedSource::authored(view.source(), view.block().span())`, using
that whole file and the file-absolute span. Raw script entities are not decoded.
The existing checked source provider rejects equal text from a foreign allocation.

The first supported file policy explicitly selects `VueVersion::V3`,
`VueDialect::Vue` and ordinary template switches. It selects JS/TS **Module**
scripts with JSX disabled; the eventual Program consumer must use those explicit
parse options. Absent `lang` means JS; only exact plain `js` and `ts` values are
accepted. Template language follows the matching actual script roles, including
legal setup-only TS; no scripts means JS. Template `lang` is absent or plain
`html`, and setup is a bare attribute. Quoted and unquoted plain values preserve
the same authored spelling and spans.

Other versions/dialects/switches, encoded or unsupported languages, unknown or
ambiguous attributes, valued setup, external `src`, duplicate attributes/roles,
script language mismatch, mixed-case block spelling, self-closing or uncertain
boundaries remain typed refusals. Styles and custom blocks are retained with an
explicit unsupported-block issue. This bounded structural admission does not
certify script or template syntax, native file semantics or a completed product.
No filename inference, retry, legacy descriptor or public diagnostic capability
establishes admission.

At least one actual template/ordinary/setup selection is required; absent roles
yield `MissingComponentBlock` from recorded metadata without another scan.
An empty authored template or script is still an original structural block.
The locked `@vue/compiler-sfc@3.6.0-beta.10` oracle accepts an empty template,
but its default `ignoreEmpty` drops an empty script and then rejects a file with
no other component block. This provider preserves that script and certifies only
its structural slice/role; the native assembler must enforce the actual compiler
family's empty-script/cardinality policy. It cannot treat this capability as full
SFC syntax or semantic acceptance.

The existing splitter code and scanner are preserved apart from extracting the
same loop behind a monomorphized block callback. The capture callback is empty.
No files are renamed, so there is no move-only commit or rename script. The
source-only storage inventory adds the two arena issue-vector owners and the
existing L0 string import used by identity tests; no classifier, fixture or
instruction ceiling changes.

Meaningful laws cover reversed block order, setup-only TS, ordinary/template
language policy, Unicode/CRLF/raw entity spans, exact root/block pointer identity,
foreign equal-source rejection, retained original captures/errors, typed dialect,
attribute, role and boundary refusals. Compile-fail laws forbid capture promotion,
owner mutation and forged admitted views; a positive public example proves the
real API is available. Local module checks use cached actual L0/L1 primitives;
full current-source crate compilation, no_std/wasm consumers, differential output
and all unchanged instruction gates remain Actions and protected-queue work.

This slice changes no shipped product route. Whole #6837, #6836 and every
#6879–#6883 product fix-history gate remain open. The native SFC assembler and
file-owned Vue factory/exposure receipts remain separate actual-provider work;
full dialects, styles, custom blocks, JSX/TSX and template/script syntax still
require their own source-owned contracts and laws. Publication and actual merge
are recorded separately from source implementation.
