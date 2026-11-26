# Third-party triage handoff, 2026-11-26 (evening JST)

The user is moving to another machine. Every piece of work is pushed. Nothing here needs local state.

## Done today
- **v0.798.0 to v0.803.0 shipped.**
- **#13560 Stage 2** (#13731): in v0.803.0, verified on the published binding, closed.
  - Verification: https://github.com/ubugeeei-prod/vize/issues/13560#issuecomment-7189994383
- **#13567:** status on 0.799.0 posted: https://github.com/ubugeeei-prod/vize/issues/13567#issuecomment-7190040527

## 1. PR #13765: bulk-edit HMR regression (Refs #13565)
`fix(vite-plugin): compile the SFCs of one hot-update wave in native batches`
- **State:** OPEN with auto-merge on, head `ef4fdd6f99`. CI was still pending when the session stopped; nothing had failed.
- **Root cause:** #13700 made the client `hotUpdate` hook accept compiled modules that were first requested by their plain `.vue` URL. Each hook then ran a single-file native compile, and those calls ran one after another. When all 1,000 SFCs were edited at once, HMR was blocked for about 50 s on 0.799.0, against 11 s on 0.797.0.
- **Fix evidence:** time until the hooks are quiet fell from 17–32 s to about 10 s (the bench floor). The longest hook fell from 16–24 s to 0.4 s. 1,000 single-file compiles became 9 batches. Outputs are byte-identical.
- **TODO:**
  1. Confirm it reaches MERGED. If it drops out of the queue, rebase and re-queue it. It may conflict with the other session's #13754 and #13764; a test merge had no conflicts.
  2. Release with `gh workflow run release.yml --ref main -f source=<full sha>`.
  3. Verify on the published package, then comment on #13567 and #13565.

## 2. #13547: template source maps, compile-time split (supersedes #13620)
- **User decisions:**
  - Split mapped and no-map at compile time, and accept the size growth.
  - No-map rows must be **exactly 0**.
  - On 11-26 the user chose **separate copy crates**, after an experiment showed that a copy inside the same crate always shifts LLVM's inlining of shared generic instances.
- **Branch:** `fix/sfc-template-maps-composed` @ `dfb99a2111`, based on main `1249b1608d`. No PR yet. `wip/composed-copy` @ `6f385176e8` holds the earlier v14 work.
- **Design:**
  - One copy macro each in core, DOM and SSR.
  - `tools/commands/generate/composed-crates.ts` writes `vize_atelier_core_composed`, `vize_atelier_dom_composed` and `vize_atelier_ssr_composed`. The generated files carry do-not-edit headers, and `check-js` runs the generator with `--check`.
  - The SFC crate calls a copy only when a map is requested.
- **Open runs:** shipped-profile `45821609242` and level-instruction-counts `45821554893`.
- **TODO:**
  1. Read both runs. No no-map row may move up.
     - The A/A twin control currently fails on `formatter_script_large` on main itself: `cf3ae603ed` against its own twin moved that row. Read the instruction rows separately.
  2. Rerun the full tests. The last run on the rebased branch was 2273 passed and 74 failed. The failures checked were missing `node_modules` (`happy-dom`, `vite`), but not all were checked. Reinstall first with `pnpm install --frozen-lockfile --ignore-scripts`, then confirm 0 failures.
  3. One byte-custody failure is in `crates/vize_croquis/src/sfc/css_transform.rs`, which this branch doesn't touch. Check whether main fails it too.
  4. Open the PR (not a draft) with `PR-BODY-13547.md` from this branch, and fill in its evidence section.
     - `Closes #13547`, supersedes #13620.
     - `Co-Authored-By` RyutaroYako.
     - Turn on squash auto-merge and watch it until it merges.
  5. **Before the next release:** the three new crates need their first crates.io publish, or a trusted-publishing setup. This is a manual step.
  6. Then release, verify, and close #13547 and #13620.
- **For the record:** v10 had 5 rows up and +0.7% size. v13 had 23 rows moved (11 up) and a CLI +529 KB larger.

## Gotchas
- **Repro bench:** `NAPI_RS_NATIVE_LIBRARY_PATH` also hijacks rolldown's native binding. To swap in vize's binding, preload a `Module._load` hook that matches `vize-vitrine.*.node`.
- **pnpm:** the global `minimumReleaseAge` policy blocks new vize versions. Add the version to the existing `minimumReleaseAgeExclude` entries.
- **Disk:** other sessions (`nuxt-auto-import`, `useTemplateRef`) hold about 60 GB of `target`. Free space fell to 2–7 GB at times.
- **Owned by another session:** #13697, #13689, #13558, #13561, #13565 (the perf track), and #13754/#13764.
