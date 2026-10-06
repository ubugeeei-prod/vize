## Area

Linter, `vue/require-component-registration` (diagnostic span)

## Version

`vize` 0.432.0

## Minimal reproduction

`my-panel.vue`

```vue
<template>
  <div>
    <MyButton>OK</MyButton>
  </div>
</template>
```

`vize.config.json`

```json
{ "linter": { "preset": "incremental", "rules": { "vue/require-component-registration": "warn" } } }
```

```sh
vize lint -f json my-panel.vue | jq -c '.[0].messages[] | {line,column,endLine,endColumn}'
vize lint -f ansi --help-level none my-panel.vue
```

## Actual

```
{"line":3,"column":5,"endLine":3,"endColumn":13}
```

```
  ⚠ [vize:vue/require-component-registration] Component is used but not explicitly imported
   ╭─[my-panel.vue:3:5]
 2 │   <div>
 3 │     <MyButton>OK</MyButton>
   ·     ────────
 4 │   </div>
```

Columns 5–13 are `<MyButto`: the span starts at `<` but its length is the tag name's length, so
the last character of the name is cut off. In an editor the squiggle covers `<MyButto`.

## Expected

The span covers the tag name, `MyButton` (columns 6–14), or the whole start tag.

## Cause (from reading the source)

`collect_components()` in `crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs`
pushes `(start, start + tag.len())` with `start = element.loc.span.start`, which is the offset of
`<`. Adding 1 to `start` (or using the tag-name span) fixes it.
