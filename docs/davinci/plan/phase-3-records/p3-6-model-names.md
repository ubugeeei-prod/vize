# P3-6 — Computed component model arguments (2026-09-26)

Component `v-model:[name]` retains the authored argument AST independently of
the read/write target. The checked model expands into three typed prop values:
the value, assignment listener, and optional modifiers. Code generation derives
the update and modifier keys after resolving that original AST; it never parses
compiler-generated key or assignment source. Static component models and DOM
model realization retain their existing contracts.

Seven source fixtures exercise direct, indexed, call, conditional and
concatenated arguments, dynamic components and scoped slot aliases in both
prefix settings. The payload mutation witness requires all three generated keys
to follow the changed checked operand. These seven shapes also require zero
legacy walks and zero expression reparses in the isolated floor binary.

Three exact generated-code and decoded-map snapshots compare every native and
retained mapping segment. They preserve the existing component mapping units;
they add no finer prop-key anchors. Existing allocation ceilings remain fixed.

Four mounted parent forms compare the pinned official compiler and runtime.
The exact traces cover argument replacement, old update-listener removal,
static value restoration, reactive modifiers with trimming, external updates,
repeated key changes, preserved child node identity and unmount. Computed
arguments on DOM models stay on the explicit legacy lane. Full P3-6 acceptance
remains open.

Contract: [P3-6](p3-6.md).
