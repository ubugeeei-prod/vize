# Script-setup emits validators

Issue: [#6097](https://github.com/ubugeeei-prod/vize/issues/6097).
Parent: [#8643](https://github.com/ubugeeei-prod/vize/pull/8643).

The authenticated public v0.443.0 CLI returns an empty diagnostic vector for
this runtime validator, even though it has no value-returning return:

```vue
<script setup lang="ts">
const emit = defineEmits({
  submit(payload) {
    void payload;
  },
});
</script>
```

The parent PR repairs Options API prefilters. This example also needs an AST
producer extension: the existing emits-object resolver only examines Options
API component exports. It must remain unchanged for its other consumers.

## Decision

Extend only `script/return-in-emits-validator` for compiler-owned, top-level
`defineEmits` calls in actual SFC script-setup blocks. Preserve direct standalone
script-rule dispatch and every other rule's existing SFC dispatch. Runtime
literal validators and safely resolved immutable object bindings use the same
validator diagnostic and own-return analysis as Options API validators.

The normal script block may own a binding that shadows `defineEmits`, or own
the constant validator object. Reuse its already parsed AST; when an actual
active setup-macro candidate requires the sibling and no other rule parsed it,
parse that block once in the existing script pass. An unavailable or malformed
sibling cannot prove that the macro name is unshadowed. Do not add fields to
the public `SfcScriptContext` struct or add a new all-rule forwarding stage.

Compiler ownership must account for value imports and declarations in both
blocks, including destructuring and hoisted `var`. Exact unaliased named Vue
macro imports remain compiler-owned; foreign and local value bindings do not.
Type imports, interfaces, aliases, and ambient declarations do not create
runtime bindings. Nonambient namespaces remain conservatively treated as
potential value bindings, including namespaces containing only interfaces or
aliases. That named false-negative limitation is pinned beside the existing
value-namespace suppression; namespace runtime-emission analysis is unfinished.
Nested calls, ordinary
JavaScript files, inline HTML scripts, unknown or mutable object bindings, and
type-only `defineEmits` declarations cannot produce validator findings.

## Delivery and qualification

This dependent PR is initially a draft; this record is a decision and work
boundary, not passing implementation evidence. The ready parent can enter the
protected queue as a native Stack prefix while this extension is qualified.

An external `src` on either script block cannot prove the loaded bindings or
inline compiler ownership. Only this macro extension receives unavailable
ownership evidence; existing Options and other-rule dispatch remain unchanged.
The extension never forces a parse of an external sibling's fallback content.

The implementation now has 51 source-authored positive and negative full
diagnostic vectors, UTF-8 and sibling offsets, preset and explicit-off controls,
and legacy fixtures. The separately retained authenticated public v0.443.0
baseline matches every complete before vector. Existing import-macro and local
reference findings remain in the vectors; following a setup const does not make
that forbidden hoisted reference valid. Source-build CLI Actions execute the
same cases across seven routes (357 packets) and preserve the original 51-rule
n8n projection. A namesake configured rule exercises its unchanged SFC dispatch.
The original 42 complete vectors remain unchanged; nine review controls cover
external blocks, module-chain scope, Vue alias calls, mixed type/runtime
arguments, constant cycles, and the conservative namespace limitation.

Source qualification exposed existing dispatcher file-length obligations and
the expanded rule description's generated references/localized explain pages.
Keep the unchanged dispatch bodies in separate modules in a move-only commit,
retain the public type and trait paths, and update only this rule's description
in all three native locales and their committed explain snapshots. Regenerate
the existing reference and migration manifests; no limit or expectation is
relaxed. The 357 complete source CLI packets passed on the prior source while
its unrelated qualification obligations were red, so the repaired head still
requires fresh exact-head/full qualification.

The conservative byte escape candidate also admits the existing Options
producer's escaped `emits` key; an explicit legacy fixture records this restored
finding. It never establishes ownership itself. Escaped identifier spelling is
decoded by the shared AST. The unrelated computed-name prefilter is unchanged.

Local package compilation and the two integration tests passed. Exact-head
Actions and protected corpus/instruction gates remain required. Public installed
acceptance requires a successful official release that includes the actual
merged change. #6097 and broader default-linter coverage remain unfinished until
their remaining scope is proven.

The final source replay follows actual main `006c564067`: the new protected
native phase workflow calls its retained syntax-semantics helper, which is
absent from the earlier parent-only source. The earlier exact `b494a46` native
run failed before measuring lint with exit 127 and is retained as failed
evidence. This replay preserves the owned canonical clause and all incoming
decisions, with fresh source/357-packet/protected qualification required. The
unchanged module relocation also refreshes the Croquis consumer shard's file
count (18 SfcDescriptor files, still 26 sites); no producer or budget changed.

The rule's public contract now names the existing supported Options and owned
inline setup runtime-object/immutable-const domain, explicitly excluding
unavailable external blocks, namespace-shadowed calls and all-path analysis.
The existing Complete tier describes that bounded domain; it grants no broad
#6097 or release completion. Three explain snapshots follow this source-authored
per-rule contract; the generated rules reference is unchanged and checks clean.
Other rule contracts and complete diagnostic vectors remain unchanged.
