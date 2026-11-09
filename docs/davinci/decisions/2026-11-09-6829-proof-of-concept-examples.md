# Proof-of-concept examples on the levels

Issue: #6829. Gates: `poc.solid` (#6859), `poc.svelte` (#11540), `poc.astro` (#11557),
`poc.angular` (#11558), `poc.coffeescript` (#11560), `poc.moonbit` (#11559) and
`poc.python` (#11561).

The [2026-11-03 scope decision](../completion-gates.md#maintainer-decisions-2026-11-03) made
these seven inputs proofs of concept: examples under `examples/`, not a supported surface, with
no parity commitment. This record fixes one shape for all seven so each gate is one small change
and none of them can leak framework code into the levels.

## Shape

Each proof of concept is a Rust crate `examples/poc-<name>/`, a workspace member with
`publish = false`:

- `input/` holds the authored component in the framework's or language's own syntax.
- `src/dialect.rs` is the example's dialect: it reads the input through a level surface
  (the L1 container and markup, or the parsed module when the markup lives in script) and
  lowers it to neutral L2 ops (`ui.element`, `ui.if`, `ui.on`, ...).
- The shared L2 to L3 lowering decides what is reactive and the L3 verifier checks it.
- `src/target.rs` is the example's L4 target. It walks the L3 program and writes the module
  with the L4 writer.
- `src/main.rs` runs the example: `cargo run -p poc-<name> -- examples/poc-<name>/input/<file>`.
- `README.md` names the input and the syntax the example covers and refuses.
- `__snapshots__/` holds the compiled output. A unit test compiles the input, parses the output
  as an ES module and compares it byte for byte with the snapshot (`VIZE_POC_UPDATE=1` rewrites it).

`examples/poc-support` holds what every example repeats: the L3 tree view, the
lowering-plus-verifier call, string escaping, the module check, the snapshot comparison and the
command-line runner. Its `vue` module is the Vue render-function target and SFC pipeline the
script-language examples share, because Vue stays their framework.

The script input languages (CoffeeScript, MoonBit, Python) keep Vue as the framework and change
the Lang axis. Their `src/lang.rs` takes the place of `src/dialect.rs`: it implements the L2
`ExprDialect` capability for the language and translates the `<script setup lang>` block. Template
expressions enter L2 as foreign expressions through the existing
`lower_source_block_with_foreign_expressions` hook, and the shared Vue target asks the same
capability to emit JavaScript. Each example translates its tiny language subset in Rust; CI
installs no toolchain for these languages.

## Isolation and CI

`tests/tooling/davinci-level-dialect-isolation.test.ts` checks that every `examples/` workspace
member depends only on level crates, oxc and `poc-support`, and that no level crate's code names
a proof-of-concept input. A missing neutral extension point lands in its level crate as its own
change with its own test, never as an example-specific branch.

The crates join the Compiler Rust test cohort, so the ordinary affected-crate and full Rust test
jobs run them. No new workflow or job is added. Adding an example touches only its own directory
and one line in each of the workspace members, the Rust test cohort and the clippy ratchet.
Snapshots sit in `__snapshots__/`, which the repository formatter and linter already skip,
because they are compiler output.
