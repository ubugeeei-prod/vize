---
title: "type/unbound-method"
description: "Disallow referencing methods without their receiver (`this`)."
---

# `type/unbound-method`

Disallow referencing methods without their receiver (`this`).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | No | [Rule options](../options.md) |

Applies to: Type information in Vue SFC scripts and templates, for the constructs shown below.

This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.

## Bad

counter.increment takes the method without its receiver, so this would be lost when it is called.

```vue annotate="remove:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
const unbound = counter.increment;
</script>
```

## Good

counter.increment() calls the method on counter, so this stays bound.

```vue annotate="add:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
counter.increment();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/unbound-method": "warn"
    },
    "typeAware": true
  },
});
```

Then run `vp run lint`.

## Differences from typescript-eslint

Library methods read from the global object are not reported. This covers a receiver typed `Window & typeof globalThis` or `typeof globalThis`, including `window.setTimeout`, `const { matchMedia } = window`, `self.requestAnimationFrame` and `document.defaultView?.setTimeout`. Platform operations use the global object when they are called without a receiver, so these references run as written. A method that your own code adds to `Window` is still reported. So are another frame's `Window` (`parent.setTimeout`) and a method of an object reached through the global (`window.localStorage.getItem`). Set `checkGlobalObjectMethods: true` to report these references as typescript-eslint does.

A built-in that is merged into (`declare global { interface Math {} }`) keeps its exemption, and `RegExp.escape` is exempt through any path to the `RegExp` constructor. The exemption follows the TypeScript library declarations, not the Node.js version that runs the linter.

## Related

- [Type and script rules](../type-and-script.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/unbound_method.rs#L5)
