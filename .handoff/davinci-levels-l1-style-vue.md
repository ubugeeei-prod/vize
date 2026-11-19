# Handoff: davinci-levels-l1-style-vue (input-neutral levels, step 3: `css/` → `style/`)

## State

- Step 4 (Vue owner, `moved.rs`, `htmlish` allow-list) was split out to
  `davinci-levels-l1-vue` (its own PR); this branch still holds both steps.
- Step 3 here: `level-layout-move.ts l1-style` (scripted) plus the hand split
  into `style.rs` + `style/{css/, scss, less, sass, stylus, nesting,
  structure, classes}` and `framework/vue/style_bind.rs`. It compiles and the
  L1/L1→L2/L2/armature/glyph/croquis/patina/maestro suites pass (the
  `vize_atelier_sfc` runtime-harness failures are `Cannot find module 'vue'`
  under `npm/ui` on a Mac checkout, environment only).
- Output: behavior is unchanged (pure moves; the split moves items verbatim).

## Why it is not open

Instruction rows (ci-opt, thin LTO, 16 CGUs), base `dc783944a3` run
44474854095 vs head `4fcce957d0` run 44474858403, deterministic (twins
44476093773 / 44476097614, A/A 0 except the known-noisy
`sfc_parse_template_only`):

| Row | Δ |
| --- | ---: |
| `formatter_sfc_reuse`, `formatter_sfc_simple` | +1 |
| `musea_parse_art_inline` | +1 |
| `sfc_parse_template_only_retained` | +1 |
| `atelier_vapor_compile_large` | −2 |

- Bisect: the style step alone (947dbf522b, run 44476137532) gives exactly
  these deltas; the Vue step adds nothing.
- Callgrind (runs 44477516474, 44477521759): the only function that moves is
  `vize_l1::container::vue::boundary::template_boundary::find_template_block_end`
  (+1 self instruction per call); objdump (run 44478423075) shows only register
  spills moving at its entry. Its source is unchanged: moving the style modules
  changes the CGU partition of `vize_l1`, and with it this function's register
  allocation. `prepare_decoded_value` improves by 2 the same way.

Attempts (all on `bench/levels-l1-boundary*`):

| Variant | Result vs base |
| --- | --- |
| V1: flat path stays in `find_template_block_end`, nested scan in an `#[inline(never)]` fn (3eb4a76963) | formatter −80, `musea_parse_art_inline` −47, `sfc_parse_template_only_retained` −69; but `musea_parse_art_define_art` +6 and `musea_parse_art_legacy_attrs` +1, both in `hashbrown::RawTable<(Cow<str>, Cow<str>)>::reserve_rehash` (the block attribute map), whose codegen moved with the CGU change |
| V2: V1 + `#[inline]` entry (56b9164a93) | `musea_parse_art_inline` +11: `find_flat_template_end` stops being inlined |
| V4: V1 + attribute table pre-sized to 7 on first insert (63c4b95fff) | 16 rows move, croquis and musea up to +0.1% |

The same move under the release codegen (fat LTO, one CGU; runs 44486453993 /
44486459632) moves 25 rows by −0.02%…+0.07%: whole-program inlining shifts
with module layout too.

## Decision needed

Under "every instruction row ≤ base", a module move that changes `vize_l1`'s
CGU partition cannot be shown neutral row by row; the perturbation lands on an
unchanged function. Options:

1. Accept V1-style fixes that make the specific function robust, and keep
   iterating per perturbed function (each iteration is ~1 h of CI, and each fix
   perturbs other generics such as `reserve_rehash`).
2. Measure move-only PRs with a codegen that does not depend on module layout
   for the gate (e.g. a per-function-ordered or `-Z` stable partition), or
   accept ±1-instruction CGU-layout shifts on move-only PRs whose byte output
   and release-binary size are identical.
3. Keep `css/` where it is and change only the gate's allow-list (the
   maintainer called the layout "not beautiful"; this defers the beauty).

## Next steps once decided

- Rebase on main after `davinci-levels-l1-vue` merges: re-run
  `level-layout-move.ts l1-style` (it reports 0 changes on this head), then
  cherry-pick the hand split commit (947dbf522b) and the boundary fix if
  chosen.
- Regenerate `docs/davinci/plan/input-placement-owed.json` and the storage
  inventory rows for `style/{nesting,stylus,tests}.rs`.

## Track status (levels-inputs, 2026-11-19)

Landed / queued:

- #13393 merged: `Op::JsxChildValue` → `Op::ChildValue` and friends (owed 147 → 117).
- #13403 (auto-merge): step 4, one Vue owner, `moved.rs` deleted, `htmlish` allow-list (−4).
- #13405 (auto-merge): L3/L4/L0 concept names (−33).
- `davinci-levels-l2-concepts` (PR pending measurements): every L2 `native` name (−31).

#13403, #13405 and the L2 branch each carry the same move-script additions;
whichever merges later needs a rebase (keep the union of the steps, regenerate
`docs/davinci/plan/input-placement-owed.json`).

Remaining owed after those (≈49):

1. **L1→L2 `native` names** (17 `ident:` + 7 `status:`): same scripted pattern
   as `l2-concepts`. Draft map: `native` → the component/template lowering it
   is (`NativeComponent`/`NativeChild`/`NativeEmbed`/`NativeLowered` collide
   with L1 and Vue names; pick `Lowered*`/`File*`), `native_file` → the SFC
   file lowering, `native_vue2` → `vue2`. Watch `include!` paths and the
   `davinci-l1-dialect-isolation` lists.
2. **L1 `NativeComponent`/`NativeTemplateComponent`** (2 `ident:`): their
   `Debug` text is pinned 84× in the #7502 reviewed outputs
   (`crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502/`);
   renaming needs a reviewed successor of that evidence (maintainer).
3. **`css/` → `style/`** (1): this file, "Decision needed" above.
4. **Structural steps 5–9** (L2→L3 `lib.rs`, L3 `jsx*`, L4 `jsx*`, `lang/tsrx*`,
   `targets/ts*`, `lib.rs`, `framework/vue.rs`): `davinci-levels-tsrx-lowering-2`
   (step 5) still needs #13340 and has the same CGU-layout sensitivity as the
   style move; steps 6–9 are design work (ModuleDef).
