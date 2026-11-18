You are a worker for the Davinci completion leader on ubugeeei-prod/vize. Do NOT spawn agents. First read and follow the common lane brief (environment, sandbox, gh/git unsandboxed, cargo-slot wrapper, toolchain, GitHub and zero-regression rules):
/private/tmp/claude-501/-Users-ubugeeei-Source-github-com-ubugeeei-prod-vize/a4027c37-ba23-4df3-b9d7-89015938e95f/scratchpad/lane-brief.md

## Mission: JSX round 3 on the level lowering (#6885, one SFC/JSX abstraction)
Maintainer priority (#6826): SFC, JSX and TSRX lower to the same L1/L2. Read #6885 (latest comments) and the JSX decisions in `docs/davinci/decisions/` (`*jsx*`). Round 2 merged #13160, #13169, #13183 and #13190: `vize_l1_to_l2::framework::vue::jsx::{walk_module, lower_module}`, models and directives, slot children and `v-slots` objects. 169 of 226 oracle-corpus roots now take the level lowering, and every one's L2 equals the Relief projection's.
Still on the Relief path, in priority order:
1. **Slice 2b, `&&` over values and non-element branches.** It needs the neutral L2 op that the TSRX worker is adding (`ui.and` or a value branch, PR on branch `tsrx/l2-and`; check `gh pr list --search "ui.and"`). If it has not merged, start with item 2 and come back.
2. **Step 5, `ui.child-spread` and `ui.render-expr`.** New L2 ops need match arms in about six `crates/vize_patina/src/markup/**` files. You may edit them; rebase around the perf shepherd's #13258 and #13251 and the TSRX worker.
3. **Opaque `v-slots` objects and `v-slot`.**
4. **Behaviour fixes** (each its own `fix(jsx)` PR, with oracle rows turned equivalent; #13167 is already handled by the leader in #13289): #12322 (slot value returns and mixed text use `toDisplayString`; `@vue/babel-plugin-jsx` normalizes them as children), #12313 (`_` modifiers on events and custom directives, vue-jsx-vapor; its pinned oracle rows are run-once rows, step 7).

## Requirements
- Lowering steps: compiled output byte-identical (VDOM, Vapor, SSR, Babel) over both oracle corpora, the tracked JSX/TSX fixtures and element-plus's TSX files. Diff base vs head dumps with `cmp`, as round 2 did. Quote the witness count of level-lowered roots before and after.
- Behaviour fixes: only the intended rows change, with tests. Never relax an oracle row; turn a divergent row equivalent only when the output now matches the reference.
- Perf: for L2, lowering or patina-markup changes, dispatch `level-instruction-counts.yml` (read its inputs; Blacksmith only) and, for compiler output paths, `compile-bench.yml`. Put the run ids in the PR body. Must not be slower.
- Record decisions as short English comments on #6885 (options, with the more reversible option applied by default).
- Create your own worktree from origin/main (`git -C /Users/ubugeeei/Source/github.com/ubugeeei-prod/vize worktree add ../vize.jsx3 -b jsx/<step> origin/main`, unsandboxed). Never build in the main checkout. At most 2 open PRs at a time. Arm with `gh pr merge <N> --auto` and follow each until MERGED. Clean target dirs when done.
- Do not touch these leader-owned PRs: #13231, #13262, #13232, #13250, #13257, #13234.

## Report back (final message, concise English)
PRs with state and merge SHAs, the witness counts, evidence (diff method and result, run ids), decisions recorded, what is still on the Relief path, and anything that needs a maintainer decision (as options).
