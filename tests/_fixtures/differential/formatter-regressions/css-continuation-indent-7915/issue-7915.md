### Area

Formatter (glyph): `<style>` blocks in SFCs

### Version

`vize` 0.432.0

### Summary

For a multi-line CSS declaration, the continuation lines are re-indented on every `vize fmt --write` when the indentation unit is not 2 spaces. Examples are a `transition` list with one item per line, where the first value stays on the property line.

- `--tab-width 4`: the continuation indent doubles on every pass (8 → 16 → 32 spaces).
- `--use-tabs`: the continuation indent loses one tab on every pass, down to column 0.

The formatter is therefore not idempotent, and `vize fmt --check` fails right after `vize fmt --write`. With the default `tabWidth: 2` the output is stable.

### Minimal reproduction

`Button.vue`

```vue
<template>
  <button class="button" type="button">Save</button>
</template>

<style scoped>
.button {
  transition: background-color var(--duration, 0.2s) var(--ease, ease),
    color var(--duration, 0.2s) var(--ease, ease);
}
</style>
```

```sh
npx vize fmt --no-config --tab-width 4 --write Button.vue   # run 3 times
npx vize fmt --no-config --tab-width 4 --check Button.vue   # fails right after --write
```

### Actual

`--tab-width 4`, the declaration after each pass:

```css
/* pass 1 */
    transition: background-color var(--duration, 0.2s) var(--ease, ease),
        color var(--duration, 0.2s) var(--ease, ease);
/* pass 2 */
    transition: background-color var(--duration, 0.2s) var(--ease, ease),
                color var(--duration, 0.2s) var(--ease, ease);
/* pass 3 */
    transition: background-color var(--duration, 0.2s) var(--ease, ease),
                                color var(--duration, 0.2s) var(--ease, ease);
```

`--use-tabs` (`→` is a tab):

```css
/* pass 1 */
→transition: background-color var(--duration, 0.2s) var(--ease, ease),
→→color var(--duration, 0.2s) var(--ease, ease);
/* pass 2 */
→transition: background-color var(--duration, 0.2s) var(--ease, ease),
→color var(--duration, 0.2s) var(--ease, ease);
/* pass 3 */
→transition: background-color var(--duration, 0.2s) var(--ease, ease),
color var(--duration, 0.2s) var(--ease, ease);
```

`--check` right after `--write`:

```
Would reformat: Button.vue
  1 file(s) would be reformatted
```

(exit code 1)

A value without functions (`transition: color 0.2s ease,` + `    background-color 0.2s ease;`) is stable. Its continuation line is kept verbatim at 4 spaces, though. With `--tab-width 4` that is the same column as the property. With `--use-tabs` it stays as 4 spaces under a tab-indented property, which mixes tabs and spaces.

### Expected

`vize fmt` output is a fixed point: a second `--write` changes nothing, and `--check` passes after `--write`. Continuation lines get a stable indent (one indent unit deeper than the property, or the Prettier layout with the property on its own line), expressed in the configured unit (tabs with `useTabs`).

### Environment

- OS: macOS 26.4.1 (arm64)
- Node.js: 26.8.1
