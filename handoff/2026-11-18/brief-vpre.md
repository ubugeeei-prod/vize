You are a worker for the Davinci completion leader on ubugeeei-prod/vize. Do NOT spawn agents. First read and follow the common lane brief (environment, sandbox, gh/git unsandboxed, cargo-slot wrapper, toolchain, GitHub and zero-regression rules):
/private/tmp/claude-501/-Users-ubugeeei-Source-github-com-ubugeeei-prod-vize/a4027c37-ba23-4df3-b9d7-89015938e95f/scratchpad/lane-brief.md

## CI restriction (queue-drain mode)
Runner capacity is throttled. Do NOT push, open a PR, start a workflow_dispatch or rerun anything until the leader says so. Work and commit locally, then report with the PR body drafted in a scratch file. When you may push: at most one workflow_dispatch in flight, and the orchestrator feeds the merge queue, so open the PR without arming it unless told.

## Mission: fix #13314. DOM codegen resolves components inside a `v-pre` subtree.
Read the issue first: `gh issue view 13314`. Inside a `v-pre` subtree Vue keeps every element plain (compiler-core `onCloseTag` sets `tagType` only when `!inVPre`; this covers the v-pre element itself and all its descendants). Vize's SSR already matches Vue. Vize's DOM output instead emits `_resolveComponent("Child")` for `<div v-pre><div><Child>{{ x }}</Child></div></div>`.

The pointer: `crates/vize_armature/src/parser/element/classify.rs` `determine_element_type` forces `ElementType::Element` inside v-pre only when `frozen_elements` is set (`parse_with_frozen_elements`). Find out why that gate exists before changing it: use `git log -S`, the tests around `frozen_elements` in `parser/attribute/head.rs`, and the docs/decisions. Then make the ordinary parse follow Vue.

The native Davinci path (L1 → L2 → L4) must agree byte for byte with the legacy path. Check how `davinci/vize_l1_to_l2` decides component vs element under v-pre, and fix it there too if it differs.

Oracle: Vue 3.5.35 `@vue/compiler-sfc` at `/Users/ubugeeei/Source/github.com/ubugeeei-prod/vize.13110-workspace-runtime-props/node_modules/.pnpm/@vue+compiler-sfc@3.5.35/node_modules/@vue/compiler-sfc`. For Vapor use `@vue/compiler-vapor@3.6.0-beta.10` from the same `node_modules/.pnpm`. Compare DOM (`compileTemplate`, and `compileScript` with `inlineTemplate: true`), SSR and Vapor output for:
- `<div v-pre><div><Child>{{ x }}</Child></div></div>`;
- `<Child v-pre>{{ x }}</Child>` (the v-pre element itself);
- `<div v-pre><component is="x"/></div>`;
- `<div v-pre><my-comp/></div>`;
- `<div v-pre><slot/></div>`;
- `<div v-pre><template v-if="a">x</template></div>`;
- a control with no v-pre (`<div><Child/></div>`), which must stay a component.

Requirements:
- Compiled output byte-identical for every input without a component-like tag inside `v-pre`: compile-bench (`compile-bench.yml`, Blacksmith, one dispatch in flight) plus the corpus suites.
- A compiler differential case under `tests/_fixtures/differential/compiler/` (the legacy-fix gate #6852 requires a new active case for a `fix(...)` PR). Copy an existing `compiler/sfc/*` case's layout.
- Fixtures for DOM, SSR and Vapor parity, with exact oracles only (the assertion lint bans `contains`, `starts_with` and `ends_with` in asserts).
- Run every crate that has a `legacy` feature (vize_armature, vize_atelier_core, vize_atelier_dom, vize_atelier_ssr, vize_atelier_sfc, vize_l1) also with `--features legacy`: frozen witnesses such as `vize_armature` `tokenizer_contract_tests` only run there. A frozen digest that changes needs a reviewed correction, following `crates/vize_armature/src/parser/tokenizer_authored_structure_witness.rs` on #13313. Known pre-existing failure with legacy: `vize_atelier_ssr` `vue1_triple_mustache_emits_raw_html_without_the_escape_helper`.
- Full suites for vize_armature, vize_l1, vize_l1_to_l2, vize_l3, vize_l4, vize_atelier_core, vize_atelier_dom, vize_atelier_ssr, vize_atelier_vapor, vize_atelier_sfc, vize_patina, vize_croquis and vize_glyph: `--no-fail-fast`, each `-p` crate wrapped in cargo-slot. Diff failure sets by name against origin/main. Many runtime-mount and tsgo tests fail locally for environment reasons.
- Patina and the linter may move if they classify v-pre components. If so, list each change; a lint change that aligns with Vue is intended.
- Base the work on origin/main. If #13313 (`fix(compiler): keep a Vue template's authored structure`) has not merged when you push, base on main anyway: it does not touch element classification. Rebase if it lands first.
- Worktree: `git -C /Users/ubugeeei/Source/github.com/ubugeeei-prod/vize worktree add ../vize.vpre -b fix/v-pre-subtree-elements origin/main`, unsandboxed, with its own `CARGO_TARGET_DIR=<worktree>/target`. Disk is tight (about 20 GB free), so keep the target under about 12 GB.
- The PR title is `fix(compiler): keep a v-pre subtree's elements plain, as Vue does`. The body has "Fixes #13314", a "Behavior change" line, the oracle table and the compile-bench run id. (When pushing is allowed) open the PR, do not arm it, and report its number. Commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`, and the PR body ends with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Everything is in English.

## Report back (final message, concise English)
PR number and state, what changed per path (legacy, native), why the `frozen_elements` gate existed, the oracle comparison per fixture, updated snapshots or pins with a reason each, the failure-set diff, and anything that needs a maintainer decision, as options.
