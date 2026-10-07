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

Fresh source `3c8ab916e4c7db4e32085f6d8e9f2b7327fcd4a5`, Rust shard
`112925841148`, exposed a premise error in the new helper test: bare `<template>`
parses as `Element`, not transparent `Template` (`parser/element/classify.rs`).
The failed original input returns `[true, false]` under both the unchanged
9515 forward planner and the optimized reverse planner. Preserve all five
original strings and transparent expected vectors. First authenticate their
raw parsed-AST vectors against the verbatim 9515 planner; then explicitly build
transparent helper-boundary IR and compare the complete original vectors under
both planners. This creates no valid-source or runtime credit for the synthetic
transparent IR, and changes no production code, corpus, input or ceiling.
The fresh instruction workflow `37659292846` succeeded, while full source
qualification remains red until this test and the separately owned hotspot
producer-context test pass on a new head.
