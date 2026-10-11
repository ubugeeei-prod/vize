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
Type-only declarations do not create runtime bindings. Nested calls, ordinary
JavaScript files, inline HTML scripts, unknown or mutable object bindings, and
type-only `defineEmits` declarations cannot produce validator findings.

## Delivery and qualification

This dependent PR is initially a draft; this record is a decision and work
boundary, not passing implementation evidence. The ready parent can enter the
protected queue as a native Stack prefix while this extension is qualified.

The implementation now has 42 source-authored positive and negative full
diagnostic vectors, UTF-8 and sibling offsets, preset and explicit-off controls,
and legacy fixtures. The separately retained authenticated public v0.443.0
baseline matches every complete before vector. Existing import-macro and local
reference findings remain in the vectors; following a setup const does not make
that forbidden hoisted reference valid. Source-build CLI Actions execute the
same cases across seven routes (294 packets) and preserve the original 51-rule
n8n projection. A namesake configured rule exercises its unchanged SFC dispatch.

The conservative byte escape candidate also admits the existing Options
producer's escaped `emits` key; an explicit legacy fixture records this restored
finding. It never establishes ownership itself. Escaped identifier spelling is
decoded by the shared AST. The unrelated computed-name prefilter is unchanged.

Local package compilation and the two integration tests passed. Exact-head
Actions and protected corpus/instruction gates remain required. Public installed
acceptance requires a successful official release that includes the actual
merged change. #6097 and broader default-linter coverage remain unfinished until
their remaining scope is proven.
