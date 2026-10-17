# Property quotes

Choose when script object keys need quotes. Use the preserve mode to retain authored quoted keys, or the other modes to make quoting follow the object structure.

Keep shared options in `vue.formatter` in `vite.config.*`; the CLI and editor read the same settings. See [Configuration](./configuration.md).

## Configure property quotes

Set `vue.formatter.quoteProps` in your existing Vite+ configuration to choose how
script object keys are quoted:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: { quoteProps: "preserve" },
});
```

The existing modes are:

- `"as-needed"` (default): quote keys that require quotes.
- `"consistent"`: quote all keys in an object when one requires quotes;
  remove optional quotes when none require them.
- `"preserve"`: keep whether each key was authored with quotes.

For example, `preserve` formats `{"name":1,plain:2}` as
`{ "name": 1, plain: 2 }`. `as-needed` and `consistent` both format that object
as `{ name: 1, plain: 2 }`. When an object contains `"needs-dash"`, `consistent`
quotes its sibling keys as well.

The `singleQuote` setting still chooses the quote character. `quoteProps`
controls which keys have quotes; it does not keep the original quote character.

The CLI and editor read the same `vue.formatter` settings from `vite.config.*`. See [Configuration](./configuration.md) for discovery and migration from removed dedicated configuration files.
