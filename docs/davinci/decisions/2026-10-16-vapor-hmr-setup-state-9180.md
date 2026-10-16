# Return Vapor setup bindings to the runtime (#9180)

[naitokosuke's report](https://github.com/ubugeeei-prod/vize/issues/9180) is a
repeatable third-party development crash: editing a child makes the parent
render read an undefined setup context. It takes precedence over roadmap work
under the third-party delivery policy. It was triaged as `priority:p0` during
this repair; it was not already labeled when intake began.

The historical report uses Vize 0.443.0 and Vue 3.6.0-rc.10. Preserve its original
application revision, source blob identities, scenario, and version boundary in
[`historical-report.md`](../../../tests/_fixtures/differential/compiler/vapor-hmr-9180/historical-report.md).
The runtime regression uses the independently authored minimal parent and child
in that directory, with the currently supported Vue 3.6.0-rc.11. It does not
rewrite the older rc.9 runtime fixtures or claim execution of the rc.10 app.

Vapor's public `getCurrentInstance()` intentionally does not expose its instance.
Returning an already rendered block from setup therefore leaves `setupState`
unset, even though the component has a separate render function. Child HMR calls
that render again and fails. Return the setup bindings instead, allowing Vue to
initialize its own setup-state proxy and render scope. Remove the obsolete
instance helpers and setup-context arguments from both sync and async output.
Keep the collision-safe render alias and template-ref setter behavior.

The focused Rust target compiles Vapor modules from the checked-out Vize source
and runs Vue's real `__VUE_HMR_RUNTIME__.reload` API under Happy DOM, not a
Chromium/dev-server replay. The independent oracle is
the official rc.11 VDOM compiler under the same runtime. Both direct child and
KeepAlive/dynamic-child cases verify initial DOM, two child script edits,
retained parent state, reactive text/props after each edit, complete observed
DOM/namespaces, event delivery, removed old child nodes, and unmount. Production
inline-template controls exercise the same inputs without HMR. Runtime errors,
warnings, missing phases, and failed subprocesses fail the test.

Before the source fix, the development case on
`b4be3b70977b80a689c8e5ada62d05283e86b868` failed with the exact reported
`Cannot read properties of undefined (reading 'inRoom')`; the production control
passed. The source fix makes both cases pass. The runtime pack is legacy compiler
regression coverage and adds no whole-product native coverage claim.

The existing 21 SFC Vapor unit tests also pass after updating only the 11
snapshots whose setup contract changes. The separate `vapor_runtime_contracts`
template-ref group locally has the same three passes and four production-inline
failures on both the untouched base and this patch. Those tests load rc.9;
the failing loop cases render proxy internals as list entries. In contrast,
all seven cases passed in the unchanged hosted workspace's
[merge-queue run](https://github.com/ubugeeei-prod/vize/actions/runs/38872907951).
The local/hosted difference remains unexplained; these tests stay unchanged.
Final Actions validation must establish the patched hosted result.

## Remaining acceptance

Actions on the final source commit and the protected merge queue must succeed.
After publication, verify the installed public Vite plugin with the supported
Vue runtime through a real dev server and repeated child saves. The reporter's
full RouterView/application scenario and older rc.10 runtime remain separate
acceptance inputs. Source tests alone do not close the external report.

## Separate finding retained for follow-up

TODO: investigate Vize VDOM child root-text updates separately. Compiling these
same parent script and `Child.vue.txt` with `vapor: false`, with either
`SeparateTemplate` development output or `InlineTemplate` production output,
mounts correctly but the first increment leaves the child at `room-0:first`
instead of `room-1:first`. The parent button becomes `1`. This was observed on
the base above, before changing production source, using Vue 3.6.0-rc.11 and the
same Happy DOM runner. That reproduction used this parent template before the
HMR fixture moved its conditional onto the section:

```vue
<section><button id="increment" @click="increment">{{ count }}</button><template v-if="inRoom"><Child :label="label" /></template></section>
```

The emitted child root is
`_createElementBlock("p", _hoisted_1, _toDisplayString($props.label) + ":" + _toDisplayString($setup.revision))`
in dev (and `__props.label` / `revision` in production), without a dynamic-text
patch flag. Preserve this failing source and assertion for a separate issue;
this repair uses the official VDOM compiler as its independent oracle and does
not alter the VDOM implementation.

Reporter credit was verified against the public report/profile and the cited
application commit: `Co-authored-by: ナイトウコウスケ <102337893+naitokosuke@users.noreply.github.com>`.
