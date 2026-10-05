# Property quotes

Set `formatter.quoteProps` in `vize.config.*` to choose how script object keys
are quoted. CLI and editor formatting use the same setting:

```ts
import { defineConfig } from "vize";

export default defineConfig({
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
