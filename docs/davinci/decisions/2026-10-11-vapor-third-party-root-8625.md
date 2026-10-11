# Single-root fallthrough (#8625)

Preserve Hebilicious's #8620 root selection and all focused single, keyed,
conditional, nested and sibling controls. Move its root collector to a small
module to keep generate.rs inside the existing 350-line source budget. Both
recursive function bodies and stack protection remain unchanged. The composed
slot and root fixes also move the existing scope lookup into context/scopes.rs
without changing its resolution order or body; context.rs stays below the same
source limit. Exact trailing source newlines are retained in JSON inputs and
decoded without trimming.

The historical capture-b11eb and its five expected files remain immutable.
The root-document-order fixture gets one explicit current successor: only the
first sibling component's root flag changes from true to false. An independent
whole-packet test authorizes precisely that literal change. The original history
integrity, all original options, all five live whole-output comparisons and all
Davinci unsupported coordinates remain enforced. The differential manifest pins
both the historical reference and its current successor with their hashes and
the defect link. No case is removed or marked inactive.

The reporter's original whole FallthroughOuter and FallthroughInner SFCs are
also registered as a runtime corpus. Explicit owner props must land on the
section and never on its nested span, with complete locked Vue VDOM parity and
unmount coverage. Typed-handler snapshots in the next Stack layer must inherit
the corrected sibling-root flag.

Exact Actions, protected merge and installed release acceptance remain pending
until their actual executions succeed.

Independent review found the same leaked owner attribute inside a KeepAlive
slot whose single root is an element containing a component. Replace the blanket
KeepAlive/Transition component flag with the existing single-return collector at
those wrappers' slot boundary. Returned conditional and keyed component roots
retain the flag; nested element descendants do not. TransitionGroup remains a
fragment even with one child and does not get this forwarding rule. Pre-resolved
components with no props may use the short creation call only when their flag is
false, preserving the required root flag for `<KeepAlive><Child /></KeepAlive>`.

Additive complete runtime controls compare direct, nested, conditional and keyed
KeepAlive output, updates and disposal with the independent locked Vue VDOM
oracle. The original reporter inputs and expectations stay byte-exact. Official
rc.9 Vapor itself omits inherited attrs on the direct KeepAlive child; VDOM and
the authored full DOM expectations are the semantic authority for this case.
Both retained compiler lanes also cover Transition and TransitionGroup emission.
Fresh exact-head Actions and protected gates still qualify the resulting source.

Fresh Check 38143613522 executes the original report and all four added KeepAlive
runtime controls successfully. Its two red shards expose three stale downstream
expectations: a JSX component among sibling roots, the nested AfsButton in the
#8441 actions slot, and the checked KeepAlive no-props child. Update only the two
focused snapshot flag bytes shown by the actual run. Preserve #8441's entire
historical runtime packet and `runtime-baseline.json`; register a separately
hashed complete successor that changes only AfsButton's enclosing-render root
flag. The existing complete-packet test selects that successor only for the
reported Vapor packet, and a custody law verifies every remaining field and
original hash. The independently installed official Vue compiler 3.6.0-rc.9 and
3.6.0-rc.10 produce identical complete emissions for the mixed-root and ordinary-
slot controls, omitting the nested component root flag. The separate successor
records their version, control and output digests; this compiler diagnosis gives
no installed public Vize or release-consumer credit. KeepAlive's authored VDOM
runtime expectations govern its previously recorded omission. Fresh source and
protected merge-group qualification remain required.
