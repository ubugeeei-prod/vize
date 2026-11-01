---
title: Attribute Options
---

# Attribute Options

## `vue/attribute-hyphenation`

Use this option for static prop names on custom components. `always` reports camelCase-authored
attributes; `never` reports authored attributes that are hyphenated. Native attributes, `aria-*`, `data-*`,
SVG mixed-case attributes, dynamic arguments, and directive shorthand that is parsed as an attribute
are skipped.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      rules: {
        "vue/attribute-hyphenation": "warn",
      },
      ruleOptions: {
        "vue/attribute-hyphenation": "never",
      },
    },
  },
});
```

Bad with the config above:

```vue
<template>
  <UserCard user-name="Ada" />
</template>
```

Good:

```vue
<template>
  <UserCard userName="Ada" aria-label="Ada Lovelace" />
</template>
```

### Literal attribute exemptions

The original style string remains valid. Use `{"style":"always","ignore":["libraryProp"]}` to add exemptions. Entries are case-sensitive substrings, with literal matching. The existing `data-`, `aria-`, `slot-scope`, SVG and other built-in exemptions remain in force.

Omitting `style` selects `always`. An empty list or object resets the custom exemptions. Blank entries, names ending in `:exit`, and duplicates are rejected. Select the rule and its severity separately in `linter.rules`.

Oxlint also accepts `["warn", "always", {"ignore":["libraryProp"]}]`. The native API accepts `attributeHyphenationIgnore` or the canonical object in `ruleOptionsJson`.

### Literal tag exemptions

Use `{"style":"always","ignoreTags":["LibraryWidget"]}` to exempt attributes on that exact authored tag. Matching is case-sensitive and treats dots and colons literally. Children and neighboring elements are checked independently, even when their parent matches.

An empty list or object resets the custom tag exemptions. Empty strings, whitespace and `:exit` are valid literal entries; they do not match an ordinary tag. Duplicate and non-string entries are rejected. Regular-expression notation such as `/^Library/` is explicitly unsupported and rejected.

Oxlint accepts `["warn", "always", {"ignoreTags":["LibraryWidget"]}]`. The native API accepts `attributeHyphenationIgnoreTags` or the canonical object in `ruleOptionsJson`. Select the rule and its severity separately.
