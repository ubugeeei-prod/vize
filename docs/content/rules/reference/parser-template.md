---
title: "parser/template"
description: "Understand and repair template parser diagnostics reported by the linter."
---

# `parser/template`

The linter reports this diagnostic when the template parser finds malformed template syntax,
an invalid directive or interpolation expression, or an invalid `v-bind()` expression in style.
It is an operational diagnostic outside the preset rule catalogue. There is no automatic fix.

## When it is reported

The diagnostic keeps the parser's message and source location. In a Vue SFC, the location points
into the original `.vue` file, rather than starting again at the template's first character.
This includes malformed named character references such as `&amp` and `&unknown;`.
Parser errors classified as recoverable produce a warning; other parser errors produce an error.
Standard-mode compatibility notices for rewritten HTML self-closing syntax are omitted from lint reports.

Recovery and severity are separate decisions. A documented parser recovery can still produce an
error while allowing semantic rules to inspect the recovered tree. An error without a documented
recovery prevents those semantic checks from using an incomplete template tree.

## Bad

The `span` has no closing tag before the containing `div` closes.

```vue
<template><div><span>Notice</div></template>
```

## Good

Close the nested element before its parent.

```vue
<template>
  <div><span>Notice</span></div>
</template>
```

## How to repair it

Read the message at the reported location, then check the surrounding tags, attribute names,
quotes, character references, interpolation delimiters, and JavaScript expressions. Repair the
source and run lint again. Fixing the earliest
syntax problem can also remove later diagnostics caused by parser recovery.

Changing a lint preset does not repair the malformed template. If the message concerns the
outer SFC blocks, use [`parser/sfc`](./parser-sfc.md) instead.

## Related

- [Native lint execution settings](../../guide/lint-execution.md)
- [`parser/sfc`](./parser-sfc.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/parse_diagnostics.rs)
