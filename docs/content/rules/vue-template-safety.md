---
title: "Vue Rules: Template Safety"
---

# Vue Rules: Template Safety

HTML content, duplicate attributes, URL safety, and reusable element IDs.
See [all Vue rules](./vue.md) for the complete reference and [Vue Rule Options](./options-vue.md)
for configurable settings.

## `vue/no-v-html`

Reports `v-html` because it renders raw HTML and can turn user-controlled content into an XSS sink.

Default severity: `warning`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

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

## `vue/no-duplicate-attributes`

Reports duplicate attributes on the same element.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <button class="primary" class="large">Save</button>
</template>
```

Good:

```vue
<template>
  <button class="primary large">Save</button>
</template>
```

## `vue/no-unsafe-url`

Reports URL bindings and static URL attributes that may resolve to unsafe schemes such as
`javascript:`, `vbscript:`, or executable `data:` payloads.

Default severity: `warning`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <iframe src="javascript:alert(1)"></iframe>
  <object data="data:text/html,<script>alert(1)</script>"></object>
  <img srcset="/safe.png 1x, javascript:alert(1) 2x" />
  <a :href="nextUrl">Continue</a>
</template>
```

Good:

```vue
<script setup lang="ts">
const rawNextUrl = ref("/next");
const nextUrl = computed(() => {
  return rawNextUrl.value.startsWith("/") ? rawNextUrl.value : "/";
});
</script>

<template>
  <iframe src="/embedded/report" title="Report"></iframe>
  <img srcset="/avatar.png 1x, /avatar@2x.png 2x" />
  <a :href="nextUrl">Continue</a>
</template>
```

## `vue/use-unique-element-ids`

Reports static literal IDs in places where `useId()` is safer for component reuse and SSR.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

Good:

```vue
<script setup lang="ts">
const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

## `vue/no-textarea-mustache`

Reports mustache interpolation inside `<textarea>`. The text does not bind
the control. Use `v-model`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <textarea>{{ message }}</textarea>
</template>
```

Good:

```vue
<template>
  <textarea v-model="message"></textarea>
</template>
```

## `vue/no-v-text-v-html-on-component`

Reports `v-text` or `v-html` on a component. Those directives replace the
component's own output. A native element, or `<component is="div">`, may use
them.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

Good:

```vue
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

## `vue/permitted-contents`

Reports HTML nesting the parser or the content model forbids, so the DOM and
the virtual DOM would disagree. An unresolved component, a slot, or a dynamic
binding stays silent rather than guessing.

Default severity: `error`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

Good:

```vue
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```
