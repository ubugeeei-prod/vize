# Drain handoff: #13427 #13428 #13429 #13571 #13574 (2026-11-24 JST)

Fleet rules apply (`.handoff/fleet/fleet-brief.md` on `handoff/davinci-fleet-2026-11-19`).
Release-profile evidence comes from `shipped-profile-measure.yml` (base/head + twins). On the
current runner pool, 29 instruction rows are bimodal between builds of the same source. Judge a head
value against every base build pooled (see the PR bodies), not against the primary base alone.

## State

- #13431: **merged** (baddbe9417, 21:32 JST). Release rows unchanged, CLI −256 B, N-API −384 B.
- #13427: **closed**. Release CLI +115,648 B and N-API +114,736 B (local aarch64 fat LTO), ci-opt
  CLI +175,744 B. Cause: the `NoProvenance` Vue lowering instantiation inlines far more once SSR and
  Vapor use it. The reason and the symbol table are in `drain-perf/close-13427.md`. Branch
  `perf/observer-driven-lowering-11573` is kept (rebased, abbd915b3c).
- #13428 `perf/consumer-provenance-11573` @ 4232abea3a: evidence clean. Release rows: only
  `patina_s2_markup_one_root` −303, none up. CLI −39,424 B, N-API −44,016 B (run 45361608324).
  The Vitrine transform half was reverted because it grew the N-API library by 35,600 B. Its PR CI
  was cancelled during the queue backlog.
- #13429 `fix/vapor-static-attribute-spelling` still points at the old stacked head (base =
  13427's branch). The standalone fix is on `fix/vapor-static-attribute-spelling-standalone` @
  c2bfccde2a: the six fix/test commits cherry-picked onto 5a48a7d7e2, minus the 13427-dependent budget
  tighten. Release rows: 5 Vapor compile rows down (up to −4.1%), none up. CLI −4,992 B,
  **N-API +512 B** (run 45374965978). This is the fix's own code (~1.1 KB of text), so it **needs a
  maintainer decision** under the no-size-growth rule.
- #13571 + #13574: the old stack (#13514/#13516/#13519) was combined into #13628
  (`feat/scoped-scss-component-stack`). Both are rebased into one branch,
  `refactor/expanded-css-and-line-laws` @ 07f36936a0, on `origin/feat/scoped-scss-component-stack`
  (f5f08cbebe): 13571's commit plus 13574's change, with conflicts resolved onto the extracted
  workflow scripts and the new ARM job. IR identity with default features against the old stack tip:
  vize_l0/l2/l4/atelier_sfc all identical (12,728 functions). Local: assertion lint, fmt,
  source-length, `lsp_breaks`, `lsp_line_breaks`, `expanded_css` and the feature clippy pass. The
  full clippy and test run was not finished (disk fell to 28 GB).

## Next steps (one PR in CI at a time; push once each after #13633 merges)

1. #13428: `git merge origin/main` into `perf/consumer-provenance-11573`, push, wait for green PR CI.
   Fill `PERF_OBS` in `drain-perf/body-13428.md` from that push's `PR performance observation`
   summary. A/A control: run 45361603294 (5a48 vs 5a48, spread 0.990–1.060). Then
   `gh pr edit 13428 --body-file ...` and `gh pr merge 13428 --auto --squash` (merge queue).
2. #13429: force-push `fix/vapor-static-attribute-spelling-standalone` onto
   `fix/vapor-static-attribute-spelling` (lease 7d7710edf1), merge main, then
   `gh pr edit 13429 --base main --body-file drain-perf/body-13429.md`. Do not enable auto-merge
   until the maintainer accepts the +512 B N-API growth, or until it is removed.
3. #13571/#13574: when #13628 merges, rebase `refactor/expanded-css-and-line-laws` onto main.
   Open one combined PR (title `refactor(ir): emit expanded CSS from input-neutral rules`). Close
   #13571 and #13574 with a link, keeping their branches. Evidence: IR identity
   (`node tools/commands/levels/ir-identity.ts --base <#13628 merge> --crate vize_l0 --crate vize_l2
   --crate vize_l4 --crate vize_atelier_sfc`), an identical release CLI (shipped-profile-measure) and
   identical `vize` binary bytes in the PR performance observation. 13574's part is
   `#[cfg(test)]`/Kani/Lean/workflow only. Then auto-merge.
