# Native Component context ownership

Bounded source prerequisite for #6836/#6838. The private conversion Context
borrows the exact immutable NativeComponent that entered `construct_in`;
its checked SourceBlock comes from that owner. A separately supplied block is
not stored beside the carrier. This prepares actual event identity without
introducing native For or File/profile admission.

The ordinary child walk and concrete ComponentFactory callbacks are unchanged.
The existing const dialect function-pointer table accepts the short Component
borrow through a higher-ranked lifetime. There is no dynamic dispatch, buffer,
parse, source/AST walk, artifact validation walk or new public API.

After construction, explicitly consume Context into its complete holes,
diagnostics, retained expressions and rejected syntax. Its immutable Component
borrow ends before the same normally owned Component moves into NativeProduced.
The original arena roots/children/comments stay retained by their existing
owners. Existing rejected foreign-source owners remain intact before any mint.

Header laws now borrow a genuine NativeComponent and its original carrier,
instead of independently assembling a block-bearing Context. The real
equal-copy foreign-source and different-root-length factory refusals, exact
prepared ordinals, recursive factory output, retained AST pointers and absolute
whole-file block/provenance laws continue to exercise the production path.

Private selected-source validation uses the retained coherent 700/945 cached
dependencies and source-qualified ordinary L1/L2 F634 libraries: 40 native
laws, strict production Clippy and three external private-construction/retag
compile-fail laws pass. This is not a whole current dd6 L1/L2 or legacy bridge
Cargo build. Current protected Actions and unchanged instruction/allocation
gates remain required on genuine later publication ancestry.

JointFor full-Attribute event admission remains held for immutable corrected
File profile and private control providers. No For
observation, File completion or SFC admission is added by this prerequisite.
Frozen public Stack #7416 source heads and all original private refs are intact.
