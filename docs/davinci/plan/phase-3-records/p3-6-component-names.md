# P3-6 — Computed component prop and event names (2026-09-26)

Component `:[name]` and `@[name]` now retain their name expression AST in the
checked S3 payload. The emitter forwards the keys through the existing reactive
prop sources and event-key normalization. Static and computed source keys have
distinct duplicate identities. A computed `:[is]` remains a prop on a dynamic
component; only static `:is` selects its implementation. The retained lowerer
also fixes that ambiguity.

Seven source fixtures cover reference, indexed, call and concatenated names,
scoped slot aliases, named models, static/computed key collisions and dynamic
components in both prefix settings. A payload mutation snapshot proves both
generated keys come from checked operands. Five floor fixtures require zero
legacy walks and zero expression reparses in both prefix settings. The existing
allocation ceilings are unchanged.

Three exact code and decoded map snapshots pin equality between native and
retained generation. This preserves the existing component mapping units;
it does not add finer individual prop-key or prop-value anchors.

TS-33 compares three parent forms with the pinned official compiler. Exact
traces cover computed prop replacement, removal of old event handlers,
restoration of static prop values, repeated updates, stable child node identity
and unmount. Computed DOM prop names, slot outlet prop names, component event
modifiers, prop modifiers and computed model arguments remain explicit legacy
surfaces. P3-6's full acceptance remains open.

Contract: [P3-6](p3-6.md).
