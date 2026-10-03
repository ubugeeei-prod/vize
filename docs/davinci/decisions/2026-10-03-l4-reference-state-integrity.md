# L4 reference-state integrity

This bounded correction addresses the two still-valid review findings on
[#7465](https://github.com/ubugeeei-prod/vize/pull/7465), paired with
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

The findings are the [portable capture identity thread](https://github.com/ubugeeei-prod/vize/pull/7465#discussion_r4169905339)
(`PRRT_kwDOQvjuaM6ofo6f`) and the [execution integrity thread](https://github.com/ubugeeei-prod/vize/pull/7465#discussion_r4169905344)
(`PRRT_kwDOQvjuaM6ofo6i`). Both remain unresolved until public-source verification.

The six original Vue 3.5.35 reference files remain byte-identical. Their
capture metadata uses package manifest specifiers, resolved through the actual
installed Vue package, instead of developer-local absolute paths. The tests
validate the existing manifest SHA-256 digests. The original uncommitted
development capture recipe is identified by its historical artifact ID and
unchanged digest; the identifier does not claim a portable executable path.
The committed reference test independently replays the complete pinned
compiler options, template modules, script oracle and unfiltered raw maps.

Each render now checks every enumerable setup-state value, the complete VNode
snapshot and forbidden foreign reads. It compares all measured execution
records with the pinned records and verifies `executionsSha256` both before
execution and against the measured records. A coherently rehashed wrong state
and an incorrect digest are meaningful negative witnesses: both were accepted
by the old observer and are rejected by the corrected test.

The two witnesses fail on the byte-identical `e565` execution helper before
this correction. The complete test then passes ten cases, including the six
reference modules and their twelve initial/mutated-state render contexts.
The optional native-capture bridge is absent in this check, so these results
grant no new native render credit. Existing native proofs keep their original
source and ABI attribution.

No Rust production source, dependency, module output, source-map expectation,
raw reference fixture or performance budget changes. The metadata-only
`capture.json` change is explicitly excluded from old whole-fixture-tree
identity claims; actual compiled Rust and six native input files remain exact.
Current full Check, instruction counts, review follow-through and protected
queue validation remain necessary after the delivery actor updates the PR.
Review threads are not marked resolved before the corrected source is public.
Full Vue grammar, scripts/reactivity, controls/directives/dialects, complete
upstream maps and compiler history #6880 remain unfinished.
