---
name: vize-handoff-2026-11-27-p0
description: "11-27 night handoff: Hal P0 round DONE (0.806–0.808, all 9 closed); main over formatter_template_complex ceiling from #13809 → perf-fix branches pending; whole queue dequeued with re-arm list"
metadata:
  node_type: memory
  type: project
  originSessionId: ee71f8ac-090c-40c4-8d7f-efcaae043c92
  modified: 2026-11-27T13:33:48.084Z
---

The user left on 2026-11-27 at ~22:30 JST ("一旦帰りたい"). See [[vize-hal-spidernight-p0]] and [[vize-feedback-concise-github-comments]].

## Done
- **Hal-Spidernight P0 round:** all 9 issues shipped, verified on the published packages with `scratchpad/hal-verify/run.sh <V>`, replied to and closed.
  - 0.806.0: #13787, #13791
  - 0.807.0: #13788, #13790, #13792, #13793
  - 0.808.0: #13797, #13798
  - #13789: intended behavior; docs added in #13804.
- **Merge queue:** ruleset 24013395 is back to **active**. It was off from ~17:45 to ~19:40 JST.

## BLOCKER: main over the instruction ceiling
- main `55aac8bc81` measures `formatter_template_complex` at 251,152 against a ceiling of 244,347, so **every merge group fails**.
- Cause: #13809 ("bound nesting", merged while the queue was off) runs a two-lexer guard scan on every template expression.
- **Fix in progress** (glyph agent): a short-text certificate in `vize_l0::expression_guard`.
  - Soundness: a structural proof in the module doc, an exhaustive length-6 test over 18 symbols, and a random test.
  - Variant A measures 244,477 (130 over). B (pre-#13809 call shape) and C (B plus an inlined entry) are on CI.
  - Open the PR from the first variant under the ceiling and arm `--auto`. **Do not raise the ceiling.**
  - Follow-up: a recursion limit in the OXC fork (`upstream/forks/oxc_parser`) would delete the pre-scan.
  - Branches:
    - `tmp/perf-guard-certificate-a`: 244,477, over.
    - `tmp/perf-guard-certificate-b`: 244,472, over.
    - `tmp/perf-guard-certificate-c`: run **46005654508** pending.
    - `perf/glyph-template-expression-certificate`: the same tree as C, squashed onto `55aac8bc81` with the final message. Pushed, no PR yet.
  - **Next:** if 46005654508 shows `formatter_template_complex` ≤ 244,347, open the PR from `perf/glyph-template-expression-certificate` and run `gh pr merge --auto`. Otherwise about 150 more instructions must come off the certificate path. Delete the `tmp/perf-guard-certificate-*` branches afterwards.
- **Lesson:** turning the queue off skips the queue-only gates (ceilings). Never turn it off again without a ceiling run on the combined main.

## Re-arm after the ceiling fix merges
- Run `gh pr merge <n> --auto` for: 13771 13777 13776 13739 13782 13769 13736 13778 13729 13781 13811 13810.
- #13810 belongs to the usetemplateref-db session, which agreed that I re-queue it.
- The list is also in scratchpad `dequeued-2-2026-11-27.txt`.
- **Hold #13808** (glyph parent-`>` follow-up, +535): re-queue it only once there is headroom.

## Then
- **#13811** (docs deploy failing since the `build-docs.yml` → `11-docs-build.yml` rename): after it merges, run `gh workflow run 11-docs-build.yml --ref main`. Confirm `Deploy docs` succeeds and that vizejs.dev/guide/type-checking shows "Bare attributes on native elements".
- **#13769** (warflash, closes #13768): after it merges, delete the mirror branch `perf/pr-13769-oxlint-type-world`.
- **#13777** (#13720): after it merges, rebase `fix/canon-global-fallthrough-root` onto main, open the #13721 PR (body in scratchpad `canon-1372x/body2.md`), dispatch level-instruction-counts, then `--auto`.
- **#13778** (#13770): after it merges, refresh the nightly App E2E lint snapshots (renamed rule id).
- **#13774** (#13767): head `d83b3afb07`, rebased. If the lockfile changes, regenerate the canon lock successor and the listhen key.
- **#13775** (#13547): auto-merge OFF. It needs the user to publish the three `*_composed` crates to crates.io manually first.
- **Releases:** run `gh workflow run release.yml --ref main -f source=<full sha>` after the third-party PRs merge. Verify, then post a short reply and close #13768 #13767 #13720 #13721 #13770 #13547.

## Follow-ups noted
- A camelCase `@rowClicked` on packaged generic components is still `any`, although Vue delivers `row-clicked` to it at runtime.
- glyph: idempotence bug with `endOfLine="cr"` plus `{{ }}` wrapping.
- Nine zombie runs from Nov 5 and Nov 25 can't be cancelled.
