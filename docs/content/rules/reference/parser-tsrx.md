---
title: "parser/tsrx"
description: "Understand TSRX module parsing and lowering diagnostics reported by the linter."
---

# `parser/tsrx`

The linter uses this diagnostic for problems reported while parsing a `.tsrx` module or lowering its markup regions.
It is an operational diagnostic outside the preset rule catalogue. There is no automatic fix.

## When it is reported

A `.tsrx` module is parsed once, host TypeScript and markup together. Each markup region is then lowered to the
same intermediate form as a JSX/TSX render root, and the markup rules check every region.

- A syntax or template-rule error is reported at its source range. A message from the TSRX grammar starts with the
  specification's code, such as `TSRX1002`. The first syntax error ends the parse, so the module's markup is not checked.
- A construct the lowering does not support yet, such as `@switch` or `@try`, is reported at that construct, and the
  module's markup is not checked. The message names the construct.

Both are errors.

## Bad

The `div` element is closed with `</span>`.

```tsx annotate="remove:2"
export function Notice() @{
  <div>Notice</span>
}
```

## Good

Close each element with its own name.

```tsx annotate="add:2"
export function Notice() @{
  <div>Notice</div>
}
```

## How to repair it

Repair the syntax at the reported range, then run lint again. Check matching element names, expression braces,
directive blocks, and the surrounding TypeScript.

For a construct that is not supported yet, the module stays unchecked until support lands. Keep the module out of
the lint selection with `ignores` if the error blocks CI, and report the construct with the complete message.
Changing a lint preset does not change either result.

## Related

- [Linting with the CLI](../../guide/cli.md#lint)
- [Native lint execution settings](../../guide/lint-execution.md)
- [Diagnostic conversion source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/tsrx.rs)
- [TSRX lowering source](https://github.com/ubugeeei-prod/vize/blob/main/davinci/vize_l2/src/lang/tsrx.rs)
