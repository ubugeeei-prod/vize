# 2026-11-06 App E2E lint snapshot review

This archive extends, rather than replaces, the 2026-10-21 and 2026-10-27
reviews. It binds the seven lint snapshots captured by
[App E2E lint run 42188006790](https://github.com/ubugeeei-prod/vize/actions/runs/42188006790)
to their complete predecessors at `698cc668042616b43dfb7027f741a5a99fb38adc`.
The fixture Git pins and ordered file packets remain unchanged.

`snapshots.json.gz` retains every complete before and after snapshot, both
whole-file hashes, every changed packet hash, message totals, and field-level
classifications. `inventory.json` authenticates that archive. The loader keeps
the prior two review boundaries intact, then admits this boundary only when the
checked-in snapshots, archived bytes, and current fixture pins match exactly.

The high packet-change counts are almost entirely the reviewed migration from
category documentation paths such as `docs/content/rules/vue.md` to the
corresponding per-rule reference path. The admission policy rejects changes to
any other field on those diagnostics. The remaining semantic changes are
closed explicitly:

- ant-design-vue adds 40 `vue/v-on-event-hyphenation` findings and one
  `script/require-explicit-emits` finding.
- misskey adds 11 `vue/no-template-shadow` findings and removes one
  `vue/no-invalid-html-attribute` finding.
- npmx.dev adds three `vue/no-template-shadow` findings.
- nuxt-ui removes one `a11y/no-static-element-interactions` finding.
- reka-ui adds two `script/require-explicit-emits` findings and removes two
  `a11y/interactive-supports-focus` findings.
- elk and vuefes-2025 change only documentation paths.

Every packet retains all non-message fields, and each warning count must equal
its predecessor plus reviewed additions minus reviewed removals. This records
current source behavior for #10824; it does not claim that every remaining
real-project diagnostic is a product bug or has been fixed.
