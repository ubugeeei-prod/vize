# Solid proof of concept

Compiles a Solid component through the compiler levels. Not a supported input.

```bash
cargo run -p solid-poc -- examples/solid-poc/input/Counter.jsx
cargo test -p solid-poc
```

- Input: `input/Counter.jsx`. Output: `__snapshots__/Counter.js`, a `solid-js/web` DOM module.
- Levels: the parsed module is the surface (Solid's markup lives in script); `src/dialect.rs`
  lowers each outermost JSX element to L2 ops; the shared L2 to L3 lowering decides what is
  reactive; `src/target.rs` writes the module from the L3 program.
- Covered: elements, components with props and children, static and expression attributes,
  `on*` events (delegated), text and expression children.
- Refused with a span: fragments, spread attributes and children, namespaced attributes,
  member-expression tags, character references.
