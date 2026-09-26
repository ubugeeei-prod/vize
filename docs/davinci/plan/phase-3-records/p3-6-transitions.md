# P3-6 - Controlled Transition and TransitionGroup contracts

The bounded native contract uses Vue 3.6.0-rc.9's published `VaporTransition`
and `VaporTransitionGroup`. Both compiler lanes preserve that builtin identity.
The previous retained output resolved these tags as user components.

Admission requires authored `:css="false"`. A Transition contains one implicit
HTML root, optionally carrying a single `v-if` without an else branch. A
TransitionGroup contains one keyed, direct HTML `v-for` root and a static
`tag="ul"` or `tag="div"`. Root tags are `p`, `button`, `div`, `span` and `li`;
children are text/interpolations. Root bindings are ordinary props and events. The known enter/leave lifecycle listener names
retain their authored expression ASTs.

CSS transitions, appearance, modes, dynamic tags, computed wrapper props,
spreads, components, named/scoped slots, multiple roots, nested elements or controls,
keyed Transition roots and other directives remain explicit legacy selections.
These restrictions describe this native contract, not every capability of the
published runtime. Full P3-6 acceptance remains open.

## Runtime boundary

The official compiler marks a conditional slot root with `createIf` flags
`129` (`TRUE_SINGLE_ROOT | SLOT_ROOT`) and a keyed group root with `createFor`
flags `40` (`IS_SINGLE_NODE | SLOT_ROOT`). Both Vize lanes preserve these flags
and the nonstable root slot function metadata. Nested HTML controls are outside
the admitted slice; the slot boundary flag does not leak into nested blocks.

## Evidence

The dedicated `davinci_transition_parity` binary compares native, retained and
official compiler output under the same rc.9 runtime against recorded exact
frames. The JS runner controls hook completion through the real `done`
callbacks and awaits Vue's scheduler. It uses no timing sleeps.

The single-root trace checks leave-pending DOM, completed removal, held enter,
reactive text and click delivery, completed enter and unmount during a pending
leave. The cancellation trace removes a root while its enter callback is held,
then calls that stale callback and checks that it cannot complete or resurrect
the cancelled enter. Final unmount and late callbacks leave the host empty.

The keyed group traces use both accepted container tags. They check pending
removal and addition, hook order, retained keyed node identities, text updates,
reordering, removal of every child and final teardown. Runtime warnings and
errors are asserted empty. CSS transform/move animation is not measured.

The compile contract covers both identifier prefix settings, exact parser
agreement and native/retained code equality. Source-map snapshots cover a
conditional root and keyed loop. Payload mutations change the checked group
container independently of the authored source and refuse a changed CSS
operand. The isolated floor binary checks zero legacy walks and expression
reparses; the seven existing allocation ceilings are unchanged.
