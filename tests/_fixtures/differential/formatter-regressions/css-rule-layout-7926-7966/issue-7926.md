### Area

Formatter (`vize fmt`, glyph), SFC `<style>`

### Version

`vize` 0.432.0

### Minimal reproduction

`App.vue`

```vue
<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}

.first-state.first-modifier,
.second-state.second-modifier,
.third-state.third-modifier,
.fourth-state.fourth-modifier,
.fifth-state.fifth-modifier {
  color: red;
}
</style>
```

`Insert.vue`

```vue
<style scoped>
.a {
  color: red;
}
.b {
  color: blue;
}
</style>
```

```sh
vize fmt --write --no-config App.vue Insert.vue
```

### Actual

1. Selector lists are joined onto one line, with no width limit:

```css
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s;
}

.first-state.first-modifier, .second-state.second-modifier, .third-state.third-modifier, .fourth-state.fourth-modifier, .fifth-state.fifth-modifier {
  color: red;
}
```

2. A blank line is inserted between adjacent top-level rules in `Insert.vue` (`}\n.b {` → `}\n\n.b {`).

### Expected

Both files unchanged, as with Prettier / Oxfmt: one selector per line in a selector list, and the authored spacing between rules preserved (Prettier keeps up to one blank line but never adds one).

The `.fade-enter-active, .fade-leave-active` transition pair is in almost every Vue codebase, so a Prettier-formatted project gets a diff in most files that have a `<style>` block.

Related: #7826 (blank lines removed inside rule blocks) and #7866 (normalization stops for the whole block); this is the opposite direction (added between rules) and also happens in blocks without any of the #7866 triggers.

### Environment

- OS: macOS 26.4.1
- Architecture: arm64
- Node.js: 26.8.1
- Package manager: npm 11.19.0
