# Indentation width

Choose the indentation width that matches your project. Space or tab settings change the layout around template content; authored raw text stays intact.

Keep shared options in `vue.formatter` in `vite.config.*`; the CLI and editor read the same settings. See [Configuration](./configuration.md).

## Configure indentation

`tabWidth` controls the number of spaces per indentation level. Its default is
`2`. Set it in the existing formatter configuration:

```ts
// vite.config.ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: { tabWidth: 3 },
});
```

For `<template><p>hello</p></template>`, this produces:

```text
<template>
   <p>hello</p>
</template>
```

`useTabs: true` selects tabs for indentation instead. Quote preferences keep
their existing behavior. Whitespace inside raw template regions such as `<pre>`,
`<textarea>` and `v-pre` retains its authored bytes.
