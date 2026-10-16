# App inspector qualification: original outputs and retained authority

[App E2E 38890448461](https://github.com/ubugeeei-prod/vize/actions/runs/38890448461/job/116938501368)
failed at `d2506ca6b5e31252973f05981829c0367cacf26b`. Its ordinary app check snapshots
passed; the final inspector comparisons failed. This record preserves the failures and
their original comparisons. It does **not** change the inspector budgets or qualify a release.

## Evidence retention repair

[Issue #9229](https://github.com/ubugeeei-prod/vize/issues/9229) identifies a missing
`VIZE_APP_CHECK_CAPTURE_DIR` in the planned App row. `prepareNuxtCheckFixture` therefore
wrote its complete preparation receipts into runner `/tmp`, while `retainAppCheckPacket`
returned without saving whole checker answers. Neither output reached the failure artifact.
The actual log names `/tmp/vize-vuefes-2025-check-prepare-U7sVxN` for VueFes and
`/tmp/vize-elk-check-prepare-X0LV3o` for Elk.

The row now points that existing capture contract at
`target/app-e2e-logs/check-authority` inside `github.workspace`, already covered by the
row artifact uploader. The helper inventory and failure/cancellation upload conditions
remain intact. It retains the authored manifests/configuration, resolved installation
contracts, generated Nuxt providers, installed declaration candidates, and whole checker
packets that the helpers already know how to capture. Their receipts explicitly distinguish
observed provider candidates from native selected dependency reads and installed tarball
authentication. This patch does not broaden those claims.

The original failed run cannot retroactively acquire its discarded `/tmp` bytes. A fresh
coordinated App check must produce its own receipts before any baseline decision.

## Exact historical Elk comparison

The unchanged Elk Gitlink is `8aa2b4650760eeba4ea1fdd9dc18d8a88ca5d7f0`.
The public [v0.388.0 release](https://github.com/ubugeeei-prod/vize/releases/tag/v0.388.0)
points at `9c0d6a5d9d6879174662e7c0b25eda070e1f83e8`.
Its `vize-aarch64-apple-darwin.tar.gz` SHA-256 was independently recomputed and equals the
release checksum:
`ee85c996e98117f0c9e38ae08e0805093f79c822ce2eafe2bf3c643865a218d4`.

Running that original executable against all 259 pinned Elk SFCs, with exact
`vue@3.6.0-beta.10` and `typescript@6.0.3`, without a resolvable Prettier provider, reproduces
the old SSR budget **exactly**. The compiler subprocess script is byte-for-byte identical
between budget commit `536ed2e47c729a272a1baab03bee8b3a6b529ff1` and the failed current head.
Both complete reports have the same official compiler code for every file.

| SSR measurement          | Original v0.388.0 | Failed current head | Delta |
| ------------------------ | ----------------: | ------------------: | ----: |
| Files inspected          |               259 |                 259 |     0 |
| Changed files            |               259 |                 258 |    -1 |
| Added lines              |             9,201 |               9,217 |   +16 |
| Removed lines            |            25,188 |              25,007 |  -181 |
| Unchanged lines          |                 — |                   — |  +181 |
| Official compiler errors |                 3 |                   3 |     0 |
| Vize errors              |                 0 |                   0 |     0 |

Across all 259 files, 113 Vize code outputs changed. Nineteen files contribute **+29**
added diff lines; the remaining improvements contribute **-13**, yielding the observed
**+16**. The full per-file audit includes every file, source SHA-256, complete official
and Vize code SHA-256, original/current statistics, and category attribution.

### Every positive addition delta

Paths below are relative to `app/`. Deltas count the existing weighted line-diff output;
they are not counts of source lines or automatically permitted credits.

| File                                                | Original → current additions | Delta | Observed correction          |
| --------------------------------------------------- | ---------------------------: | ----: | ---------------------------- |
| `components/account/AccountFollowButton.vue`        |                      34 → 35 |     1 | reactive-props-rest-8837     |
| `components/account/AccountFollowRequestButton.vue` |                      39 → 40 |     1 | reactive-props-rest-8837     |
| `components/modal/ModalDialog.vue`                  |                      27 → 28 |     1 | teleport-root-fallthrough    |
| `components/nav/NavUser.vue`                        |                      33 → 35 |     2 | conditional-root-fallthrough |
| `components/status/StatusActions.vue`               |                      78 → 79 |     1 | reactive-props-rest-8837     |
| `components/status/StatusActionsMore.vue`           |                    118 → 119 |     1 | reactive-props-rest-8837     |
| `components/status/StatusCard.vue`                  |                    127 → 129 |     2 | reactive-props-rest-8837     |
| `components/status/StatusDetails.vue`               |                      57 → 59 |     2 | reactive-props-rest-8837     |
| `components/status/StatusMedia.vue`                 |                      18 → 20 |     2 | ssr-css-vars                 |
| `pages/[[server]]/collections/[id].vue`             |                      88 → 91 |     3 | preserve-page-metadata-7873  |
| `pages/blocks.vue`                                  |                      37 → 38 |     1 | preserve-page-metadata-7873  |
| `pages/bookmarks.vue`                               |                      37 → 38 |     1 | preserve-page-metadata-7873  |
| `pages/compose.vue`                                 |                      33 → 34 |     1 | preserve-page-metadata-7873  |
| `pages/conversations.vue`                           |                      37 → 38 |     1 | preserve-page-metadata-7873  |
| `pages/domain_blocks.vue`                           |                      37 → 38 |     1 | preserve-page-metadata-7873  |
| `pages/mutes.vue`                                   |                      37 → 38 |     1 | preserve-page-metadata-7873  |
| `pages/pinned.vue`                                  |                      37 → 38 |     1 | preserve-page-metadata-7873  |
| `pages/settings.vue`                                |                      54 → 57 |     3 | preserve-page-metadata-7873  |
| `pages/share-target.vue`                            |                      39 → 42 |     3 | preserve-page-metadata-7873  |

The six reactive-rest changes preserve the authored rest binding through Vue's real
`createPropsRestProxy`, corrected by [#8837](https://github.com/ubugeeei-prod/vize/pull/8837)
for [#8835](https://github.com/ubugeeei-prod/vize/issues/8835). The old compiler incorrectly
rewrote rest-object reads to all props and omitted the binding. Existing runtime and
DOM/SSR compilation regressions live in `props-rest-runtime-8835`.

The ten metadata changes preserve authored `definePageMeta` calls, corrected by
[#7873](https://github.com/ubugeeei-prod/vize/pull/7873). Removing those calls to meet a line
count would restore that source-loss defect.

`StatusMedia` gains the SSR CSS variable object and attribute merge; its source contains
`v-bind(gridColumnNumber)`. The historical output omitted the server-side variable.
The CSS ownership and emission checks are covered by the SSR CSS variable suites and
[#7971](https://github.com/ubugeeei-prod/vize/pull/7971).

`ModalDialog` gains `mergeProps` when its Teleport descendant's authored `v-bind="$attrs"`
is rendered. `NavUser`'s two conditional native button roots gain `_attrs` merging.
These are observed behavior changes consistent with their authored attribute ownership;
this audit does not invent a historical corrective PR identity or claim a new runtime
execution of these two complete Elk components. Their exact complete code changes remain
available for the baseline review.

The +16 difference is therefore not evidence by itself that compiler correctness regressed.
The added runtime helpers and preserved source cannot be removed merely to meet a text
metric. The original 9,201 additions ceiling remains enforced in this patch. A separate,
explicitly reviewed baseline regeneration must assess all complete outputs, preserve the
old comparison, and tighten the improved bounds rather than add a blanket exception.

## VueFes official error decline

The unchanged VueFes Gitlink is `8eff3cd7dd79a7f47bbf2b78202b9aa2070ec5a6`.
With the original v0.388.0 executable, workspace `vue@3.6.0-beta.10`, and no generated
Nuxt aliases, the original provider independently rejects exactly:

- `app/pages/timetable/_components/TimetableCard.vue`
- `app/pages/timetable/_components/TimetableCell.vue`

Both errors are `Failed to resolve import source "~~/i18n/timetable"` while resolving
`defineProps<TimetableCell>()`. Original DOM additions/removals are 1,889/2,339; SSR is
3,339/7,353. Both fit the existing ceilings and have two official errors and zero Vize errors.
These are actual old executable/provider observations, not a transcription of the budget.

[#8888](https://github.com/ubugeeei-prod/vize/pull/8888) added real Nuxt preparation before
the original App checks. The official comparator resolves `vue/compiler-sfc` from the app's
current directory before the workspace fallback. The authored VueFes package overrides
Vue to `3.6.0-alpha.2`. An independent exact `vue@3.6.0-alpha.2` provider, with the unchanged
source bytes and a **reconstructed minimal** generated configuration mapping `~~/*` to the
project root, reproduces all 71 current official DOM code outputs byte-for-byte and zero
official errors. Without that alias, the same provider reproduces the two import errors.
CRLF source bytes must be preserved when replaying these files.

This independently explains how preparation/provider selection can eliminate the two errors.
The reconstructed configuration is **not** the discarded original CI `.nuxt` configuration.
The current report's actual additions/removals are 1,753/2,593, so changing only the expected
error count would expose another old bound. No VueFes expectation is changed here; fresh
actual provider and generated configuration receipts are still required.

## Complete retained packets and replay

[The audit manifest](./fixtures/inspector-qualification-20261016/elk-ssr-audit.json) records
all 259 per-file comparisons and hashes for five complete gzip-compressed JSON reports:
original/current Elk SSR, original VueFes DOM/SSR, and failed-current VueFes DOM. Gzip
decompression yields the exact original report bytes, including all code, warnings,
errors, diff records, and statistics. These report packets contain no acceptance override.

Current reports came from artifact `12042492663`,
`app-e2e-full-check-all-38890448461-1`, archive digest
`sha256:5de36fd88757aab533ef576463410962c7e822fe57dd369015aa98b8dc842220`.
Their complete bytes are retained here before the original seven-day artifact expires.

To replay the old Elk comparison, obtain the pinned source archive and original release,
verify the release checksum, expose exact `vue@3.6.0-beta.10` plus TypeScript through the
fixture's local `node_modules`, keep Prettier unavailable, and run from that fixture root:

```sh
/path/to/v0.388.0/vize inspector 'app/**/*.vue' \
  --format compare --target ssr --template-syntax quirks
```

Use `--save-exact` when installing isolated reference providers: a later dependency install
can upgrade an existing caret prerelease request. Verify the actual
`vue/compiler-sfc` exported version before recording reference output. A fresh current
Actions check remains the qualification authority; this historical replay requires no local
Rust build.
