---
title: "parser/sfc"
description: "Repair single-file component structure errors reported by the linter."
---

# `parser/sfc`

The linter reports this error when it cannot parse the structure of a Vue single-file component.
It is an operational diagnostic outside the preset rule catalogue. There is no automatic fix.

## When it is reported

The SFC parser must identify complete top-level blocks before their contents can be checked.
For example, a missing closing tag for a top-level `template` block produces `parser/sfc`.
The diagnostic retains the SFC parser's message and points into the original source.
The failed parse does not supply a valid SFC descriptor to subsequent checks.

Art variant processing also uses this error if a variant cannot be located in its original SFC
source. That failure has a zero-length location at the file's start. If it persists with valid
source, preserve the `.art.vue` file and report the failure; suppressing a rule cannot restore
the missing source relationship.

## Bad

The outer `template` block is never closed, even though the inner element is complete.

```vue
<template>
  <div>Notice</div>
</template>
```

## Good

Close the outer block so the SFC parser can identify the complete template.

```vue
<template>
  <div>Notice</div>
</template>
```

## How to repair it

Check the top-level block named in the message and restore its required closing tag.
Also check whether a damaged quote or an incomplete edit made later source look like part of
that block. Run lint again after the SFC structure is valid.

If the SFC blocks are complete and the error is inside the template, consult
[`parser/template`](./parser-template.md). Changing a lint preset does not repair the SFC structure.

## Related

- [Native lint execution settings](../../guide/lint-execution.md)
- [`parser/template`](./parser-template.md)
- [SFC diagnostic source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/parse_diagnostics.rs)
- [Art variant diagnostic source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/engine/sfc/art.rs)
