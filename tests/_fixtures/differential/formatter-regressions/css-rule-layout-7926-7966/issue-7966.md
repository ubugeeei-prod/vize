### Area

Formatter (`vize fmt`): SFC `<style>` layout compared with Oxfmt / Prettier

### Version

`vize` 0.432.0 (compared with Oxfmt 0.64.0), Node v26.8.1, macOS arm64

### Summary

Two layout changes in `<style>` make `vize fmt` rewrite almost every SFC of a project that is formatted with Oxfmt (or Prettier), without any option to keep the existing layout:

1. A blank line is inserted between every pair of sibling rules. Oxfmt and Prettier keep the author's blank lines (collapsed to one) and do not add any.
2. A selector list written one selector per line is joined onto one line (`.b, .c {`). Oxfmt and Prettier print one selector per line.

In a 191-file Vue project, (1) alone added 864 blank lines in 120+ files; (2) changed every multi-line selector list. Neither changes the CSS, so this is about parity, not correctness, but it blocks switching `.vue` formatting from Oxfmt to Vize (the Vite+ `fmt.vize` switch) without a full-repo reformat.

Related: #7826 (blank lines inside a rule are removed), #7877 (blank line inserted after a root-level comment).

### Reproduction

`Rules.vue`

```vue
<template>
  <div class="a"></div>
</template>

<style scoped>
.a {
  color: red;
}
.b,
.c {
  margin: 0;
}
.d {
  padding: 0;
}
</style>
```

```sh
npx vize@0.432.0 fmt --no-config --write Rules.vue
```

### Actual (`vize fmt`)

```vue
<style scoped>
.a {
  color: red;
}

.b, .c {
  margin: 0;
}

.d {
  padding: 0;
}
</style>
```

### Oxfmt 0.64.0 (and Prettier 3)

The input is already formatted; output is unchanged:

```vue
<style scoped>
.a {
  color: red;
}
.b,
.c {
  margin: 0;
}
.d {
  padding: 0;
}
</style>
```

### Expected

Keep blank lines between rules as written (at most one), like Oxfmt / Prettier, and print each selector of a list on its own line. If the current layout is intended, an option to keep the Oxfmt / Prettier layout would be enough.
