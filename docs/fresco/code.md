# Code viewer

`Code` renders source and streamed logs without adding a syntax-highlighting dependency to the
core package. It supports a bounded viewport, controlled grapheme selection, diagnostics, and a
synchronous provider contract for optional syntax packages.

## Stream bounded content

```ts
import { createCodeBuffer } from "@vizejs/fresco";

const output = createCodeBuffer("Starting…", { maxLines: 10_000 });
output.append("\nReady");
```

```vue
<Code :buffer="output" :offset="offset" :viewport-size="30" :overscan="3" />
```

The line-retention limit bounds the source buffer. `viewportSize` independently bounds the Vue
host nodes to the visible rows plus overscan.

## Add a syntax provider

Providers return tokens whose text must concatenate to the original line. Fresco falls back to a
plain token when a provider returns an incomplete line, so extensions cannot drop source text.

```ts
import type { CodeSyntaxProvider } from "@vizejs/fresco";

export const provider: CodeSyntaxProvider = {
  tokenize(line, context) {
    if (context.language !== "log") return [{ text: line }];
    return line.startsWith("ERROR")
      ? [{ text: "ERROR", fg: "red", bold: true }, { text: line.slice(5) }]
      : [{ text: line }];
  },
};
```

Selections use zero-based line and grapheme columns. Diagnostics use the same coordinates, making
emoji and combining characters safe across keyboard navigation and highlighted ranges.
