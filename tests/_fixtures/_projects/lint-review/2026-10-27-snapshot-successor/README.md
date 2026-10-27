# 2026-10-27 lint snapshot successor review

The 2026-10-21 review still authenticates the original provider captures. Its
current hashes describe the snapshots before [PR #10873](https://github.com/ubugeeei-prod/vize/pull/10873),
which refreshed six lint snapshots without updating that derivative review.
Both historical review tests then failed at the current snapshot hash check.

This archive adds a second, explicit comparison. `snapshots.json.gz` retains the
complete before and after snapshot strings from parent
`b9cecc04d9cd0fc263235b575482d8465de2a341` and snapshot commit
`9228b7e0a75490ea7c9d530c92b274537d0332df`. The old review must pass against the
archived predecessor before the successor may compare the current snapshot.
All original provider ZIPs, job logs, packets, source receipts, review hashes,
and checked-in snapshots remain untouched.

| Fixture        | Changed whole packets | Whole diagnostic changes                                                    |
| -------------- | --------------------: | --------------------------------------------------------------------------- |
| ant-design-vue |                    71 | 72 attribute ranges; 20 plain-text help fields                              |
| elk            |                    10 | 10 plain-text help fields                                                   |
| misskey        |                   277 | 988 attribute ranges; 7 plain-text help fields; 4 added component listeners |
| npmx.dev       |                    30 | 52 attribute ranges; 11 plain-text help fields                              |
| reka-ui        |                     6 | 6 plain-text help fields                                                    |
| vuefes-2025    |                     1 | 1 plain-text help field                                                     |

The unchanged nuxt-ui snapshot continues to use the original review directly.
All seven fixture Git pins and every ordered file identity stay unchanged.

`source-witnesses.json.gz` retains all 395 changed packets' complete source
files, retrieved from each repository's immutable raw Git URL. Every source is
checked against the Git blob, byte length, and fixture pin in the original
1977-file or 803-file provenance archives. SHA-256 and Git blob hashes are also
recomputed from the retained bytes. The loader checks every delta against these
sources and admits only:

- 1112 `vue/attribute-hyphenation` ranges that cover the authored attribute and
  contain the original name range, as implemented by #10809. All other message
  fields stay identical.
- 55 `vue/require-scoped-style` help fields with Markdown backticks removed,
  matching JSON's existing `render_help(PlainText)` boundary. The #10205
  historical wording remains bound by the original review.
- Four `vue/v-on-event-hyphenation` warnings for `@updateProps` (twice),
  `@update:modelValue`, and `@headerWheel` on components in the unchanged Misskey
  sources, as implemented by #10852. Only the corresponding three packets'
  warning counts increase.

The new archive includes complete packet hashes, field classifications, message
totals, and both whole snapshot hashes. Unreviewed provider, predecessor, or
current bytes fail the original or successor boundary. Existing direct-client
correction witnesses and the explicitly unproven indirect guard remain intact.

Validation: both historical tests reproduce the original hash failures on
unchanged main `530b4ec43ffa9621c04abc28c09272fb236b452a`. The two-step review,
all original packet/source/oracle tests, and chain mutation controls pass after
admission. This is a historical evidence repair, not a new App E2E run or a
claim that every current real-project lint run is green.
