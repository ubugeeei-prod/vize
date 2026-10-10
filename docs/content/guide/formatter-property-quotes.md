# Property quotes

Set `fmt.vize.quoteProps` in your existing Vite+ configuration to choose how
script object keys are quoted:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  fmt: {
    vize: { quoteProps: "preserve" },
  },
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

For CLI/editor sharing and optional dedicated settings, see [Configuration](./configuration.md), including the current release availability.
