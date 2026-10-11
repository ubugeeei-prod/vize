# SVG namespace and normalized bindings (#8624)

Preserve Hebilicious's original #8619 change, author identity and focused
regressions. The original whole SFC and issue body are also registered in the
legacy differential corpus. The existing locked Vue VDOM oracle compares complete
DOM, ordered namespace vectors, updates, events and unmount with the Vapor output.
SVG branch variants and an HTML branch control remain separate authored cases.

The shared SFC runtime accepts explicit root props and the original child import
name for reports that need them. Its default child alias and no-props mounting
behavior remain unchanged. This establishes legacy runtime coverage; it does not
claim a whole-product Davinci native path or acceptance of an unpublished release.

GitHub requires all native Stack branches to live in one repository. Keep the
original fork PR Draft and source intact; carry the reviewed changes in small
same-repository layers with credited original commits and squash through the
protected merge queue. Exact Actions, actual merge and installed release
verification remain required before reporting the issue complete.

The first source plan failed the existing ordinary-module layout guard before
executing Rust or runtime checks. Preserve the guard and move the complete shared
report helper into `tests/third_party_vapor/mod.rs`, with an ordinary module
declaration in the integration target. Every original input, runtime comparison
and authored assertion remains unchanged. Native Stack #8638 orders #8634,
#8635, #8636 and #8637; each repaired exact head and the protected candidate still
require fresh Actions evidence.

The repaired source plan exposed two real reference-harness failures in the
registered whole-SFC tests: Check 38142193165's Rust shard 1 rejects an empty
script block, and Check 38142212288's shard 2 creates a Vapor component inside
the intended independent VDOM tree. Vue omits empty script blocks from its parsed
descriptor, so use the normal empty component plus the unchanged compiled
template in that case. For the VDOM reference only, clear the parsed descriptor's
Vapor target before `compileScript`; preserve every raw source byte and compile
the same template. Product compilation remains Vapor. These corrections retain
the full comparison, diagnostics and expected DOM assertions without adding
interop to the VDOM oracle or changing any historical input.
