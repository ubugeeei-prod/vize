# Svelte proof of concept

Compiles a Svelte 5 component through the compiler levels. Not a supported input.

```bash
cargo run -p poc-svelte -- examples/poc-svelte/input/Counter.svelte
cargo test -p poc-svelte
```

- Input: `input/Counter.svelte`. Output: `__snapshots__/Counter.js`, a `svelte/internal/client`
  module.
- Levels: `src/container.rs` is an L1 container format for `.svelte` files; the L1 markup parser
  reads the template; `src/dialect.rs` lowers Svelte's tags and directives to L2 ops; the shared
  L2 to L3 lowering decides what is reactive; `src/target.rs` writes the module from the L3
  program; `src/runes.rs` turns runes into signal accesses.
- Covered: `{expr}`, `{@html}`, `{#if}` / `{:else if}` / `{:else}`, `{#each list as item}`,
  `onclick={}` / `on:click={}`, `bind:value={}`, `{expr}` attributes, and `$state`, `$derived`
  and `$props` in the instance script.
- Refused with a span: `{#await}`, keyed or indexed `{#each}`, components, `class:` / `use:`
  directives, attribute text mixed with tags, shadowed runes.
