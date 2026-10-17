# Refresh application lint expectations after auditing each change

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The latest [main App E2E lint run](https://github.com/ubugeeei-prod/vize/actions/runs/38866347055/job/116860085028)
failed seven snapshots last updated in [#8589](https://github.com/ubugeeei-prod/vize/pull/8589).
All seven upstream revisions and all 2,780 ordered input files are unchanged.
The relevant source, fixture and workflow files are also identical between
the failed run's `b4be3b7` and the CI optimization merge `84b21e2`.

Refresh expectations only after separating intended diagnostic changes from
two actual regressions. Preserve every file and every unrelated diagnostic.

| Application    | Files | Previous diagnostics | Corrected expectations |
| -------------- | ----: | -------------------: | ---------------------: |
| Ant Design Vue |   731 |                  387 |                    388 |
| Elk            |   259 |                  251 |                    180 |
| Misskey        |   583 |                1,753 |                  1,787 |
| npmX           |   220 |                  431 |                    438 |
| Nuxt UI        |   187 |                   20 |                     20 |
| ReKa           |   729 |                  368 |                    471 |
| VueFes         |    71 |                   40 |                     44 |

The retained artifact contains 3,339 diagnostics. The corrected expectations
contain 3,328, removing exactly ten false component-content findings on native
SVG `g` elements and one computed-mutation finding suppressed by an authored
ESLint comment. The accompanying product fixes and original-source controls
establish those corrections; accepting the unmodified artifact would retain
both bugs. Reverse insertion of the eleven complete diagnostic objects and
their owning file counts reconstructs the artifact exactly. The other five
snapshots are byte-identical to their captured output.

The remaining changes correspond to delivered behavior:

- [#9117](https://github.com/ubugeeei-prod/vize/pull/9117) removes 71 obsolete
  Elk border findings. [#8861](https://github.com/ubugeeei-prod/vize/pull/8861)
  honors the actual suppression comments in Elk, Misskey and Nuxt UI.
- [#8877](https://github.com/ubugeeei-prod/vize/pull/8877) detects setup/prop
  collisions. All 73 new ReKa locations match the retained whole-original
  ESLint oracle at the same fixture revision; Misskey's RouterView case also
  matches the pinned reference.
- [#8805](https://github.com/ubugeeei-prod/vize/pull/8805) and
  [#9087](https://github.com/ubugeeei-prod/vize/pull/9087) correct nested and
  split-script attribute inheritance, including the original Ant ThemeIcon.
- [#8798](https://github.com/ubugeeei-prod/vize/pull/8798) and
  [#8797](https://github.com/ubugeeei-prod/vize/pull/8797) correct iteration
  ownership and authored diagnostic spans.
- [#8829](https://github.com/ubugeeei-prod/vize/pull/8829) detects computed
  return fallthrough; [#9024](https://github.com/ubugeeei-prod/vize/pull/9024)
  accounts for spread-provided labels.
- [#8748](https://github.com/ubugeeei-prod/vize/pull/8748) corrects attribute
  ordering across spreads; [#9072](https://github.com/ubugeeei-prod/vize/pull/9072)
  corrects the exact-modifier convention in the original TreeRoot handlers.

The input archive is artifact `12031671138`, SHA-256
`5fbde09ea1075fbf6536bb65dd706327798b1cd9c5a3ea04133dffcb2d1f27e5`.
Against the old expectations, the corrected set removes 150 complete messages
and adds 228, including span/message replacements. No source pin, file
membership, rule severity, assertion, timeout or execution scope changes here.
Fresh complete App E2E remains required before claiming main is healthy.
