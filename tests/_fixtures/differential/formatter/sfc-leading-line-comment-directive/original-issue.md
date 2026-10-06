### Area

Formatter (`vize fmt`, glyph), multi-line directive values

### Version

`vize` 0.432.0

### Minimal reproduction

`Min.vue`

```vue
<template>
  <input
    :placeholder="// note
    x"
  />
</template>
```

```sh
vize fmt --write --no-config Min.vue   # run it a few times
```

### Actual

The continuation line moves 2 columns right on every run, so `vize fmt --check` never passes:

```
run 1:       x"     (6 spaces)
run 2:         x"   (8 spaces)
run 3:           x" (10 spaces)
```

`@click="// note\n    run()"` drifts the same way. The shape with the comment on its own line (`="\n  // note\n  x\n"`) is stable.

This is exactly the output older Vize versions produced for #6694 (see the "Actual" there), so files formatted with vize ≤ 0.426 now drift forever.

### Expected

Idempotent: after one pass the value is stable, e.g. continuation lines anchored at attribute indent + 2 (or normalized to the Prettier shape from #6694).

### Likely cause

`crates/vize_glyph/src/template/directives.rs`: the early return added for #6694,

```rust
if value.contains('\n') && trimmed.starts_with("//") {
    return (value.to_compact_string(), false);
}
```

returns the authored value with `indent_multiline_value = false`, which skips the continuation re-anchoring added for #3346. The SFC-level indent is then applied on top of the already-indented continuation line on every pass.

### Environment

- OS: macOS 26.4.1
- Architecture: arm64
- Node.js: 26.8.1
- Package manager: npm 11.19.0
