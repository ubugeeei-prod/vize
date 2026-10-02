# Native component verbatim scope

This implements the component-profile `v-pre` consumer of the checked Vue
directive provider under [#6841](https://github.com/ubugeeei-prod/vize/issues/6841)
and [#6836](https://github.com/ubugeeei-prod/vize/issues/6836). Typed directive/embed dispatch,
document-profile construction and complete dialect selection remain unfinished.

## Ownership and the actual callback boundary

The native component constructor moves from `markup/parse.rs` to
`dialect/vue3/surface.rs` in a pure rename commit. The old public path is a narrow
export. Replay that move with:

```sh
vp node tools/support/levels/move-vue-surface-provider.ts move --root PATH
vp node tools/support/levels/move-vue-surface-provider.ts check --root PATH
```

The concrete Vue sink receives the generic lexer's events. It reconstructs each
complete authored attribute head from its first name piece to `AttrNameEnd`.
At open-tag end, after shared structural recovery, it consumes only that tag's
already recorded complete heads and calls `VueDirectives::decompose` with each
head's absolute byte offset. This uses no source rescan or scratch allocation. Plain
attributes are `Ok(None)`. A parsed full directive whose logical name is `pre`
marks the pending element, including heads with modifiers, arguments and
recoverable malformed arguments. Shorthand `:pre`, `@pre`, `#pre`, `.pre` and
similar full names do not enable the mode.

Inherited verbatim descendants do not invoke the unused directive hook or
misclassify ignored raw heads as unsupported syntax. The mode changes after the
open tag ends. Subsequent child interpolation and
directive callbacks are suppressed through the existing `Sink::mode` contract;
the generic lexer, profile and recorder contain no concrete Vue scope policy.
Native surface attributes retain every authored name and value, including the
control attribute, rather than dropping bytes to imitate the semantic AST.
Text entities remain authored source slices.

An admitted `v-pre` also makes the other heads on its own opening tag raw,
regardless of whether they precede or follow it. The one current-tag head scan
keeps an unsupported-list starting length and discards only facts added by that
tag once its control is proved; facts from earlier tags remain intact. Goldens
and the actual legacy AST prove both attribute orders. A sole over-budget
`v-pre` has no admitted control proof and retains its typed unsupported fact.

## One structural recovery implementation

`build/scope.rs` owns the generic namespace, interactive-element recovery and
redundant-end-tag decisions. Both the node builder and the live mode consumer
use it with their own frame payloads. The builder keeps its existing node stack
and one redundant-close list; this extraction adds no second builder stack or
pipeline stage. The mode consumer keeps arena frames for its live scopes.

Nested scopes inherit the current mode. Void and self-closing elements do not
leave it enabled for a following sibling. A matching ancestor close also ends
its descendants, while stray closes do not change the live owner. An implicitly
closed interactive element stops owning the newly opened sibling. Namespace
integration points use the same decisions as construction.

Normal and authored projections share this single recorded event stream. The
authored projection changes parent recovery, not the already recorded lexical
mode. Existing redundant-close recovery is preserved: a shallower newly opened
element can remain open when a close claims an older implicitly closed element.
Independent goldens and the actual legacy parser AST pin that behavior.

One native-only policy improvement is explicit: in
`<a v-pre><a v-pre>{{inner}}</a>{{tail}}</a>{{end}}`, the outer owner is implicitly
closed before the second element becomes an independent owner. Native resolves
the second element's own `v-pre` after recovery and suppresses `inner`. The actual
legacy AST still treats `inner` as interpolation because its control attribute
was read under the old owner's mode. An independent golden and a dev-only AST
test assert both results, rather than pretending they match. Production routes
and bytes retain the legacy behavior.

## Typed unsupported facts retain the recovered tree

The three native `parse_component*` APIs return
`Result<ComponentParse, ComponentSourceError>`. Davinci has no users, so this
intentional API change is allowed. The carrier contains:

- `tree`: the recovered lossless surface;
- `authored`: an optional non-repairing projection requested by the caller;
- `errors`: the existing ordered recoverable lexical diagnostics;
- `unsupported`: typed directive admission facts, with absolute head spans and
  the checked provider's `DirectiveNameError`.

A source larger than the `u32` coordinate space returns `SourceTooLarge` before
construction. A directive over the provider's initial 64 distinct delimiter-run
bound retains its raw attribute and all subsequent nodes, adds `NestingLimit`
to `unsupported`, and does not pretend that its scope was understood. It can
therefore still expose interpolation in unresolved content. A nonempty
`unsupported` list cannot establish native completion. Malformed admitted heads
remain recoverable syntax and are not blanket-rejected as invalid directives.

## Evidence and remaining work

Independent source and structure goldens cover logical names, modifier and
argument forms, Unicode coordinates, raw attributes, entities, nested owners,
void/self-closing elements, matching/stray closes, interactive recovery,
namespaces, typed unsupported retention and every UTF-8 cut of malformed scopes.
A dev-only actual Armature AST oracle compares admitted interpolation policy;
it does not supply production syntax or compare two recorders.

The existing native construction fixtures retain their complete tree,
diagnostic and hole comparisons for sources without the new dialect policy.
The generic policy-isolation and normal/build dependency-direction gates remain
in force. Actions and the 100 instruction ceilings must pass on the published
exact head and protected candidate; numeric ceilings may not increase.

Remaining work is explicit:

- Remove the directive provider's artificial distinct-run limit without losing
  its allocation and recovery contract.
- Connect typed Shape/grammar/language dispatch to the existing
  [native embed-source decoder](./2026-10-01-l1-embed-source.md) and
  [JS/TS syntax providers](./2026-10-01-l1-native-embed-syntax.md). Complete
  every Vue dialect, custom delimiters and document-profile construction.
- Factor the remaining directive-prefix dispatch from the generic lexer into
  an actual syntax policy boundary.
- Build the complete dialect/file descriptor and core legalization isolation.
- Complete each product's fix-history corpus and native parity gates before
  changing any production parse route.

The native API's existing dump roundtrip consumer renders the recovered tree
even with lexical errors or unsupported facts: success asserts authored byte
identity only. An explicit CLI law pins over-budget retention; source overflow
maps to a dump error before rendering.

The root `parse*` APIs, compiler routes and legacy semantics remain unchanged.
This slice completes admitted native component `v-pre` lexical scope; it does
not establish full Vue history, typed embedding or product acceptance.
