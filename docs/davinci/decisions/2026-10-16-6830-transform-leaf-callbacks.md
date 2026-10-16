# Transform leaf callback overhead

The merge-queue qualification for [#9200](https://github.com/ubugeeei-prod/vize/pull/9200)
failed the unchanged `stress-interp` instruction ceilings in
[run 38888282902](https://github.com/ubugeeei-prod/vize/actions/runs/38888282902),
job `116931293359`:

| Stage | Instructions | Ceiling |
| --- | ---: | ---: |
| Core transform | 303,423 | 295,602 |
| Vapor transform | 303,119 | 295,556 |

That PR does not modify the core transform implementation. The failure is an
existing qualification blocker, rather than evidence that its fixture updates
made the transform slower.

Both failure hotspot tables attribute 21,021 exclusive instructions to dropping
`ExitFns` and another 21,021 to dropping its owning iterator. The fixture has
500 interpolations. Interpolations, text, and comments never register exit
callbacks or own structural directives or child traversal, yet previously went
through the shared callback-list construction and reverse-iteration path.

Handle these leaves before constructing that list. Interpolations still call the
same expression transform, including identifier prefixing and lexical scope
resolution. Every leaf still updates `current_node`, registers the same helper
usage counts in the same order, passes the recursion stack guard, and records
the same transform walk visit. Element/structural traversal and callback ordering
keep their existing implementation.

The direct traversal regression exercises all three leaf kinds with an active
`v-for` scope, in normal and Vapor modes. It pins local and external identifier
rewriting, authored expression spans, helper counts and order, and the final
current-node pointer. The existing instruction gate is the performance regression
check: all workloads, fixtures, measurement windows, three-run equality checks,
and numeric ceilings remain unchanged. The x86-64 Actions run must pass before
claiming the ceiling blocker is resolved; a local macOS unit test does not prove
instruction-count acceptance.
