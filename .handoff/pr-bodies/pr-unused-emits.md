## Summary

`script/no-unused-emit-declarations` reported events as never emitted in three situations where they can be. In the Real Project Matrix (#12336) that produced:

- **The template passes the emit function on.** In reka-ui stories (`_PinInput.vue`, `_DismissableLayer.vue`, `_Accordion.vue`, `_DropdownMenu.vue`, `_Menubar.vue`), `v-bind="{ ...props, ...useEmitAsProps(emits) }"` hands the emit function to other code. eslint-plugin-vue clears every declaration when the emit function is a call argument; Patina only did so for script references.
- **The instance escapes.** In element-plus `table.vue`, `const table = getCurrentInstance()` exposes `table.emit`, and other modules of the table call it for 18 events.
- **The event name is computed.** splitpanes forwards `emitEvent(name)` to `emit(name, …)` (9 events), mealie calls `emit(UPLOAD_EVENT, …)` (11), and frappe-builder, alexandrie, directus and n8n have similar calls. Every one of those events was reported. `test_dynamic_emit_argument_does_not_mark` pinned this, although the event in that test is emitted.

The rule now treats every declared event as possibly emitted when any of these holds:
- a template directive value or interpolation uses the emit binding, or `$emit`, as a value rather than calling it. A bare `@click="emit"` handler still counts as unused, since Vue calls it with the native event, and prose in the template is not read.
- the script calls `getCurrentInstance()`.
- a call of the binding in script or template takes a computed first argument. A name the rule can still resolve marks exactly its events: a literal, a static template literal, a conditional between such names, a top-level `const`, or a static key of a `const` object (`as const` or not).

Each change only makes the rule report less. Events emitted by literal name, and events that stay unused without an escape, are reported as before.

eslint-plugin-vue counts only literal names and has no instance escape. The tooling commits classify its findings when the setup block calls `getCurrentInstance()`, when a call has a computed name, or when Patina resolves the name. The classifier mirrors Patina's reading: top-level consts only, and in the template only directive values and interpolations.

## Tests

- `template::an_emit_function_passed_on_from_the_template_may_emit_every_event`: `useEmitAsProps(emits)` in `v-bind`, `:emit="emits"` and `:forward="$emit"` report nothing. Prose naming the binding, next to one direct call, still reports the other event.
- `template::get_current_instance_may_emit_every_event`.
- `test_a_computed_emit_name_may_emit_every_event` replaces `test_dynamic_emit_argument_does_not_mark`. `test_a_resolvable_emit_name_marks_exactly_its_events` covers a const, a const-object key and a conditional, which mark their events while `unused` is still reported. `test_sfc_template_dynamic_event_name_may_emit_every_event` flips the template twin.
- The existing `test_sfc_template_bare_reference_does_not_count` (`@click="emit"`) still reports.
- `lint-unused-emit-escapes.test.ts` covers both sides:
  - classified: the instance call; computed script and template calls; const, object-key and conditional names;
  - not classified: literal-only files, template prose, and a commented, member or sibling-script instance call.
  - A const nested in a function stays computed, as Patina reads it.
- Differential cases `linter/unused-emits-escapes-12336` and `linter/computed-emit-names-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
