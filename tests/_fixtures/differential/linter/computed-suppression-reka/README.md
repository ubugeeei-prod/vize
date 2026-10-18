# ReKa computed suppression

`NavigationMenuContentImpl.vue.fixture` preserves the complete original file
from `unovue/reka-ui` at the repository's pinned commit
`c23e0ed8a38c66cd4f9350e7a46c6b502a8491c2`, path
`packages/core/src/NavigationMenu/NavigationMenuContentImpl.vue`.
It is byte-identical to that upstream source, with SHA-256
`5a92b27acfc737c1a836327d182b74afbfa5293f66dfb07772ce49519442d5a6`.

The authored `vue/no-side-effects-in-computed-properties` disable-next-line
comment at line 74 must suppress the assignment at line 75. Removing only
that directive must restore the complete computed-side-effect diagnostic.
`crates/vize_patina/tests/computed_suppression_history.rs` runs both cases and
focused directive-scope controls. The rule implementation and the broad ReKa
snapshot are not weakened or rewritten by this fixture.
