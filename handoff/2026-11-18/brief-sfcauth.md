You are a worker for the Davinci completion leader on ubugeeei-prod/vize. Do NOT spawn agents. First read and follow the common lane brief (environment, sandbox, gh/git unsandboxed, cargo-slot wrapper, toolchain, GitHub and zero-regression rules):
/private/tmp/claude-501/-Users-ubugeeei-Source-github-com-ubugeeei-prod-vize/a4027c37-ba23-4df3-b9d7-89015938e95f/scratchpad/lane-brief.md

## CI restriction
Blacksmith runners are starved right now (about 84 queued). Do NOT push, open a PR, start a workflow_dispatch or rerun anything until the leader tells you runners are back. Work and commit locally, then report. When you may push: at most one workflow_dispatch in flight.

## Mission: SFC templates keep their authored structure in the compiler (orchestrator decision "A")
For SFC/component templates in the default `templateSyntax: "standard"`, keep the authored structure exactly as Vue 3.5 does, the same principle as #13307 (tables). Keep the HTML tree-construction fix-ups for in-DOM/document inputs only. Today the compiler applies them to SFC templates. For example, `<form @submit="a"><form @submit="b">` drops the inner form and its handler from the compiled output, and `<p><ul>` compiles as siblings. Both are runtime bugs for drop-in users.

A previous worker did most of the code. Its full state, the failing-test list and the remaining steps are in:
/private/tmp/claude-501/-Users-ubugeeei-Source-github-com-ubugeeei-prod-vize/a4027c37-ba23-4df3-b9d7-89015938e95f/scratchpad/sfcauth/HANDOFF.md
Read it first. Continue in the same worktree `/Users/ubugeeei/Source/github.com/ubugeeei-prod/vize.sfc-authored`, branch `fix/sfc-templates-keep-authored-structure-w`, with its own `CARGO_TARGET_DIR=<worktree>/target`. Its baseline worktree is `../vize.sfc-authored-base`. Delete the stray untracked probe `crates/vize_atelier_sfc/tests/zz_tbody_probe.rs`. The scratchpad folder `sfcauth/` holds the Vue 3.5.35 oracle scripts (`compare.cjs`), the case input and the oracle outputs.

Stack: the branch sits on #13308 (`origin/fix/l1-sfc-templates-keep-authored-structure`, itself on #13304) plus the cherry-picked #13307 commits. When those merge, rebase onto origin/main and drop the cherry-picks.

Decisions already made, so do not re-ask:
- The native parse scope stays as the previous worker built it. Every Vue 3 component parse is authored, including native lint (TAG_FACTS), the formatter and the native SFC path, so the native and legacy paths agree byte for byte. Patina's native suites will move. Update them to the authored behaviour, and list each one with its reason. Native rules that used to refuse with RecoveryContext on these shapes may now lint directly. That is expected, but the native and complete products must agree.
- Omitted optional end tags (`<li>a<li>b`) stay lenient: no compiler error. Invalid-nesting warnings belong in lint rules, not the compiler.
- Vapor keeps the browser's repairs (`html_tree_construction: true`) in this PR, so its output stays byte-identical to main. Mention in the PR body that Vue 3.6 Vapor splits invalid nesting into separate templates, which is a separate follow-up.

Requirements:
- Compiled output byte-identical for every corpus input without these shapes: compile-bench (Blacksmith, once runners are back) plus the corpus suites.
- Fixtures against the Vue 3.5.35 oracle: nested form, `a` and `button`; `<p><ul>`, `<p><div>` and `<p><p>`; misnested formatting; omitted li/dt/dd/option; the table shapes; and SSR and Vapor targets.
- A compiler differential case `compiler/sfc/sfc-authored-structure` (legacy-fix gate #6852), plus a `crates/vize_atelier_sfc/tests/sfc_authored_structure.rs` test, both described in the handoff.
- The docs: the Template Syntax section of `docs/content/guide/compiler-configuration-reference.md` describes exactly what `standard` still rewrites. The decision record's Evidence paths must point at the real case and test.
- Full suites for vize_armature, vize_l1, vize_l1_to_l2, vize_l2, vize_l3, vize_l4, vize_atelier_core, vize_atelier_dom, vize_atelier_ssr, vize_atelier_vapor, vize_atelier_sfc, vize_patina, vize_glyph and vize_maestro: `--no-fail-fast`, each `-p` crate wrapped in cargo-slot. Diff the failure set by name against the base worktree. Many runtime-mount, tsgo and corsa tests fail locally for environment reasons, so compare names.
- Squash the WIP into conventional commits. The PR title is `fix(compiler): keep a Vue template's authored structure`. The body has a "Behavior change" line naming the shapes and the Vapor note. Commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`, and the PR body ends with `🤖 Generated with [Claude Code](https://claude.com/claude-code)`. Everything is in English.

## Report back (final message, concise English)
Branch, commit SHAs, and what changed per path (armature, L1, L1→L2, targets, Patina, docs). Then the oracle comparison per fixture, each updated snapshot or pinned test with its reason, the failure-set diff against base, and anything that needs a maintainer decision, as options. If you run low on context, update HANDOFF.md first and say so in the report.
