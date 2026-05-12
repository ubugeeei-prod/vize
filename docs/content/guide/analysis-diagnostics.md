---
title: Analysis Diagnostics
---

# Analysis Diagnostics

This page is the diagnostic reference for the checks Vize can emit. Use it like an ESLint rule
index: find the rule or diagnostic code, see when it is enabled, and see the concrete pattern it
detects.

## How To Read This Page

`Default` is the severity used when the rule is enabled by a preset. `Enabled by` lists built-in
lint presets. `incremental` starts empty, so it only runs rules configured by the host.

`vize lint --cross-file` emits diagnostics with `vize:croquis/cf/*` codes instead of Patina rule
names. Those are listed separately because they are project-graph diagnostics, not single-file lint
rules.

## Patina Rules

These rules are part of the public lint rule name set used by `vize lint`, `vize.config.*`, and the
Oxlint bridge.

### a11y

| Rule                                     | Default | Enabled by                    | Detects                                                               |
| ---------------------------------------- | ------- | ----------------------------- | --------------------------------------------------------------------- |
| `a11y/alt-text`                          | warning | happy-path, nuxt, opinionated | Require alternative text for media elements                           |
| `a11y/anchor-has-content`                | warning | happy-path, nuxt, opinionated | Require anchor elements to have accessible content                    |
| `a11y/anchor-is-valid`                   | warning | happy-path, nuxt, opinionated | Enforce valid href on anchor elements                                 |
| `a11y/aria-props`                        | error   | happy-path, nuxt, opinionated | Disallow invalid ARIA attributes                                      |
| `a11y/aria-role`                         | error   | happy-path, nuxt, opinionated | Elements with ARIA roles must use a valid, non-abstract ARIA role     |
| `a11y/aria-unsupported-elements`         | error   | happy-path, nuxt, opinionated | Disallow ARIA attributes on elements that do not support them         |
| `a11y/click-events-have-key-events`      | warning | happy-path, nuxt, opinionated | Require keyboard event handlers with click events                     |
| `a11y/form-control-has-label`            | warning | happy-path, nuxt, opinionated | Require form controls to have associated labels                       |
| `a11y/heading-has-content`               | warning | happy-path, nuxt, opinionated | Require heading elements to have accessible content                   |
| `a11y/heading-levels`                    | warning | nuxt, opinionated             | Disallow skipping heading levels                                      |
| `a11y/iframe-has-title`                  | warning | happy-path, nuxt, opinionated | Require iframe elements to have a title attribute                     |
| `a11y/img-alt`                           | warning | happy-path, nuxt, opinionated | Require alt attribute on images for accessibility                     |
| `a11y/interactive-supports-focus`        | warning | happy-path, nuxt, opinionated | Require interactive role elements to be focusable                     |
| `a11y/label-has-for`                     | warning | happy-path, nuxt, opinionated | Require labels to have associated form controls                       |
| `a11y/landmark-roles`                    | warning | nuxt, opinionated             | Validate landmark role placement and uniqueness                       |
| `a11y/media-has-caption`                 | warning | happy-path, nuxt, opinionated | Require media elements to have captions                               |
| `a11y/mouse-events-have-key-events`      | warning | happy-path, nuxt, opinionated | Require focus/blur events with mouse events                           |
| `a11y/no-access-key`                     | warning | happy-path, nuxt, opinionated | Disallow the use of the accesskey attribute                           |
| `a11y/no-aria-hidden-on-focusable`       | error   | happy-path, nuxt, opinionated | Disallow `aria-hidden="true"` on focusable elements                   |
| `a11y/no-autofocus`                      | warning | happy-path, nuxt, opinionated | Disallow the use of the autofocus attribute                           |
| `a11y/no-distracting-elements`           | warning | happy-path, nuxt, opinionated | Disallow distracting elements like `<marquee>` and `<blink>`          |
| `a11y/no-i-for-icon`                     | warning | happy-path, nuxt, opinionated | Disallow using `<i>` element for icons                                |
| `a11y/no-redundant-roles`                | warning | happy-path, nuxt, opinionated | Disallow redundant ARIA roles                                         |
| `a11y/no-refer-to-non-existent-id`       | warning | happy-path, nuxt, opinionated | Disallow references to non-existent IDs                               |
| `a11y/no-role-presentation-on-focusable` | error   | happy-path, nuxt, opinionated | Disallow `role="presentation"` or `role="none"` on focusable elements |
| `a11y/no-static-element-interactions`    | warning | happy-path, nuxt, opinionated | Disallow event handlers on static elements                            |
| `a11y/placeholder-label-option`          | warning | nuxt, opinionated             | Require disabled or hidden on select placeholder option               |
| `a11y/role-has-required-aria-props`      | warning | happy-path, nuxt, opinionated | Require ARIA roles to have required properties                        |
| `a11y/tabindex-no-positive`              | warning | happy-path, nuxt, opinionated | Disallow positive tabindex values                                     |
| `a11y/use-list`                          | warning | nuxt, opinionated             | Suggest using list elements for bullet-like text                      |

### html

| Rule                             | Default | Enabled by                               | Detects                                             |
| -------------------------------- | ------- | ---------------------------------------- | --------------------------------------------------- |
| `html/deprecated-attr`           | warning | happy-path, nuxt, opinionated            | Disallow deprecated HTML attributes                 |
| `html/deprecated-element`        | warning | happy-path, nuxt, opinionated            | Disallow deprecated HTML elements                   |
| `html/id-duplication`            | error   | essential, happy-path, nuxt, opinionated | Disallow duplicate element IDs                      |
| `html/no-consecutive-br`         | warning | happy-path, nuxt, opinionated            | Disallow consecutive `<br>` elements                |
| `html/no-duplicate-dt`           | warning | happy-path, nuxt, opinionated            | Disallow duplicate `<dt>` names in `<dl>`           |
| `html/no-empty-palpable-content` | warning | happy-path, nuxt, opinionated            | Disallow empty elements that expect visible content |
| `html/require-datetime`          | warning | happy-path, nuxt, opinionated            | Require datetime attribute on `<time>` element      |

### script

These are built-in script rules wired into `vize lint` presets today. Additional script rules exist
in the Patina library, but they are not part of the public CLI preset surface yet.

| Rule                             | Default | Enabled by        | Detects                                                  |
| -------------------------------- | ------- | ----------------- | -------------------------------------------------------- |
| `script/no-get-current-instance` | error   | nuxt, opinionated | Disallow `getCurrentInstance()` in Vapor mode            |
| `script/no-next-tick`            | error   | nuxt, opinionated | Disallow `nextTick()` usage in Vapor-oriented components |
| `script/no-options-api`          | error   | nuxt, opinionated | Disallow Options API patterns in Vapor mode              |

### ssr

| Rule                            | Default | Enabled by                    | Detects                                                |
| ------------------------------- | ------- | ----------------------------- | ------------------------------------------------------ |
| `ssr/no-browser-globals-in-ssr` | warning | happy-path, nuxt, opinionated | Disallow browser-only globals in SSR context           |
| `ssr/no-hydration-mismatch`     | warning | happy-path, nuxt, opinionated | Disallow non-deterministic values that cause hydration |

### type

These are lint diagnostics that need semantic or checker-backed context. `type/no-reactivity-loss`
also runs when `--strict-reactivity` is passed.

| Rule                              | Default | Enabled by                    | Detects                                                  |
| --------------------------------- | ------- | ----------------------------- | -------------------------------------------------------- |
| `type/no-floating-promises`       | warning | nuxt, opinionated             | Disallow floating or unhandled Promises                  |
| `type/no-reactivity-loss`         | warning | nuxt, opinionated             | Disallow plain snapshots of reactive values across flows |
| `type/no-unsafe-template-binding` | warning | nuxt, opinionated             | Disallow template bindings that resolve to unsafe types  |
| `type/require-typed-emits`        | warning | happy-path, nuxt, opinionated | Require a type definition for `defineEmits`              |
| `type/require-typed-props`        | warning | happy-path, nuxt, opinionated | Require a type definition for `defineProps`              |

### vapor

| Rule                            | Default | Enabled by                    | Detects                                                              |
| ------------------------------- | ------- | ----------------------------- | -------------------------------------------------------------------- |
| `vapor/no-inline-template`      | error   | nuxt, opinionated             | Disallow deprecated `inline-template` attribute                      |
| `vapor/no-suspense`             | warning | nuxt, opinionated             | Warn about `Suspense` in Vapor-only apps                             |
| `vapor/no-vue-lifecycle-events` | error   | happy-path, nuxt, opinionated | Disallow `@vue:*` per-element lifecycle events                       |
| `vapor/prefer-static-class`     | warning | nuxt, opinionated             | Prefer static `class` over dynamic class binding for string literals |
| `vapor/require-vapor-attribute` | warning | nuxt, opinionated             | Suggest adding `vapor` attribute to `script setup`                   |

### vue

| Rule                                    | Default | Enabled by                               | Detects                                                      |
| --------------------------------------- | ------- | ---------------------------------------- | ------------------------------------------------------------ |
| `vue/attribute-hyphenation`             | warning | happy-path, nuxt, opinionated            | Enforce attribute naming style on custom components          |
| `vue/attribute-order`                   | warning | happy-path, nuxt, opinionated            | Enforce a consistent order of attributes                     |
| `vue/component-definition-name-casing`  | warning | happy-path, nuxt, opinionated            | Enforce PascalCase for component definition names            |
| `vue/component-name-in-template-casing` | warning | nuxt, opinionated                        | Enforce casing for component names in templates              |
| `vue/html-quotes`                       | warning | happy-path, nuxt, opinionated            | Enforce quote style of HTML attributes                       |
| `vue/html-self-closing`                 | warning | nuxt, opinionated                        | Enforce self-closing style                                   |
| `vue/multi-word-component-names`        | error   | essential, nuxt, opinionated             | Require component names to be multi-word                     |
| `vue/mustache-interpolation-spacing`    | warning | happy-path, nuxt, opinionated            | Enforce spacing inside mustache interpolations               |
| `vue/no-boolean-attr-value`             | warning | nuxt, opinionated                        | Disallow explicit values for boolean HTML attributes         |
| `vue/no-child-content`                  | error   | essential, happy-path, nuxt, opinionated | Disallow child content when using `v-html` or `v-text`       |
| `vue/no-dupe-v-else-if`                 | error   | essential, happy-path, nuxt, opinionated | Disallow duplicate conditions in `v-if` / `v-else-if` chains |
| `vue/no-duplicate-attributes`           | error   | essential, happy-path, nuxt, opinionated | Disallow duplicate attributes on the same element            |
| `vue/no-inline-style`                   | warning | nuxt, opinionated                        | Discourage inline `style` attributes                         |
| `vue/no-lone-template`                  | warning | happy-path, nuxt, opinionated            | Disallow unnecessary `<template>` elements                   |
| `vue/no-multi-spaces`                   | warning | happy-path, nuxt, opinionated            | Disallow multiple consecutive spaces                         |
| `vue/no-mutating-props`                 | error   | happy-path, nuxt, opinionated            | Disallow mutating component props                            |
| `vue/no-preprocessor-lang`              | warning | nuxt, opinionated                        | Discourage CSS preprocessors in favor of modern CSS          |
| `vue/no-reserved-component-names`       | error   | essential, happy-path, nuxt, opinionated | Disallow reserved names as component names                   |
| `vue/no-script-non-standard-lang`       | warning | nuxt, opinionated                        | Discourage non-standard script `lang` values                 |
| `vue/no-src-attribute`                  | warning | nuxt, opinionated                        | Discourage `src` attribute on SFC blocks                     |
| `vue/no-template-key`                   | error   | essential, happy-path, nuxt, opinionated | Disallow `key` attribute on `<template>`                     |
| `vue/no-template-lang`                  | warning | nuxt, opinionated                        | Discourage `lang` attribute on template block                |
| `vue/no-template-shadow`                | warning | nuxt, opinionated                        | Disallow template variables that shadow outer variables      |
| `vue/no-textarea-mustache`              | error   | essential, happy-path, nuxt, opinionated | Disallow mustache interpolation in `<textarea>`              |
| `vue/no-unsafe-url`                     | warning | essential, happy-path, nuxt, opinionated | Warn about potentially unsafe URL bindings                   |
| `vue/no-unused-components`              | warning | happy-path, nuxt, opinionated            | Disallow registered components unused in templates           |
| `vue/no-unused-properties`              | warning | happy-path, nuxt, opinionated            | Disallow unused properties defined in `defineProps`          |
| `vue/no-unused-vars`                    | warning | essential, happy-path, nuxt, opinionated | Disallow unused variables in `v-for` and `v-slot` directives |
| `vue/no-use-v-if-with-v-for`            | warning | essential, happy-path, nuxt, opinionated | Disallow `v-if` on the same element as `v-for`               |
| `vue/no-useless-template-attributes`    | error   | essential, happy-path, nuxt, opinionated | Disallow useless attributes on `<template>` elements         |
| `vue/no-v-html`                         | warning | essential, happy-path, nuxt, opinionated | Warn against `v-html` because of XSS risk                    |
| `vue/no-v-text-v-html-on-component`     | error   | essential, happy-path, nuxt, opinionated | Disallow `v-text` / `v-html` on component elements           |
| `vue/permitted-contents`                | error   | happy-path, nuxt, opinionated            | Enforce HTML content model rules                             |
| `vue/prefer-props-shorthand`            | warning | nuxt, opinionated                        | Recommend shorthand syntax for props                         |
| `vue/prop-name-casing`                  | warning | happy-path, nuxt, opinionated            | Enforce kebab-case prop names in templates                   |
| `vue/require-component-is`              | error   | essential, happy-path, nuxt, opinionated | Require `v-bind:is` on `<component>` elements                |
| `vue/require-component-registration`    | warning | opinionated                              | Require explicit import or registration for components       |
| `vue/require-scoped-style`              | warning | happy-path, nuxt, opinionated            | Require `scoped` attribute on style tags                     |
| `vue/require-v-for-key`                 | error   | essential, happy-path, nuxt, opinionated | Require `v-bind:key` with `v-for` directives                 |
| `vue/scoped-event-names`                | warning | nuxt, opinionated                        | Recommend scoped event names using `context:event` format    |
| `vue/sfc-element-order`                 | warning | happy-path, nuxt, opinionated            | Enforce consistent order of SFC top-level elements           |
| `vue/single-style-block`                | warning | happy-path, nuxt, opinionated            | Recommend a single style block                               |
| `vue/use-unique-element-ids`            | warning | nuxt, opinionated                        | Prefer `useId()` over static literal IDs                     |
| `vue/use-v-on-exact`                    | warning | essential, nuxt, opinionated             | Enforce `.exact` when modifier-based handlers coexist        |
| `vue/v-bind-style`                      | warning | nuxt, opinionated                        | Enforce `v-bind` directive style                             |
| `vue/v-on-style`                        | warning | happy-path, nuxt, opinionated            | Enforce `v-on` directive style                               |
| `vue/v-slot-style`                      | warning | happy-path, nuxt, opinionated            | Enforce `v-slot` directive style                             |
| `vue/valid-attribute-name`              | error   | essential, happy-path, nuxt, opinionated | Require valid attribute names                                |
| `vue/valid-v-bind`                      | error   | essential, happy-path, nuxt, opinionated | Enforce valid `v-bind` directives                            |
| `vue/valid-v-else`                      | error   | essential, happy-path, nuxt, opinionated | Enforce valid `v-else` directives                            |
| `vue/valid-v-for`                       | error   | essential, happy-path, nuxt, opinionated | Enforce valid `v-for` directives                             |
| `vue/valid-v-if`                        | error   | essential, happy-path, nuxt, opinionated | Enforce valid `v-if` directives                              |
| `vue/valid-v-memo`                      | error   | essential, happy-path, nuxt, opinionated | Enforce valid `v-memo` directives                            |
| `vue/valid-v-model`                     | error   | essential, happy-path, nuxt, opinionated | Enforce valid `v-model` directives                           |
| `vue/valid-v-on`                        | error   | essential, happy-path, nuxt, opinionated | Enforce valid `v-on` directives                              |
| `vue/valid-v-show`                      | error   | essential, happy-path, nuxt, opinionated | Enforce valid `v-show` directives                            |
| `vue/valid-v-slot`                      | error   | essential, happy-path, nuxt, opinionated | Enforce valid `v-slot` directives                            |
| `vue/warn-custom-block`                 | warning | nuxt, opinionated                        | Warn about custom blocks in SFC files                        |
| `vue/warn-custom-directive`             | warning | nuxt, opinionated                        | Warn about custom directives that need registration          |

### Bad / Good Examples

#### `vue/require-v-for-key`

Bad:

```vue
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

Good:

```vue
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

#### `vue/no-use-v-if-with-v-for`

Bad:

```vue
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

Good:

```vue
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

#### `vue/no-mutating-props`

Bad:

```vue
<script setup lang="ts">
const props = defineProps<{ count: number }>();
props.count++;
</script>
```

Good:

```vue
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

#### `vue/no-v-html`

Bad:

```vue
<template>
  <article v-html="content" />
</template>
```

Good:

```vue
<template>
  <article>{{ content }}</article>
</template>
```

#### `a11y/img-alt`

Bad:

```vue
<template>
  <img src="/avatar.png" />
</template>
```

Good:

```vue
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

#### `a11y/click-events-have-key-events`

Bad:

```vue
<template>
  <div role="button" @click="submit">Submit</div>
</template>
```

Good:

```vue
<template>
  <button type="button" @click="submit">Submit</button>
</template>
```

#### `ssr/no-browser-globals-in-ssr`

Bad:

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

Good:

```vue
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

#### `type/require-typed-props`

Bad:

```vue
<script setup lang="ts">
defineProps(["label", "count"]);
</script>
```

Good:

```vue
<script setup lang="ts">
defineProps<{
  label: string;
  count: number;
}>();
</script>
```

#### `type/no-unsafe-template-binding`

Bad:

```vue
<script setup lang="ts">
const payload: any = await load();
</script>

<template>
  <pre>{{ payload }}</pre>
</template>
```

Good:

```vue
<script setup lang="ts">
interface Payload {
  title: string;
}

const payload = await load<Payload>();
</script>

<template>
  <pre>{{ payload.title }}</pre>
</template>
```

#### `script/no-options-api`

Bad:

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

Good:

```vue
<script setup vapor lang="ts">
const count = ref(0);
</script>
```

#### `script/no-next-tick`

Bad:

```vue
<script setup vapor lang="ts">
import { nextTick } from "vue";

await nextTick();
</script>
```

Good:

```vue
<script setup vapor lang="ts">
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

## Cross-File Diagnostics

`vize lint --cross-file` currently enables `provide_inject`, `unique_ids`,
`reactivity_tracking`, and `race_conditions`. The analyzer uses the Croquis module registry and
component graph, then emits diagnostics back into the lint output as `cross-file` diagnostics.

### CLI-enabled cross-file checks

| Code                                                 | Analyzer              | Detects                                                                                      |
| ---------------------------------------------------- | --------------------- | -------------------------------------------------------------------------------------------- |
| `vize:croquis/cf/unmatched-inject`                   | `provide_inject`      | `inject()` keys without a reachable ancestor `provide()`                                     |
| `vize:croquis/cf/unused-provide`                     | `provide_inject`      | `provide()` keys that no descendant injects                                                  |
| `vize:croquis/cf/provide-inject-type`                | `provide_inject`      | Type mismatch between provided and injected values                                           |
| `vize:croquis/cf/provide-without-symbol`             | `provide_inject`      | `provide()` using string keys instead of symbols or `InjectionKey`                           |
| `vize:croquis/cf/inject-without-symbol`              | `provide_inject`      | `inject()` using string keys instead of symbols or `InjectionKey`                            |
| `vize:croquis/cf/non-reactive-provide`               | `provide_inject`      | Non-reactive values provided to descendants                                                  |
| `vize:croquis/cf/duplicate-id`                       | `unique_ids`          | Static IDs reused across analyzed components                                                 |
| `vize:croquis/cf/non-unique-id`                      | `unique_ids`          | Static IDs, or weak dynamic IDs, inside `v-for`                                              |
| `vize:croquis/cf/spread-breaks-reactivity`           | `reactivity_tracking` | Spreading reactive values into plain objects                                                 |
| `vize:croquis/cf/reassignment-breaks-reactivity`     | `reactivity_tracking` | Reassigning a reactive binding and losing the original reference                             |
| `vize:croquis/cf/value-extraction-breaks-reactivity` | `reactivity_tracking` | Extracting reactive values into plain variables                                              |
| `vize:croquis/cf/destructuring-breaks-reactivity`    | `reactivity_tracking` | Destructuring props/reactive objects without `toRef` or `toRefs`                             |
| `vize:croquis/cf/hydration-risk`                     | `reactivity_tracking` | Patterns that can diverge between SSR output and client hydration                            |
| `vize:croquis/cf/async-boundary`                     | `race_conditions`     | Reactive mutation after `await`, `setTimeout`, lifecycle async work, or Promise continuation |
| `vize:croquis/cf/watcheffect-async`                  | `race_conditions`     | Async work inside `watchEffect`                                                              |
| `vize:croquis/cf/injected-async-mutation-race`       | `race_conditions`     | Multiple consumers asynchronously mutating provider-owned injected state                     |

### Cross-file Bad / Good Examples

#### `vize:croquis/cf/inject-without-symbol`

Bad:

```ts
provide("theme", theme);
const theme = inject("theme");
```

Good:

```ts
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");

provide(ThemeKey, theme);
const theme = inject(ThemeKey);
```

#### `vize:croquis/cf/unmatched-inject`

Bad:

```ts
const theme = inject(ThemeKey);
```

Good:

```vue
<!-- Parent.vue -->
<script setup lang="ts">
provide(ThemeKey, theme);
</script>

<template>
  <Child />
</template>
```

```vue
<!-- Child.vue -->
<script setup lang="ts">
const theme = inject(ThemeKey);
</script>
```

#### `vize:croquis/cf/non-unique-id`

Bad:

```vue
<template>
  <div v-for="item in items" id="row">{{ item.name }}</div>
</template>
```

Good:

```vue
<template>
  <div v-for="item in items" :id="`row-${item.id}`">{{ item.name }}</div>
</template>
```

#### `vize:croquis/cf/destructuring-breaks-reactivity`

Bad:

```vue
<script setup lang="ts">
const { item } = defineProps<{ item: { count: number } }>();
</script>
```

Good:

```vue
<script setup lang="ts">
const props = defineProps<{ item: { count: number } }>();
const item = toRef(props, "item");
</script>
```

#### `vize:croquis/cf/async-boundary`

Bad:

```ts
const result = ref(null);

watch(query, async () => {
  result.value = await load(query.value);
});
```

Good:

```ts
const result = ref(null);

watch(query, async (value, _oldValue, onCleanup) => {
  let cancelled = false;
  onCleanup(() => {
    cancelled = true;
  });

  const next = await load(value);
  if (!cancelled) {
    result.value = next;
  }
});
```

#### `vize:croquis/cf/injected-async-mutation-race`

Bad:

```ts
const store = inject(StoreKey)!;

watch(query, async () => {
  await load();
  store.count = 1;
});
```

Good:

```ts
const emit = defineEmits<{ loaded: [value: number] }>();

watch(query, async (value) => {
  const next = await load(value);
  emit("loaded", next);
});
```

### Lower-level cross-file codes

The analyzer has additional options that are not all exposed by `vize lint --cross-file` yet. These
codes are already represented in the Croquis diagnostic model and can be surfaced as those analyzer
groups graduate to the CLI/config surface.

| Analyzer option            | Diagnostic codes                                                                                                                                                                                                                                                                                                                                                                                    |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fallthrough_attrs`        | `unused-attrs`, `inherit-attrs-unused`, `multi-root-attrs`                                                                                                                                                                                                                                                                                                                                          |
| `component_emits`          | `undeclared-emit`, `unused-emit`, `unmatched-listener`                                                                                                                                                                                                                                                                                                                                              |
| `event_bubbling`           | `unhandled-event`, `event-modifier`                                                                                                                                                                                                                                                                                                                                                                 |
| `server_client_boundary`   | `browser-api-ssr`, `async-no-suspense`, `hydration-risk`                                                                                                                                                                                                                                                                                                                                            |
| `error_suspense_boundary`  | `uncaught-error`, `missing-suspense`, `suspense-no-fallback`                                                                                                                                                                                                                                                                                                                                        |
| `circular_dependencies`    | `circular-dep`, `deep-import`                                                                                                                                                                                                                                                                                                                                                                       |
| `component_resolution`     | `unregistered-component`, `unresolved-import`                                                                                                                                                                                                                                                                                                                                                       |
| `props_validation`         | `undeclared-prop`, `missing-required-prop`, `prop-type-mismatch`, `undefined-slot`                                                                                                                                                                                                                                                                                                                  |
| `setup_context`            | `reactivity-outside-setup`, `lifecycle-outside-setup`, `watcher-outside-setup`, `di-outside-setup`, `composable-outside-setup`, `setup-context-violation`                                                                                                                                                                                                                                           |
| reactivity strict families | `reference-escapes-scope`, `mutated-after-escape`, `circular-reactive-dependency`, `watch-can-be-computed`, `dom-access-without-next-tick`, `computed-side-effects`, `module-scope-reactive`, `template-ref-timing`, `closure-captures-reactive`, `object-identity-comparison`, `reactive-export`, `shallow-deep-access`, `toraw-mutation`, `event-listener-leak`, `array-mutation`, `pinia-getter` |

## Type Checker Diagnostics

`vize check` is not a rule runner. It generates virtual TypeScript through Canon, runs Corsa
project diagnostics, and maps TypeScript diagnostics back to the original Vue, TS, TSX, or `.d.ts`
file.

It can surface TypeScript diagnostics for:

- expressions inside interpolations, `v-if`, `v-for`, `v-bind`, and `v-on`
- bindings exposed from `<script setup>` and classic `setup()`
- `defineProps`, `withDefaults`, `defineEmits`, `defineSlots`, and template refs
- scoped slot bindings, dynamic components, and `v-model` expressions
- ordinary TypeScript files included by the selected `tsconfig.json`

Bad:

```vue
<script setup lang="ts">
const user = { name: "Ada" };
</script>

<template>
  {{ user.missingProperty }}
</template>
```

Good:

```vue
<script setup lang="ts">
const user = { name: "Ada" };
</script>

<template>
  {{ user.name }}
</template>
```

Run with virtual output when you need to debug the mapped TypeScript:

```bash
vize check --tsconfig tsconfig.app.json --show-virtual-ts src/App.vue
```

## Musea And CSS Library Checks

These checks exist as Patina library analyzers. Musea rules are used by Art-file tooling; CSS rules
are available through the CSS linter surface but are not part of the public `vize lint` preset rule
set shown above.

| Rule                            | Default | Detects                                                      |
| ------------------------------- | ------- | ------------------------------------------------------------ |
| `musea/no-empty-variant`        | warning | Empty `<variant>` blocks                                     |
| `musea/prefer-design-tokens`    | warning | Hardcoded primitive values when design tokens are configured |
| `musea/require-component`       | warning | Missing `component` attribute in `<art>` block               |
| `musea/require-title`           | error   | Missing `title` attribute in `<art>` block                   |
| `musea/unique-variant-names`    | error   | Duplicate variant names                                      |
| `musea/valid-variant`           | error   | Missing `name` attribute in `<variant>` blocks               |
| `css/no-display-none`           | warning | `display: none` patterns where `v-show` may be clearer       |
| `css/no-hardcoded-values`       | warning | Hardcoded CSS values that could use variables                |
| `css/no-id-selectors`           | warning | ID selectors in component CSS                                |
| `css/no-important`              | warning | `!important` declarations                                    |
| `css/no-utility-classes`        | warning | Utility-class definitions inside component styles            |
| `css/no-v-bind-performance`     | warning | Expensive CSS `v-bind()` patterns                            |
| `css/prefer-logical-properties` | warning | Physical CSS properties where logical properties fit better  |
| `css/prefer-nested-selectors`   | warning | Descendant selectors that can use CSS nesting                |
| `css/prefer-slotted`            | warning | Slot-content styling that should use `::v-slotted()`         |
| `css/require-font-display`      | warning | `@font-face` rules without `font-display`                    |

### Musea And CSS Bad / Good Examples

#### `musea/require-title` and `musea/valid-variant`

Bad:

```vue
<art component="./Button.vue">
  <variant>
    <Button>Save</Button>
  </variant>
</art>
```

Good:

```vue
<art title="Button" component="./Button.vue">
  <variant name="default">
    <Button>Save</Button>
  </variant>
</art>
```

#### `musea/unique-variant-names`

Bad:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

Good:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

#### `css/no-important`

Bad:

```vue
<style scoped>
.button {
  color: red !important;
}
</style>
```

Good:

```vue
<style scoped>
.button {
  color: var(--color-danger);
}
</style>
```
