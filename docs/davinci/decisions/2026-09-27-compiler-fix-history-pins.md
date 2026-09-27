# Compiler fix-history byte references

Tracks [#6880](https://github.com/ubugeeei-prod/vize/issues/6880).

Two existing, measured full SFC results become ordinary Rust integration
fixtures. Fix `fe724476c7a1fd1bd9bf784699814626994ff9b4` protects a user's
`render` import when the generated render function is emitted separately.
Fix `92adfb6927517974101e6837bed34bf76b55da9b` protects runtime and side-effect
imports while erasing inline type-only imports across both script blocks.
Their existing substring assertions remain intact.

Inputs, options, raw observations and expected Results are byte-preserved from
the prepared compiler archive. The manifest records their original paths,
SHA256 values, archive revision, measured source revision and observer hash.
Both raw observations contain the same complete payload as their golden, and
both inputs still match the authored Rust witnesses exactly. Output is copied
from actual measurement, never reconstructed from those substring assertions.
The original source-build archive remains part of the separate shared observer
preparation; this small product test does not authenticate a fresh observer.

The test invokes each original public entrypoint with its original options.
It rejects default-option drift, then compares the complete serialized Result.
Only JSON object-key order is immaterial. Strings, terminal whitespace, CRLF,
null/presence, scalar types and array order remain significant. `code`, `css`,
`map`, errors, warnings, bindings and macro artifacts are all retained. These
references contain null CSS/maps and no macro artifacts, so they supply no
coverage credit for populated instances of those fields.

These crate fixture inputs use `.input.txt` and are executed explicitly by
`compiler_fix_history`; they do not add implicit `.vue` memberships to the
separate broad ecosystem corpus or modify its L2/L3 baselines. Shared product
adapter registration and its corpus coverage remain independent work.

Validation before publication: eight archive hashes, exact authored-input
matches, complete observation/golden equality and Rust formatting pass.
Fresh Actions execution of these two Rust tests remains required. No local
workspace build or native-compiler acceptance is claimed.

## Coverage boundary and next fixtures

The issue records a historical 609-fix snapshot without a source revision.
At main `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`, a reproducible all-parent
path history over its eight named crates has 1,208 touching commits and 631
conventional `fix` subjects (626 after excluding merges). Neither count is a
reviewed semantic requirement count. Reconcile the original snapshot and
duplicate patches before assigning a remaining-fix denominator.

Existing unpublished preparation contains ten measured full SFC client/DOM
profiles. It has no complete SSR or Vapor profile registration. These two
ordinary tests establish two executable complete Result references; they do
not close the history audit or supersede the other eight preparations.

An SSR source audit found direct string comparisons for 16 Vue-alignment
preamble/code modules, six component spread code references and 12 SFC
slot-scope modules. Insta text snapshots also preserve useful code shapes,
but Insta trims terminal whitespace and converts CRLF to LF, so those alone
cannot count as byte-exact output evidence. Code-only assertions do not pin
separate preambles or source maps.

The next missing SSR exact-output slices are select `v-model` (#990), duplicate
component handlers (#3701), and keyed slot hydration (#2487). Their historical
witnesses use substring/count assertions. Capture complete actual results with
source and artifact identity, preserve original inputs/options, and retain
those partial witnesses. Review DOM, Vapor, JSX, parser and semantic fix
histories separately; no unreviewed fix is marked covered. Keep #6880 open
until the required target-specific history fixtures and Actions execution
are complete. Native acceptance remains zero.

## Complete diagnostic successor

A third fixture pins the complete public `Ok` result for fix
`336622af2aea3f9bce3e7c297e3b4307bd7445ff` (#1416). An invalid template
expression still produces its actual module plus one `TEMPLATE_ERROR`; the
fixture preserves the entire message and all eight block-location fields.
Input/options, raw observation and expected Result are immutable copies of
the source `292a5d3c80285f5dc6c66fa166f245728ea27548` measured archive, with
their own provenance file. The existing partial diagnostic witness remains.
Hashes, exact authored input equality, raw Result equality, formatting and
source lengths pass. Product execution still requires fresh Actions.

Direct commit-subject inspection also separates fixture profiles from fix
coverage: the earlier structured source-map profile originates in `feat`
#6365, and the definePage macro-artifact profile originates in `feat` #209.
The ten prepared profiles link eight fix commits and two feature commits;
feature controls do not count against the compiler fix-history obligation.

The first publication head's Actions Rust build and all four test shards pass.
Tooling exposed two source policy requirements: gated test modules use ordinary
module discovery, and the observational consumer shard includes new helper
references. The tests now share the existing ordinary `support` module, with
a documented dead-code expectation scoped to the shared test support module. The
consumer generator updates only the SFC shard. All fixture bytes stay fixed;
fresh Actions must pass again on this corrected head.

Strict Clippy also passes the shared support expectation for both fixture
integration targets and the existing CSS recovery target; each uses a different
subset. The unused-code expectation replaces allow attributes and includes its
reason. No expected compiler output or capture metadata changes.
