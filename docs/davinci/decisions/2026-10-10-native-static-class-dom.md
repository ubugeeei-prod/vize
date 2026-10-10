# Native static class DOM prerequisite

This advances #6839/#6840 after the completed lower File SSR prerequisite
in #8385. It does not replace a product's default compiler path.

## Decision

The existing DOM canonical element visit admits static `class` and observes
every retained original attribute slot, including suffixes after a refused
special name. It adds neither a pipeline stage nor another op walk.
Conditional native element eligibility follows the same static class policy.

Move the already qualified SSR class normalizer unchanged in a move-only
commit to the shared target module. DOM consumes the same borrowed or
normalized value while retaining the original attribute span. Canonical
class values borrow storage; only HTML whitespace condensation allocates.
ECMAScript trimming differs from Rust Unicode trimming: U+FEFF trims,
U+0085 does not, and interior non-HTML whitespace remains authored data.
Ordinary attributes retain their decoded values exactly.

Static and dynamic `class` together retain `DuplicateProperty` refusal.
Style, key, ref and unsupported control families keep typed whole-output
refusals. Admission of their combined semantics requires separate proof.

## Qualification

Add a separate sixteen-case primary Vue 3.5.35 DOM packet. Keep every prior
DOM/SSR, selected-SFC, history and benchmark packet unchanged. The packet
includes the unchanged literal class input from #7502, valued/bare/empty
attributes, whitespace, Unicode, entities, fragments, comments, void nodes
and ordinary attributes after class.

The test-first Rust law fails on the original DOM refusal. Its final law
uses genuine native parsing and a sealed, normally completed lower File.
Both link sinks compare every prepared template-module byte to the whole
official code. The same target fragment assembles the whole component,
and every source/generated link endpoint must retain UTF-8 boundaries.

Affected source Actions require a fresh source-built capture and official
code/raw-map recompilation. Fresh development and production processes
mount both whole components, compare complete DOM trees and diagnostics,
unmount them, and compare whole-component SSR outputs with three actual
fallthrough contexts. Every execution repeats. The independent map judge
checks every segment-bearing native link; deleting anchors must fail.
Complete upstream map-byte parity is not claimed.

Test-first source `80f926e19` failed the actual `class-root` admission law
in Check run `38029883502`, tooling job `114148465172`; the combined class
law also observed the earlier special-name refusal before the fix.

## Remaining work

This prerequisite does not grant original selected-SFC class admission.
The current original #7502 packet remains an unchanged refusal here.
The public selected entry needs genuine L2 admission and a narrowly
versioned positive successor qualified alongside the mandatory immutable
historical baseline. Vapor retains its independent static-class refusal.
#6880 and the complete L3/L4/product roadmap remain open. Instruction
ceilings are unchanged; the 10x target has no new measurement here.

Source and protected Actions must pass on their exact heads, and the
protected candidate must actually merge before this slice is delivered.
