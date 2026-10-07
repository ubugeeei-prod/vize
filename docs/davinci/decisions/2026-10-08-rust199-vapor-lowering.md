# Vapor placeholder planning on Rust 1.99

Rust 1.99 source `9515b781ffb466720fd23e1d5270bc08cf02bf2a`, tested as merge
`fce6d9dd83c23d579a7bf8c88974ef9f5211c8bf`, measured 520,095 instructions for
the unchanged `atelier_vapor_lower_stress-interp` input against its 518,158
ceiling. Run `37647487780`, job `112881946087`, retained three identical full
Callgrind packets: placeholder planning used 139,841 inclusive instructions,
including 114,283 for collecting an intermediate list of rendered/block units.

Plan placeholders directly while traversing the existing children backwards.
Transparent templates recurse backwards with the same sticky rendered-sibling
flag; text, interpolations and plain elements mark that flag, block children
append it, and comments retain their existing omission. Reverse the resulting
flags into authored block order as before. This removes the temporary unit list
and its allocations without adding a walk, stage or output representation.

Five authored controls freeze nested-template, ignored-comment, text-only and
trailing-block ordering. Existing full compiler/runtime fixtures, benchmark
inputs, stage boundaries, instruction ceilings and allocation budgets remain
unchanged. Fresh hosted source, instruction and allocation results are pending;
any observed allocation reduction may only ratchet its budget down.
