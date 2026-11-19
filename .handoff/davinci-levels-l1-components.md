# Handoff: davinci-levels-l1-components (PR #13412) — paused 2026-11-19 ~21:10 JST

Owner track: levels.inputs (goal: `docs/davinci/plan/input-placement-owed.json` empty).
Delete this file before merge.

## State

- **Commits.**
  - `be718c0d4a`: L1 rename via step `l1-components`:
    - `NativeComponent` → `OwnedComponent`;
    - `NativeTemplateComponent` → `SelectedTemplateComponent`;
    - `vize_l1_to_l2`'s own `NativeComponent` is untouched;
    - 2 owed entries leave.
  - `993781cbdb`: the shared evidence-rename mechanism the coordinator asked for (decision (a)):
    - `docs/davinci/plan/evidence-renames.json`;
    - `davinci_test_support::evidence_renames` (Rust) and `tests/tooling/support/evidence-renames.ts` (TS);
    - `tests/tooling/evidence-renames.test.ts`;
    - #7502 gates switched to it (V4 fixture dropped; V1–V3 untouched).
- **Verified.**
  - TS gates pass: evidence-renames 3/3, 7502-reference 11/11, hook 6/6, static-class 3/3, input-placement 3/3, layout-move 7/7.
  - `cargo fmt --check` and the assertion lint pass.
- **Not built locally.** The disk was under 60 GB, so the Rust code (evidence_renames.rs, #7502 gates) is unverified. Read this PR's CI first: SFC compiler tests, Rust product report, clippy.
  - Clippy rules met: lib code avoids std String/format!, indexing, slicing and panic.
- **PR body.** The PR #13412 body is stale (it describes V4); the pushed state is the map mechanism. A new body is below; replace `MEASUREMENTS` with:
  - **Release size (run 44517626548, success):** `vize` release 65,970,072 B base `8b09dea8b1` = head `be718c0d4a`. The stripped size must be read from the run log (`stripped bytes` lines).
  - **IR identity (run 44517575035, bench/levels-ir): FAILED with no report printed.**
    - Inspect the log and the `ir-identity` artifact.
    - Likely a workflow-script issue, not a finding: the `Compare` step prints nothing; check `build.log`.
    - Expect exactly 2 changed functions, the `Debug::fmt` of the two renamed types, because their name strings change length (15→14 and 23→25 bytes).
  - Rerun after fixing: `gh workflow run level-instruction-counts.yml --ref bench/levels-ir -f base_sha=<main> -f head_sha=<head> -f args="--rename-path vize_l1::markup::htmlish::view::selected::NativeTemplateComponent=vize_l1::markup::htmlish::view::selected::SelectedTemplateComponent --rename-path vize_l1::markup::htmlish::view::NativeComponent=vize_l1::markup::htmlish::view::OwnedComponent"`.
  - Rerun release size: `--ref bench/levels-release-size` with `base_sha`/`head_sha`.
  - Do NOT enable auto-merge until IR + size are in the body.

## Next steps

1. Get #13412 green and its evidence into the body, then enable auto-merge (`gh pr merge 13412 --auto --squash`).
2. **#13408 successor (L2 concept names), on top of #13412, after #13403 merges.** #13403 carries the move-script engine fixes that branch `davinci-levels-l2-concepts` (`7ec4329354`) duplicates.
   - Rebuild on fresh main:
     - take only the `l2-concepts` step from `7ec4329354:tools/commands/davinci/level-layout-move.ts`;
     - run `node tools/commands/davinci/level-layout-move.ts l2-concepts`;
     - run `cargo fmt --all`;
     - regenerate the owed list (the gate prints the expected list).
   - Register entry `l2-concepts` in `evidence-renames.json` with the single pair `["NativeTemplateIssue","TemplateRouteIssue"]`.
     - Survey result: it is the only renamed L2 name in pinned evidence, and no file already spells `TemplateRouteIssue`.
     - Evidence files (14):
       - the three #7502 outputs;
       - `crates/vize_atelier_sfc/tests/fixtures/`:
         - `native-scriptless-ssr-output.json`, `native-scriptless-ssr-output-v2.json`;
         - `native-setup-ssr-output.json`, `native-setup-ssr-direct-child-output.json`;
         - `native_original_input_model_{checkbox_all_targets,current_outside,modifiers_all_targets,number_all_targets,ssr_sfc_vue_3_5_35,trim_all_targets,vapor_sfc_vue_3_6_rc9}.json`.
   - Switch each gate that compares current output with those files to `spell_back(current, <file>)?` (Rust) or `spellBack(current, file)` (TS).
     - Rust:
       - `src/native_{selected,ssr,vapor}_setup/tests/**/finish.rs`;
       - `src/native_selected_setup/tests/current_model_outside.rs`;
       - `tests/native_input_model_trim/finish.rs`;
       - `tests/native_scriptless_setup_ssr.rs`;
       - `tests/native_scriptless_ssr.rs` and `tests/native_scriptless_ssr/class_successor.rs`;
       - `tests/native_setup_ssr/direct_child.rs`.
     - TS: `tests/tooling/native-*-reference.test.ts`, `tests/tooling/support/native-input-model-current-outside.ts`, `native-attribute-values-7502-scriptless-history.ts`.
     - Add the map and both readers to those capture triggers (as done for #7502).
   - Evidence: IR identity via bench/levels-ir, with `--rename-path` per renamed L2 type and module moves. Do not use `--rename`: it renames bare identifiers in every crate. Plus release size.
3. Later in levels.inputs: L1→L2 native names (24 owed entries), the css→style move (branch `davinci-levels-l1-style-vue`, has its own handoff), the TSRX lowering move, and structural steps 5–9.

## Tools

- `tools/commands/levels/ir-identity.ts`: PR #13410, auto-merge on.
  - When it lands, tell the coordinator; the codenames agent reuses it.
  - CI runner: branch `bench/levels-ir` (`bb98000908`). It replaces `.github/workflows/level-instruction-counts.yml` with the IR job (inputs `base_sha`, `head_sha`, `args`). It is not for main.
- First result: #13403's Vue move, vize_l1, 999/999 identical. The 6 functions only in base are the deleted `moved` module.

## Open decisions

- None beyond the coordinator's (a) ruling. The IR rule for renames: `Debug` name-string changes are the expected and only IR differences, and must be listed explicitly in the PR body.

## Ready-to-use PR body for #13412 (replace MEASUREMENTS)

See the commit messages of `be718c0d4a` and `993781cbdb`. Sections:
- **Renames:** the table above.
- **Pinned evidence through one rename map:** the mechanism, validation and #7502 gates.
- **Measurements.**
- **Checks.**
