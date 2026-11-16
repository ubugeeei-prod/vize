---
title: Event listener options
---

# Event listener options

## `vue/v-on-event-hyphenation`

The original `"always"` and `"never"` strings remain valid. The object form defaults to
`{ style: "always", ignore: [], autofix: false }`. Each `ignore` entry is a case-sensitive literal substring,
so `"update:"` exempts `@update:modelValue`, while `"Update:"` and `"update:*"` do not.
Entries must be unique, must contain a non-whitespace character, and must not end in `:exit`.
The whole rule policy is replaced by a later matching configuration; `{}` clears inherited
exemptions and resets `autofix` to `false`. Options do not enable a disabled rule. `ignoreTags` is not exposed.

Set `autofix: true` to offer static event-argument edits, then use the CLI's `--fix` or the
host's fix command to apply them. The edit keeps `@`/`v-on:`, modifiers, and handler expressions.
Names containing `_` keep their diagnostic without a fix. In `never` mode the converter removes
a hyphen before an ASCII word character; a hyphen before a non-ASCII letter can remain and still
be reported. An unchanged edit leaves file bytes untouched.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/v-on-event-hyphenation": "warn" },
    ruleOptions: {
      "vue/v-on-event-hyphenation": { style: "always", ignore: ["update:"], autofix: true },
    },
  },
});
```

The [Vue rule options](./options-vue.md#vue-v-on-event-hyphenation) describe the checked listener forms.

<span id="vue-v-on-handler-style"></span>

## `vue/v-on-handler-style`

Vize's default, `"method"`, allows only a method reference such as `@click="save"` or
`@click="form.save"`. eslint-plugin-vue's styles are available as options with its semantics:

- `["method", "inline-function"]`, eslint-plugin-vue's default, prefers a method reference and
  allows an inline function where a method reference cannot be used: the handler reads a `v-for`
  or slot variable and does not just forward its own parameters, as in `() => pick(item)`.
- `["method", "inline"]` allows an inline handler such as `pick(item)` in that place instead.
- `"inline-function"` or `"inline"` requires that style for every handler.

`ignoreIncludesComment: true` leaves a handler that contains a comment out of the method
preference, as eslint-plugin-vue does; with `"method"` alone, such a handler is not reported. A
member reference such as `foo.bar` is never reported. Messages match eslint-plugin-vue.

```ts
const ruleOptions = {
  "vue/v-on-handler-style": { style: ["method", "inline-function"], ignoreIncludesComment: true },
};
```

Bad with `["method", "inline-function"]`:

```vue
<button @click="() => send()">Send</button>
<button @click="count++">Add</button>
```

Good with `["method", "inline-function"]`:

```vue
<button @click="send">Send</button>
<li v-for="item in items" :key="item.id"><button @click="() => pick(item)">Pick</button></li>
```
