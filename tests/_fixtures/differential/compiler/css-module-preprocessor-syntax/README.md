# Preprocessor `<style module>` syntax

`Card.vue.txt` has the shape of n8n's `<style lang="scss" module>` blocks
(`@use`, `$` variables, `@include`). The Real Project Matrix compile run of
2026-10-14 logged one "CSS parse error" per such block: 175 in n8n, 58 in
misskey and 86 `lang="less"` blocks in lx-music-desktop.

`@vue/compiler-sfc` leaves preprocessor blocks to the bundler, and Vize's Vite
plugin imports `?vue&type=style&module` so Vite's own pipeline builds `$style`.
The compiler therefore treats a plain-CSS parse failure of preprocessor source
as no diagnostic and passes the block through. A `lang="css"` module that fails
to parse still warns.
