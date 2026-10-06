## Area

Linter, `css/no-utility-classes`

## Version

`vize` 0.432.0

#7210 fixed `css/prefer-slotted` for words inside CSS comments. `css/no-utility-classes` has the
same problem and is unchanged in 0.432.0.

## Minimal reproduction

`MyNote.vue`

```vue
<template>
  <p class="my-note">note</p>
</template>

<style scoped>
/*
 * This replaces the old `.app .pl-3` helper. The `.mt-2` helper
 * is not needed any more.
 */
.my-note {
  padding-left: 12px;
}
</style>
```

`vize.config.json`

```json
{ "linter": { "preset": "incremental", "rules": { "css/no-utility-classes": "warn" } } }
```

```sh
vize lint -f plain --help-level none MyNote.vue
```

## Actual

```
Patina lint report: 1 warning in 1 file

MyNote.vue
  MyNote.vue:7:32 warning css/no-utility-classes Utility class should be in global styles, not component styles
```

`7:32` is `.pl-3` inside the `/* … */` comment. (`.mt-2` is not reported only because it follows a
backtick rather than a space.)

The same happens inside a string value:

`MyQuote.vue`

```vue
<template>
  <p class="my-quote">quote</p>
</template>

<style scoped>
.my-quote::before {
  content: " .flex ";
}
</style>
```

```
MyQuote.vue
  MyQuote.vue:7:14 warning css/no-utility-classes Utility class should be in global styles, not component styles
```

## Expected

No diagnostic in either file: neither stylesheet has a `.pl-3` / `.flex` selector. A real `.pl-3 { … }` rule should still be
reported (it is, e.g. `.pl-3 { padding-left: 12px; }` → `6:1`).

## Cause (from reading the source)

`crates/vize_patina/src/rules/css/no_utility_classes.rs` runs `memmem` over the raw `<style>` text
for patterns like `.pl-` / `.flex` and accepts a match when the previous byte is whitespace, `{`,
`}` or `,`. The parsed `StyleSheet` is passed in but unused (`_stylesheet`), so comments and string values
are scanned too. Walking the parsed selectors, as the
#7210 fix did for `css/prefer-slotted`, would avoid this.
