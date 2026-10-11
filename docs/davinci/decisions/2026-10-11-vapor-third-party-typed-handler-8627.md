# Typed native element callbacks (#8627)

Preserve Hebilicious's #8622 implementation: the native element callback uses
the same TypeScript-capable expression classifier as component callbacks.
The JavaScript fast path and authored callback parameter ownership stay intact.

Register the original whole reported SFC and click action in the legacy
differential corpus. The existing locked Vue VDOM oracle compares every DOM
phase, namespace, event and unmount. Independently authored complete DOM phases
require the visible result to change from none to click. Disposal still rejects
updates from retained buttons after unmount.

The new source snapshot inherits the preceding root-flag layer's false value
for Child among sibling roots; no other snapshot line changes from #8622.
This dependent layer follows the root fix in a native Stack and must enter the
protected queue through the highest fully qualified contiguous prefix.

Legacy runtime parity is separate from Davinci native acceptance and from
installed release verification. The external issue remains open until actual
Actions, merge and public-consumer validation establish the delivered fix.

The existing locked VDOM oracle enables its official TypeScript template parser
and uses the already installed Vite Oxc transform to erase types from complete
TypeScript modules before evaluating them. JavaScript sources retain their
existing complete code; all runtime packet comparisons remain enforced.
