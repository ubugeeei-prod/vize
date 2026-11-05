---
title: "parser/jsx"
description: "Understand JSX and TSX parsing diagnostics reported by the linter."
---

# `parser/jsx`

The linter uses this diagnostic for problems reported while parsing or lowering a JSX/TSX module.
It is an operational diagnostic outside the preset rule catalogue. There is no automatic fix.

## When it is reported

JSX/TSX input is parsed as a module before markup rules inspect its render roots.
Each diagnostic preserves the parser or lowering message and its source range.
An error from that stage produces an error; a warning produces a warning.
When lowering also reports an existing parser diagnostic with the same ID and range,
the linter removes the duplicate.

This diagnostic can describe invalid JavaScript/TypeScript syntax as well as malformed JSX.
Read the complete message to distinguish a syntax error from a construct the lowering stage rejects.

## Bad

The JSX `span` is not closed before the parent `div` closes.

```tsx annotate="remove:1"
export default () => <div><span>Notice</div>;
```

## Good

Close each JSX element in nesting order.

```tsx annotate="add:1"
export default () => (
  <div>
    <span>Notice</span>
  </div>
);
```

## How to repair it

Repair the syntax at the reported range, then run lint again. Check matching element names,
expression braces, quotes, and the surrounding module syntax. Keep JSX in a `.jsx` file and
TypeScript JSX in a `.tsx` file so the selected language matches the source.

If valid syntax is rejected during lowering, reduce the construct to a small reproduction and
report it with the complete message. Changing a lint preset does not make invalid syntax parse.

## Related

- [JSX & TSX](../../guide/jsx.md)
- [Native lint execution settings](../../guide/lint-execution.md)
- [Diagnostic conversion source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/jsx.rs)
- [JSX lint dispatch source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine.rs)
