# Handoff: Nuxt auto-import track, 2026-11-26

On 2026-11-26 the user reported feedback that vize works badly with Nuxt auto-imports ("とにかく対応してリリース"). Later they said "どんどん探してどんどん直してどんどんリリース" and "ちゃんとマージまで持っていってね". At 21:37 JST they left to switch machines.

## Shipped
- **v0.801.0:**
  - #13722: an SSR build bundled externalized `vue-router` as CJS, giving 500 `require is not defined` on pages using auto-imported `onBeforeRouteLeave`.
  - #13725: Nuxt 4.7 defaults `experimental.compilerComponentResolution` on, so lazy hydration was lost; template `_ctx.resolveComponent` was also broken.
  - Verified on the published packages: SSR HTML is byte-identical to @vitejs/plugin-vue.
- **v0.803.0** (released by another session): includes #13732, lint `vue/no-undefined-refs` with Nuxt names, the `globals` option and Options API members.

## Open PRs at handoff (each must reach merged)
- In the merge queue:
  - #13736: check, auto-imported types re-exported in Nuxt's form.
  - #13754: HMR, stale setup and component add/rename.
  - #13737: JSX lowered in SFC scripts.
  - #13755: SSR, declared directives and setup bindings in dynamic keys.
- Auto-merge on, CI re-running after fixes:
  - #13759: lint, `.client`/`.server` suffixes, `v-t` and static `is`. Its snapshot custody successor is `tests/_fixtures/_projects/lint-review/2026-11-26-nuxt-mode-suffixes`.
  - #13764: HMR for SFCs without `<script>`.
  - #13766: LSP gets the Nuxt names through `vize_carton::nuxt`.
- **A PR that leaves the merge queue loses auto-merge.** Re-run `gh pr merge N --auto --squash`. CodeRabbit CHANGES_REQUESTED does not block merging.

## Follow-ups to open after their parent merges (no stacked PRs in the queue)
- **`fix/check-nuxt-builtin-and-nitro-types`** (head `2bd5e33fe3`), after #13736.
  - Rebase: `git rebase --onto origin/main 6c54a9937f`, where `6c54a9937f` is the merge commit.
  - Contents: Nitro augments via `nuxt.d.ts` references; built-in, `Lazy*` and `.server` component props typed; fallback aliases dropped when generated types exist; 4 CodeRabbit fixes from #13736.
  - Known gap: island (`.server.vue`) prop rows still differ from vue-tsc (vize_canon `$HasInstanceRawProps` on `IslandComponent<T>`).
- **`fix/vite-jsx-review-followups`** (head `6abeab3787`), after #13737.
  - Rebase: `git rebase --onto origin/main b48a7bb7cb`, where `b48a7bb7cb` is #13737's tip.
  - Contents: 5 CodeRabbit fixes. Two are real bugs: a vue import collision was silently dropped, and SSR used the client `jsxMode`.
- When everything has merged, dispatch a release: `gh workflow run release.yml --ref main -f source=<full sha>`. Then verify on published packages with a copy of the hunt app: `cp -Rc`, `nuxt prepare`, build, then diff SSR HTML against `VIZE_OFF=1`.

## Remaining known bugs, not started
- Nuxt 4.7 template warnings (NUXT_B4025/B3014/B3015) are lost, because vize runs no Vue `nodeTransforms`. This is a design gap.
- A normal `<script lang="tsx">` with TS annotations plus JSX: native `strip_typescript` parses as TS. The fix is native JSX lowering in vize_atelier_sfc.
- Lint: `match-component-file-name` / `multi-word-component-names` still include Nuxt suffixes. `mixins` / `...mapState()` / `defineNuxtComponent` members are still flagged by `no-undefined-refs`.

## Repro apps (scratchpad of session f406bf4c, on the old machine)
- `app1`/`app2`: Nuxt 4.7 + vueuse + pinia; app2 also has `round2.vue`.
- `hunt/base`: broad scenarios with HMR and Playwright drivers.
- `serve.sh` / `dev.sh` compare against `VIZE_OFF=1`.
- To recreate on a new machine: a Nuxt 4.7 app with `@vizejs/nuxt` and a `VIZE_OFF` switch in `nuxt.config`.

## Gotchas learned
- **Pins that new fixtures or changed outputs move:**
  - corpus sweeps over `tests/_fixtures/_projects` (remarks folios, `remarks-backlog.md`, `vize_incremental` equivalence planes);
  - markup battery census;
  - explain snapshots plus the `diagnostic-documentation-11818` fixtures and their sha pins;
  - the lint snapshot successor chain;
  - `compilerReferenceRepins` in `compiler-css-binding-custody`;
  - `path-host-callers.ts` call counts;
  - byte-custody registry rows;
  - `vite-runtime.mjs` hashes, pinned in both Rust and TS;
  - docs over 500 lines may not grow;
  - the JS warning budget is 0.
- **Agents in the shared worktree:** copy their files onto a fresh branch from origin/main. Apply a patch for any file main changed since their base; never copy it wholesale.
- **Disk:** it filled several times. Delete idle `target/` dirs, and check `lsof` first.

Related: [[vize-user-feedback-speed]], [[vize-fleet-rules]], [[vize-better-not-compatible]], [[vize-no-stacked-prs-in-queue]].
