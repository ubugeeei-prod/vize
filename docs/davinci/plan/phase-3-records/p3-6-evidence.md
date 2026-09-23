# P3-6 - Evidence, witnesses and measurements

Companion to the [P3-6 record](./p3-6.md): the evidence behind each slice,
the commands that re-derive it, and the local measurements taken so far.

## Evidence and remaining limits

- Graph payload mutation changes attributes, text references, and event handlers
  in emitted code even with a decoy source. Scratch source/arena lifetime tests
  prevent accidental reuse of an earlier AST. Replacing an ordering edge with a
  duplicate is rejected; permuting the serialized edge list remains valid.
- Production probe tests pin zero legacy core/Vapor walks and zero expression
  reparses for admitted native shapes. Explicit legacy selection retains the
  P2-12a ladder's two walks and exact visit counts; the production reparse floor
  remains unchanged. These are structural measurements, not a latency claim.
- The text/event expansion covers mixed and adjacent interpolations, sibling
  identity through Unicode/null/number updates, focus/custom/non-bubbling events,
  keyboard versus mouse modifiers, composed key/DOM guards and combined listener
  options. Mounted tests caught two shared
  generator defects: key filters were dropped when DOM modifiers were present,
  and multiple listener options lacked separating commas. Both are now covered
  by independent expected delivery traces as well as VDOM/Vapor agreement.
  Mouse-only modifiers normalize click to contextmenu/mouseup, keyboard aliases
  and arbitrary key names stay in key filters, and non-bubbling events attach
  directly rather than relying on document-level delegation.
- The Rust-owned static/dynamic scenario still matches the independent expected
  mounted DOM/Vapor traces. Nested/separated native nodes additionally preserve
  measured DOM identities across object replacement, prop/text updates, disabled
  clicks, and unmount, with exactly two delivered events.
- Testing the fallback exposed nontermination on rejected JS such as `(`. The
  recursive text-scanning rewrite was removed: failed parses terminate with
  opaque passthrough in the existing tolerant mode; prefixing-enabled expression
  validation still reports errors and emits no code.
- Control-flow evidence: an exact-reason refusal table, payload mutations that
  change conditions, loop sources and keys in emitted code, and five corrupt
  graphs that pass the generic verifier but are rejected. Branch toggles,
  keyed reorder/insert/delete, unkeyed in-place patching, nested loops inside
  branches, object/range loops and root-level chains/loops all have
  independently specified mounted DOM, event and identity traces for VDOM and
  Vapor; the nested scenario also runs over the retained lane. Every mounted
  and benchmark source is pinned to zero legacy walks and reparses.
- Four reviewed output snapshots reflect parent-before-child node numbering and
  equivalent child-index navigation. The adjacent dynamic-child fixture also has
  an independently specified mounted update trace.
- Ordinary phrasing elements (`abbr`, `code`, `data`, `kbd`, `mark`, `samp`,
  `sub`, `sup`, `time`, `var`) have the same template bytes as the retained lane
  with both prefix settings. A mounted trace checks live prop and text updates
  through the published DOM, native Vapor, and retained Vapor runtimes; repaired
  paragraph nesting still selects the legacy lane.
- Empty textarea models emit the same code as the retained lane with either
  prefix setting. A mounted trace observes the live textarea value, its model
  update on input, and a later external patch on DOM, native Vapor, and retained
  Vapor. Child content and conflicting value sources still select legacy;
  mutating S3's `element-kind` away from its input/textarea owner is rejected.
- `v-cloak` uses the S3 directive op's static partition at the root and the
  inherited effect scope inside branches. Unit tests pin one removal operation,
  retained template equality and rejection of a stale directive kind. The
  independent mounted trace checks that the cloak attribute is absent after
  each mount while the live node and untouched sibling keep their identities.
- Static `style` beside `:style` now projects to one `_setStyle` call in
  authored source order. Both orders match independently specified mounted DOM
  states and node identities through updates and unmount, and the pinned
  official `@vue/compiler-vapor` output under the same runtime. Empty static
  style attributes still select legacy.
- The first TS-33 fixture compiles one conditional element and its static
  sibling through Vize S3 and pinned official `@vue/compiler-vapor`. Both
  outputs mount on the same pinned `@vue/runtime-vapor`; five exact snapshots
  include DOM state, branch recreation identity and unchanged sibling identity.
  A zero-legacy-walk assertion ensures the Vize side actually exercises S3.
- References beginning with `$event` remain in the legacy lane. Their handler
  semantics follow Vue: a reference names a component/setup binding; only an
  inline statement receives the implicit event parameter. The Chromium contract
  in #6261 corrects the earlier no-op callback assumption and checks both VDOM
  and Vapor against official Vue 3.5/3.6 and independent expected traces. The
  expression lanes share retained-AST declaration ownership; five pinned
  upstream scope differences also execute authored JavaScript as an independent
  oracle, with the original official output and observations retained.

## Mechanical witnesses

- `cargo test -p vize_atelier_vapor --lib`
- `cargo test -p vize_atelier_vapor --test davinci_vapor_artifact --test davinci_walk_baseline --test davinci_expr_reparse_floor`
- `cargo test -p vize_atelier_vapor --test davinci_mounted_behavior --test davinci_s3_compiled_trace`
- `cargo test -p vize_atelier_vapor --test davinci_vapor_upstream_parity`
- `cargo test -p vize_atelier_vapor --test davinci_event_handlers -- --ignored --nocapture` (Chromium)
- `cargo clippy -p vize_atelier_vapor --lib -- -D warnings`
- `cargo run -p vize_test_runner --bin coverage` (published fixture parity)
- `cargo bench -p vize_atelier_vapor --bench davinci -- vapor_native_pair` compares
  the native/default and explicitly retained legacy routes on identical text,
  event, and control-flow fixtures. The same benchmark can be applied to a pinned base checkout to
  measure the former S3-attempt/fallback cost independently of the direct legacy
  floor. These microbenchmarks do not replace the full corpus promotion budget.
- The full Check and Davinci Lean Actions lanes; the umbrella's remaining
  backend, corpus, performance, and release gates remain independent.

Local smoke measurement on 2026-09-21 (Nix toolchain, ten Criterion samples,
one-second warmup, two-second measurement) put the admitted text fixture at
9.56–9.68 μs and the event fixture at 10.30–10.55 μs. Explicit legacy selection
on the same build remained faster at 7.45–7.58 μs and 8.02–8.08 μs respectively.
The default route at base `a8b17f73b` selected fallback on these fixtures and
measured 11.24–11.75 μs and 11.68–11.93 μs. These sequential local observations
are provisional, not isolated repeated A/B evidence or a promotion pass. The
base/head builds reused a target directory, so the changed crate was cleaned
and rebuilt before measuring the final head to avoid stale artifact reuse;
the Actions A/B lane uses isolated build graphs.

The control-flow slice's same-session smoke run (ten samples, one-second
warmup, two-second measurement, other agents sharing the host) measured the
`control_flow` fixture at 23.38–24.14 μs native versus 16.74–18.21 μs on the
retained lane; `text_runs` 10.45–10.92 μs versus 7.90–8.09 μs and `events`
11.64–12.21 μs versus 8.66–9.12 μs. The native route remains slower at this
fixture size: S1/S2/S3 construction plus admission costs more than the retained
parse/transform/lower on small templates. No promotion claim follows from it.

The expression slice's run (same settings, host heavily shared) measured the
new `expressions` fixture at 17.03–19.51 μs native versus 14.32–14.48 μs
retained, and `control_flow` at 27.49–32.34 μs versus 17.75–20.21 μs; the
spread reflects host contention, and the native gap remains.

After the single parse and admission tables (2026-09-22), Criterion's spread on
the shared host was too wide to compare lanes, so the lanes were interleaved:
300 rounds of 200 compiles per lane, alternating, per `vapor_native_pair`
fixture (prefixing on). Native/retained per-compile ratios, minimum and median:
`text_runs` 1.066/1.066 (8.31 vs 7.80 μs min), `events` 1.079/1.084,
`expressions` 0.865/0.864 (12.56 vs 14.53 μs), `components` 0.967/0.968, and
`control_flow` 1.090/1.088 (18.04 vs 16.56 μs). Before this work the admitted
text, event and control-flow shapes ran 19–35% behind the retained lane. A
sampled control-flow profile attributes the rest to S1→S2 lowering (about a
fifth of the compile), the generic S3 verifier's linear lookups and S3 lowering;
those crates are shared and unchanged here. Allocation counts of the
`atelier_vapor_compile_*` ladder (default options) fall against the current
base on every fixture (small 134→116, medium 739→642, large 1507→1507,
stress-deep 1058→879, stress-interp 3530→2729, stress-wide 435→113); the
committed budgets predate both and are not gated in CI.

The object binding slice (2026-09-22, same interleaving): `spreads`
1.146/1.146 (15.44 vs 13.47 μs min). The lanes emit different programs there:
the native lane builds upstream's merged sources, the retained lane separate
setters and no component `v-on` object. On the same run `templates` measured
1.046/1.048 and `control_flow` 1.110/1.114.

Dense ids in the generic verifier now resolve by index; a table that is not
dense still scans, so every verdict is unchanged. `rebuild_source` reserves
the exact length of a rebuilt compound text. Native admission and emission
keep the payload and their tables in the output arena, and
`davinci_vapor_native_budget` pins native allocation calls for the seven
`vapor_native_pair` fixtures at 79, 84, 112, 121, 156, 114 and 174. The
linear verifier lookups in the single-parse profile above are that earlier
measurement, not the current dense path.
