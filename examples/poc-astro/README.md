# Astro proof of concept

Compiles an Astro component through the compiler levels. Not a supported input.

```bash
cargo run -p poc-astro -- examples/poc-astro/input/Card.astro
cargo test -p poc-astro
```

- Input: `input/Card.astro`. Output: `__snapshots__/Card.js`, an Astro server-rendering module.
- Levels: `src/container.rs` is an L1 container format for the `---` frontmatter; the L1 markup
  parser reads the markup; `src/dialect.rs` lowers it to L2 ops; the shared L2 to L3 lowering
  separates static from dynamic; `src/target.rs` writes one `$$render` template from the L3
  program.
- Covered: `{expr}`, `attr={expr}`, `{cond && <el/>}` (as `ui.if`) and
  `{list.map((item) => <el/>)}` (as `ui.for`); frontmatter imports move to the module top.
- Refused with a span: other markup inside expressions, components, directives such as
  `class:list`, attribute text mixed with expressions.
