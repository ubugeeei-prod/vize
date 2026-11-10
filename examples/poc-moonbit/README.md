# MoonBit proof of concept

Compiles a Vue component whose `<script setup>` and template expressions are written in MoonBit,
through the compiler levels. Not a supported input; CI installs no MoonBit toolchain.

```bash
cargo run -p poc-moonbit -- examples/poc-moonbit/input/Counter.vue
cargo test -p poc-moonbit
```

- Input: `input/Counter.vue` with `<script setup lang="moonbit">`. Output: `__snapshots__/Counter.js`,
  a Vue render-function component.
- Levels: MoonBit is on the Lang axis and the framework stays Vue. `src/lang.rs` implements the L2
  `ExprDialect` capability for the language and translates the script; the L1 Vue container and
  markup parser read the file; the L1 to L2 foreign-expression hook admits each template
  expression as `moonbit`; the shared L2 to L3 lowering runs; the shared Vue target in
  `poc-support` asks the same capability to emit JavaScript.
- Covered: `@vue.name` members, `not(x)`, `|>`, `if c { a } else { b }`, `\{}` interpolation, a
  `Ref`'s `.val`, `to_upper` / `to_lower`; script `@vue.define_props`, `let` and `fn` with an
  optional result type.
- Refused with a span: any other statement or expression syntax. No `.mbt` file and no MoonBit
  dialect enter the repository.
