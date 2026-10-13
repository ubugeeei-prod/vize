# Known-empty Vue star exports

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The retained Nuxt UI application has 547 diagnostics, including 534 TS2308
collisions. Authored unaliased `export *` edges expose generated `Props`,
`Emits` and `Slots` from several Vue projections. The original application
inputs, membership, configuration and complete failure vectors remain fixed.
This bounded change does not establish complete Nuxt UI or application parity.

## Decision

Only a registered genuine Vue module with a proven empty authored star-export
set may replace an unaliased top-level `*` with `{}`. Both original inline
script blocks must be admitted by their existing OXC parse. No authored named,
type, indirect or star export may be present. A normal script exporting only a
local default component is eligible: default never belongs to the star-export
set. Authored exports in script setup, including a default or empty export
declaration, remain unproved because that form is invalid in script setup.

External script sources, unsupported languages, parser recovery, malformed
input, disabled analysis, a caller-supplied program without original parse
custody, or a lost script block keep the result unknown. Private setup types
and compiler macro declarations do not become authored public exports.

This slice changes ordinary configured batch and CLI graph finalization. The
existing editor document finalizer remains unchanged; there is no editor or LSP
acceptance claim from these laws.

The first slice admits only explicit relative `.vue` edges. The complete
projected literal must still be exactly that relative edge with `.ts` appended.
Its destination must be a currently registered virtual companion owned by the
same genuine source. Alias, package-export and extensionless routes remain
conservative, as do foreign or overwritten mirror destinations. Physical
source canonicalization and lexical mirror normalization have distinct jobs;
materializing the mirror cannot change admission.

Named exports are not expanded. Namespace exports, authored duplicates and
direct imports of generated `Props`, `Emits`, `Slots` and the typed default
remain unchanged. The TypeScript checker continues to own their diagnostics.
No generated helper export is removed and no diagnostic is suppressed.

## Parse and lifecycle ownership

The Croquis result and public option/result structs remain unchanged. An
opt-in, nonserialized observation is carried alongside the existing script
parse and statement processing. Ordinary consumers have no observation demand.
The existing SFC script draw and parser invocation counts are unchanged.

The existing import-rewrite parse records top-level bare-star spans and module
literals during its existing visitor. An opt-in statement-depth guard proves
top-level ownership without a separate statement iteration. It adds no parser,
AST traversal, filesystem graph scan, pipeline stage or serialization between
levels. Compatibility wrappers explicitly disable candidate collection.

Graph finalization replays captured spans onto the same owner's original
literal-rewrite baseline. It validates the original quote and complete mapped
literal bytes before admitting an edit. Source maps include the one-byte
expansion and clamp either generated brace to the original star. Finalization
does not parse an ordinary captured barrel again or rerun its literal policy.
Declaration and JSX finalization reuse their already existing rewrite parse.

Each source owns its admission and baseline. Re-registration replaces both;
removal drops the ownership, and a recreated source requires a new real parse.
An empty-to-named-to-empty transition therefore restores the original star
when the target becomes unknown and may erase it again after fresh admission.
Complete source/map/artifact controls cover registration order, deletion and
recreation, overwritten mirror identity and conservative routes.

## Evidence and remaining work

The original six native counterfactual packets are retained losslessly in the
separate supplemental corpus. They bind the genuine Vue and Vite provider
bodies, Panel source, generated source, configuration, actual native executable,
whole raw streams and exit status. Their original whole packet SHA256 is
`a5e32916175feeb304fc88744cb5fcd94ca5f39ee385dad5ed573022d893129e`.
They demonstrate the isolated star-export mechanism:

| Counterfactual          | Before                             | After                          |
| ----------------------- | ---------------------------------- | ------------------------------ |
| Valid consumer          | Three synthetic TS2308 errors      | No errors                      |
| Wrong prop              | Three TS2308 and two TS2322 errors | The same two TS2322 errors     |
| Real authored duplicate | Four TS2308 errors                 | The same authored TS2308 error |

Those packets edited generated source. They do not qualify this producer fix.
The additive production law discovers a separate frozen supplemental pack and
uses the ordinary configured batch checker with genuine installed providers.
It preserves complete authored vectors and separately retains raw native
generated-coordinate output over the actual materialized producer bytes.
Boundary cases prove conservative projection and retain their entire observed
native result without inventing a clean semantic golden.

At source preparation, formatting and complete corpus/hash preservation have
been checked; compilation and native execution remain pending. The existing
source-only Croquis consumption generator was rerun unchanged and its check
passed; only the Atelier SFC shard changed to reflect the actual new consumers.
That inventory is source bookkeeping and supplies no runtime evidence. Required hosted
qualification must identify the current source and emitted executable, admit
the pinned official native package and loaded provider, execute all new laws,
retain whole raw outputs and run every existing gate. An unavailable required
runtime must refuse completion. A normal shard without that runtime supplies
no native acceptance credit.

The historical 32 registrations, all existing differential fixtures and App
goldens remain byte-exact. The supplemental pack grants no historical adapter,
full fix-history, complete application, Matrix, release or performance credit.
The 10x target and original 400-workload comparison remain independent gates.
Their source enrollment must pin every changed complete producer body and
preserved oracle; an unknown, partial or drifting source delta must fail before
execution. Source enrollment supplies no native or performance credit.

TODO: qualify fresh whole application diagnostics with the original membership
and provider/configuration custody after this bounded producer repair. Prove
nonempty authored Vue export sets and alias/package routes independently before
expanding this policy. Preserve real duplicate authored exports, including
reserved names such as `Props`, before any later repair or baseline migration.

The hosted native supplement reuses the existing Canon ci-opt sticky build: one
Cargo no-run emission supplies one exact test executable, executed once with the
required runtime. Three named laws retain all sixteen packets (ten exact vectors
and six conservative projection boundaries), full stdout/stderr, actual source,
provider and generated-byte custody before assertions. This is pending hosted
execution, separate from the six generated-source counterfactuals.

Original400 source admission authenticates all 42 producer/test-target, 80 passive supplemental
and 152 preserved bodies plus eight metadata bodies. Historical tooling reads
are allowed only after complete raw predecessor and physical current custody;
unknown, missing, partial, executable, symlink or hybrid source refuses. Exact
hook inverses retain the original validators and their complete refusal sets.
Source tests and enrollment grant no native, App, throughput or release credit.

The initial `f7d6c2c5ae` source passed its original400 run, but the new native
three-law target did not execute: its test-only digest formatting failed to
compile. The correction emits the same full 64 hexadecimal digest from real
bytes and checks the known `abc` digest inside the existing custody law. Four
private adapters retain their original signatures and bodies under `cfg(test)`;
a separate library law preserves the complete bare-star source and map.
The new Cargo target requires `native` explicitly. The original opaque-attribute
selection guard stays unchanged. Complete builder diagnostics are replayed only
on failure, with their original status. Fresh corrective-source Actions remain
required; the initial source's success supplies no successor runtime credit.

The next `784d9ede9d` source completed its original400 comparison and all four
tooling shards. Its native target genuinely executed three laws: one passed and
two failed. Seven exact cases passed before the reserved-name consumer failed;
nine complete case packets were captured, and seven cases did not execute.
Artifact `11772906117` retains their complete original bytes, SHA256
`b822b657f6cebf1db6a03e52c73931df0149840b95ad537b092bce8177dcf4dd`.

The reserved-name case used an interface-typed value directly as the generated
component constructor input. The current patterned `data-*` input constraint
produced a genuine TS2345 at `consumer.ts:6:41`, unrelated to star ownership.
That complete mapped and raw vector remains retained. The corrected consumer
spreads that value into an object while retaining every direct public type,
authored `Props` export and concrete-type check; its expected vector stays empty.
The original source blobs remain unchanged and available.

For the malformed case, the official native `--listFilesOnly` returns status one
and the complete same-config TS1109 syntax response before its 86 filepaths.
Only this exact frozen case may partition that complete leading response after
validating native/config identity, raw bytes, hashes, empty stderr and status.
Every remaining file is still read and hashed. The whole failure response stays
in its packet and grants no semantic acceptance credit. Other status failures
remain refusals. The existing malformed-setup unit fixture also needs the new
private observation field initialized to false; its original output is retained.

These are test input, fixture construction and evidence corrections. They change
no export-admission policy, historical32 oracle, application golden or budget.
Fresh source, native and original400 qualification is required again.

The correction keeps all seven original 42/80/152 source-table files byte-exact.
One separately authenticated finite record changes only the two reviewed after
cells and adds three complete pairs: the existing malformed fixture's whole
before/after body and two new passive input blobs with genuine before absences.
The first corrective closure is 43 producer/test-target, 82 supplemental and 152 preserved
bodies plus nine metadata bodies. Any original-after substitution, incomplete
correction, mixed source or missing body refuses admission before a historical
tooling view or workload. This enrollment supplies source custody only.

Independent review found that reading the remaining paths and observing Vue and
Vite roles still allowed a duplicate-only listing to hide omitted program files.
The revised malformed law therefore compares the complete ordered 86-member
path, size and SHA256 vector. Normalization uses only the validated native
library root, selected mirror root and the common canonical package root bound
by both authenticated Vue and Vite manifests. Dropped, added, duplicate,
reordered, foreign and changed-body vectors refuse, including 85 Vue entries
plus one Vite entry. These parser controls remain within the existing first
law; they grant no native execution credit and add no fourth target test.

The new passive reference records its distinct evidence sources: membership
order and four materialized bodies from the failed malformed packet, eighty
native/provider declaration bodies from a separate actual packet using the same
native and providers, and two static shared-helper bodies from that separate
packet. The helper writer and captured options establish their constant form.
The failed malformed packet did not read all 86 bodies. The next genuine run
must perform every read and match the entire reference before acceptance.

The complete-membership successor retains those original seven tables and
qualifies exactly three corrected after cells and seven additional whole pairs.
Its current closure is 44 producer/test-target, 85 supplemental and 152 preserved
bodies plus nine metadata bodies. Source-only refusal controls and inventory
generation must pass before publication; fresh native and original400 results
remain required for this exact source.

The JS/Star source join keeps the complete actual JS93 and Star318 body gates
before the original Star custody law. Its eight original tests run unchanged
from the genuine signed `3a680d0a` source in a private 769-file selected closure
(20,164,487 bytes), with mode/blob/length/SHA custody and its real commit/tree.
The new current wrapper is counted separately. Both current guards and every
selected historical file are checked again after the original tests. This is
source qualification; it supplies no native, original400 or performance result.

The first genuine current-branch source run retained 26 passing tests and one
cold-fixture cleanup ENOTEMPTY failure. Only empty `.git/objects/pack` directories
remained; no live maintenance process or marker established its cause. Private
Git commands now disable automatic gc and maintenance before object writes and
commits. The historical fixture likewise carries those private settings. Cleanup
has no retry or ignored error, and shared Git configuration remains untouched.
