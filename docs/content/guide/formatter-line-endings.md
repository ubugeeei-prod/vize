# Formatter line endings

Keep formatter line endings consistent across a Vue file. Use an explicit mode for a project convention, or auto mode to follow the first source line ending without rewriting raw text.

Keep shared options in `vue.formatter` in `vite.config.*`; the CLI and editor read the same settings. See [Configuration](./configuration.md).

## Configure line endings

Set `vue.formatter.endOfLine` in your existing Vite+ configuration:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  formatter: { endOfLine: "auto" },
});
```

The modes are `"lf"` (default), `"crlf"`, `"cr"` and `"auto"`. Auto selects
the first source terminator; an adjacent CRLF selects CRLF, a lone CR selects
CR, and LF or no terminator selects LF. A Vue file selects its layout endings
once for all template, script and style blocks. Explicit modes take precedence
over the input's layout. Authored raw text keeps its original bytes.

Node and WASM accept the same values in their existing formatter options:

```ts
import { formatSfc } from "@vizejs/native";

const source = "<template>\r\n<p>hello</p>\n</template>\n";
const result = formatSfc(source, { endOfLine: "auto" });
const again = formatSfc(result.code, { endOfLine: "auto" });
// again.changed === false
```

Node rejects unsupported mode strings with `InvalidArg` and non-string values
with `StringExpected`. Both diagnostics name the `endOfLine` field. Formatting
returns the existing `code` and `changed` fields.

The CLI and editor read the same `vue.formatter` settings from `vite.config.*`. See [Configuration](./configuration.md) for discovery and migration from removed dedicated configuration files.
