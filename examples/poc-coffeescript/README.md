# CoffeeScript proof of concept

Compiles a Vue component whose `<script setup>` and template expressions are written in CoffeeScript,
through the compiler levels. Not a supported input; CI installs no CoffeeScript toolchain.

```bash
cargo run -p poc-coffeescript -- examples/poc-coffeescript/input/Counter.vue
cargo test -p poc-coffeescript
```

- Input: `input/Counter.vue` with `<script setup lang="coffee">`. Output: `__snapshots__/Counter.js`,
  a Vue render-function component.
- Levels: CoffeeScript is on the Lang axis and the framework stays Vue. `src/lang.rs` implements the L2
  `ExprDialect` capability for the language and translates the script; the L1 Vue container and
  markup parser read the file; the L1 to L2 foreign-expression hook admits each template
  expression as `coffee`; the shared L2 to L3 lowering runs; the shared Vue target in
  `poc-support` asks the same capability to emit JavaScript.
- Covered: identifiers, `@name`, implicit calls, `and` / `or` / `not` / `is` / `isnt`, `yes` / `no`,
  `#{}` interpolation, `if a then b else c`; script `import`, `defineProps [...]`, assignments and
  `->` functions with an indented body.
- Refused with a span: any other statement or expression syntax.
