# Angular proof of concept

Compiles an Angular component with an inline template through the compiler levels. Not a
supported input.

```bash
cargo run -p angular-poc -- examples/angular-poc/input/counter.component.ts
cargo test -p angular-poc
```

- Input: `input/counter.component.ts`. Output: `__snapshots__/counter.component.js`, an Ivy
  component definition.
- Levels: the L1 markup parser reads the `@Component` template; `src/dialect.rs` lowers it to L2
  ops; the shared L2 to L3 lowering separates static from dynamic; `src/target.rs` writes Ivy
  instructions, static ops in the creation pass and dynamic ops in the update pass.
- Covered: `{{ expr }}`, `[prop]`, `(event)`, `@if` / `@else if` / `@else`,
  `@for (item of list; track item)`.
- Refused with a span: structural directives (`*ngIf`), template references, two-way bindings,
  components, listeners inside embedded views, nested embedded views.
