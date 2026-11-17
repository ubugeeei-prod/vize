## Why

`vue.directive` contracts (#13074) are honored by lint, the language server and the compiler: `vize build` in #13218, and the Vite host in #13246. The type checker was the last consumer, so the load-time guard still refused every directive plan. This is the type-checker layer, and with it directive plans load.

## What

- **Admission.** `vize check` admits the plan as `typecheck` and lowers the directive table once (`check/runner/plugins.rs`). The registry lists `typecheck` in `vue.directive`'s `admittedBy`, so every consumer now implements the capability and `refuse_unimplemented` lets directive plans load in every product.
- **Hook types match runtime.** `VirtualTsOptions` carries the table, and batch projections keep it. A declared directive's hook call receives the same default modifiers the compiler emits, in both the VDOM binding object and the Vapor positional argument. The defaults are unmapped, because nothing authored them, so no diagnostic can land on a range that does not exist.
- **Diagnostics.** A binding that breaks its contract is a `vize check` error with the compiler's message, on the binding's authored start. The walk reads the parsed template, so it respects `v-pre`.
- **Docs.** The incubation status, the SDK README and `v2.d.ts` describe the loaded capability and what is not covered yet:
  - runtime registration, and value or argument types (the contract carries only modifiers);
  - JSX/TSX directives;
  - the editor's virtual TypeScript defaults (the editor reports violations through lint);
  - the TypeScript/Volar plugins, which read no plan.

End to end, with `@acme/tracking` declaring `once`, `passive` (default `true`) and `strict` (required):

```
vize build src/Ok.vue           [_directive_track, e, void 0, { strict: true, passive: true }]
vize build src/Ok.vue --vapor   _withVaporDirectives(n0, [[_directive_track, () => (e), undefined, { "strict": true, "passive": true }]])
vize build bad/Bad.vue          ✗ CompilerError { code: DirectiveContract, message: "v-track does not declare the modifier `bogus` (declared by @acme/tracking)", ... }
vize lint lintsrc               [vize:plugin/directive-contract] v-track does not declare the modifier `bogus` (declared by @acme/tracking)
vize check lintsrc              error:2:6 v-track does not declare the modifier `bogus` (declared by @acme/tracking)
                                error:3:6 v-track requires the modifier `strict` (declared by @acme/tracking)
```

## Tests

- **canon:**
  - declared hooks receive the defaults after the authored modifiers (`{ "once": true, "passive": true, }` and `{ "passive": true, }`), and no mapping starts at a default;
  - the contract walk reports exact spans and messages, skips `v-pre`, and adds nothing with an empty table;
  - the `directive` filter of the canon lib passes (106 tests).
- **carton:** a directive plan is admitted by every product and passes `refuse_unimplemented`; the runtime digest is re-pinned.
- **JS:** the projection test now expects the directive plan to project, and the Nuxt lint generation loads it (`pluginRules: true`). Ran `plugin-config-projection`, `plugin-plan` and the Nuxt plugin-rule tests.
- **Ran:** `cargo check --workspace --all-targets` and CI clippy for `vize_canon`, `vize`, `vize_carton` and `vize_maestro`.
- **Pre-existing failure:** `canon batch::executor::tests::nested_generated_config` fails locally on macOS because of the `/tmp` → `/private/tmp` symlink; it is unrelated.

## Performance

MEASUREMENTS

🤖 Generated with [Claude Code](https://claude.com/claude-code)

---

<!-- Handoff notes (remove this section and the .handoff/ file before opening the PR) -->

## Handoff notes (#13074 directive track)

- Stacked on `plugins-13074-directive-vite` (#13246). After #13246 merges, run `git rebase --onto origin/main 0cc69b8598`, drop this `.handoff/` file, push, and open the PR with the body above.
- MEASUREMENTS: dispatch `check-bench.yml` once (wall time from interleaved base/head CI runs only). Then fill the table: wall p50/p95, CPU, RSS, instructions, output identity, IPC 0, callbacks 0.
- Before queueing, dispatch `level-instruction-counts.yml` once.
- After a registry change, re-pin the runtime digest. It is currently `ca66b887…` in `crates/vize_carton/src/config/loader/js/runtime_source/tests.rs` and `tests/tooling/support/native-config-script.ts`.
- `canon batch::executor::tests::nested_generated_config` fails locally on macOS only (`/tmp` vs `/private/tmp`). Pre-existing.
