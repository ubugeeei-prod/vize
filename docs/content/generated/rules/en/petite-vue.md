---
title: "petite-vue rules"
---

# petite-vue rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [Bad](#petite-vue-no-unsupported-directive-bad) · [Good](#petite-vue-no-unsupported-directive-good) | Disallow directives that petite-vue does not support |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [Bad](#petite-vue-valid-v-effect-bad) · [Good](#petite-vue-valid-v-effect-good) | Require v-effect to have a non-empty expression |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [Bad](#petite-vue-valid-v-scope-bad) · [Good](#petite-vue-valid-v-scope-good) | Require v-scope to bind an object literal |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)

### `petite-vue/no-unsupported-directive`

Disallow directives that petite-vue does not support

[Bad](#petite-vue-no-unsupported-directive-bad) · [Good](#petite-vue-no-unsupported-directive-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-no-unsupported-directive-bad"></span>

**Bad**

`v-memo`, `v-slot:header`, and the custom `v-my-directive` are absent from petite-vue’s supported directive list. The petite-vue script marks this HTML as the relevant dialect.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-no-unsupported-directive-good"></span>

**Good**

The replacement uses supported `v-scope`, `v-effect`, `v-if`, `v-bind`, and `v-on` syntax instead of relying on unsupported directives.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [All rules](all.md)

### `petite-vue/valid-v-effect`

Require v-effect to have a non-empty expression

[Bad](#petite-vue-valid-v-effect-bad) · [Good](#petite-vue-valid-v-effect-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-effect": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-effect-bad"></span>

**Bad**

Each `v-effect` has no executable expression: its value is missing, empty, or only whitespace.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-effect-good"></span>

**Good**

Both `v-effect` values contain an expression: one updates `el.textContent`, and the other increments `count`. This rule checks for a nonempty expression, not the effect’s business logic.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [All rules](all.md)

### `petite-vue/valid-v-scope`

Require v-scope to bind an object literal

[Bad](#petite-vue-valid-v-scope-bad) · [Good](#petite-vue-valid-v-scope-good)

Default severity: `error`  
Presets: _none_  
Automatic fix: None; review the suggested change  
Applies to: HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.  
Options: No rule-specific options. Severity and preset selection are configurable.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-scope-bad"></span>

**Bad**

The four nonempty `v-scope` values are an identifier, a call, arithmetic, and a number; none parses as an object literal.

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-scope-good"></span>

**Good**

A valueless `v-scope` uses the root scope. The other values are object literals, including the parenthesized object, which the rule accepts.

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [All rules](all.md)
