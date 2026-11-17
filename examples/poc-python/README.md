# Python proof of concept

Compiles a Vue component whose `<script setup>` and template expressions are written in Python,
through the compiler levels. RustPython parses the real Python grammar and provides byte-accurate
AST ranges; this example then lowers a deliberately bounded semantic subset. It is not a supported
input and CI installs no Python toolchain.

```bash
cargo run -p poc-python -- examples/poc-python/input/Counter.vue
cargo test -p poc-python
```

- Input: `input/Counter.vue` with `<script setup lang="python">`. Output: `__snapshots__/Counter.js`,
  a Vue render-function component.
- Parser: [`rustpython-parser`](https://crates.io/crates/rustpython-parser) recognizes both the
  module and every template expression. There is no example-specific tokenizer or parser.
- Levels: Python is on the Lang axis and the framework stays Vue. `src/lang.rs` implements the L2
  `ExprDialect` capability for the language and lowers the parsed AST; the L1 Vue container and
  markup parser read the file; the L1 to L2 foreign-expression hook admits each template
  expression as `python`; the shared L2 to L3 lowering runs; the shared Vue target in
  `poc-support` asks the same capability to emit JavaScript.
- Covered: precedence-aware arithmetic, comparisons, `and` / `or` / `not`, `True` / `False` /
  `None`, `a if c else b`, lists, tuples, subscripts, f-strings, `len(x)`, `upper()` / `lower()`;
  script `from vue import ...`, `define_props([...])`, assignments and `def` with an indented body.
- Refused with an AST span: syntactically valid Python outside the lowering surface. Invalid Python
  reports RustPython's parser offset. No `.py` file enters the repository.
