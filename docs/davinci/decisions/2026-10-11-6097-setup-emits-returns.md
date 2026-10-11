# Script-setup emits validators

Issue: [#6097](https://github.com/ubugeeei-prod/vize/issues/6097).
Parent: [#8643](https://github.com/ubugeeei-prod/vize/pull/8643).

The authenticated public v0.443.0 CLI returns an empty diagnostic vector for
this runtime validator, even though it has no value-returning return:

```vue
<script setup lang="ts">
const emit = defineEmits({ submit(payload) { void payload } })
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

Before the child is ready: add source-authored positive and negative full
diagnostic vectors, UTF-8 and sibling offsets, explicit-off and preset controls,
legacy fixtures, and source-build CLI evidence on Actions. Preserve the existing
corpus and instruction gates. Public installed acceptance requires a successful
official release that includes the actual merged change. #6097 and broader
default-linter coverage remain unfinished until their remaining scope is proven.
