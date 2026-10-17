# JSX attribute quotes

Choose JSX attribute quotes independently of ordinary JavaScript strings. Use this for JSX/TSX code, including Vue script blocks, when the two kinds of strings need different conventions.

Keep shared options in `vue.formatter` in `vite.config.*`; the CLI and editor read the same settings. See [Configuration](./configuration.md).

## Configure JSX quotes

Set `vue.formatter.jsxSingleQuote` to prefer single quotes in JSX/TSX attributes:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: { singleQuote: false, jsxSingleQuote: true },
});
```

`singleQuote` chooses ordinary JavaScript string delimiters. `jsxSingleQuote`
chooses JSX attribute delimiters independently; its default is `false`. For
example, the configuration above formats this script:

```text
const label = "hello";
const view = <div title='world' />;
```

The preference minimizes escaping. An attribute containing an apostrophe, such
as `title="it's"`, keeps double quotes to avoid an entity. On tied quote counts,
the preference applies: `title="' &quot;"` becomes `title='&apos; "'`.

The setting applies to JSX/TSX scripts, including Vue `<script lang="tsx">` and
`<script lang="jsx">` blocks. Ordinary Vue template HTML attributes retain their
existing quote policy. CLI/editor configuration and WASM SFC options expose the
setting; native Node formatter options do not expose `jsxSingleQuote` yet.

[Property quotes](./formatter-property-quotes.md) choose which object keys need
quotes and remain independent of JSX attribute quotes.
