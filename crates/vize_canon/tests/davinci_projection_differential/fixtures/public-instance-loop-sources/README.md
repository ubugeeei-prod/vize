# Additive loop-source projection witnesses

The binding-check differential originally counted 64 template diagnostics, plus
the separately authenticated 7 registered carriers and 2 SVG carriers. #10933
adds unresolved reads for same-element `v-for` collection expressions. Two
existing fixture inputs now contribute comparisons that were previously absent:

| Source below `tests/_fixtures`                                  | SFC span    | Template span | Read           |
| --------------------------------------------------------------- | ----------- | ------------- | -------------- |
| `_projects/ecosystem-products/src/NaiveUiOptionsApi.vue`        | `2474:2486` | `347:359`     | `visibleTasks` |
| `vue-language-tools/upstream/test-workspace/tsc/#3121/main.vue` | `53:64`     | `43:54`       | `slotFilters`  |

`slotFilters` is unbound in the upstream fixture's empty script setup. In the
Naive UI fixture, `visibleTasks` is an Options API computed property. This test
intentionally calls `type_check_sfc` with the existing default binding-check
options, without enabling Options API resolution; the original census already
contains the same fixture's other data/computed/method reads. This addition
extends that unchanged low-level projection contract to its collection read.
It does not claim that an Options API consumer should reject this component.

`additions.json` retains each complete authored source, byte length and SHA-256,
the complete diagnostic, template base and exact authored span/text. Admission
requires the complete observation to match. Other diagnostics in these same
files remain original comparisons. Missing, duplicate, changed-body, changed-span
and substituted-path observations fail closed.

`comparisons.json` retains all 66 observations before admission from
[run 40492880810, job 122151205163](https://github.com/ubugeeei-prod/vize/actions/runs/40492880810/job/122151205163),
at merge source `8b700d458ecf5ab9f61d84f61948b73d23029213`; its metadata also
records the PR head and raw job-log SHA-256. `original-comparisons.json` retains
the 64 exact path/diagnostic/span/text identities after separating these two
audited reads. The test requires this exact original list as well as its
unchanged count, and a same-count mutation control guards the list comparison.

The existing 64, registered 7 and SVG 2 counters remain independent. The two
loop-source additions have a separate exact counter and cannot replace them.

`baseline-observer.rs.txt` is the complete capture-only test from
`626b4f2b25437ec9d4b2cff46181586babec0105`, before the additive registration.
Its immutable bytes, SHA-256 and original baseline source are recorded in
`baseline-observer.json`. Copying this observer onto
`481c94bd55a7276339adc277edd93e30313f6ea9` leaves the production source intact
and permits an Actions job to compare the real before observations against the
64-entry retained list. This binding-only test needs the committed corpus and
`--features legacy-differential`; it needs no npm installation, submodule or
native checker process.

The immutable before producer was independently replayed in
[run 40494413713, job 122157096840](https://github.com/ubugeeei-prod/vize/actions/runs/40494413713/job/122157096840).
It checked out `481c94bd55a7276339adc277edd93e30313f6ea9`, copied only the
retained observer into the test target, and required an empty tracked diff
for every other file. The native test passed once and all 64 complete
observations matched this original registry. `baseline-proof` retains the
complete stdout, stderr, observer-only patch and authenticated receipt; the
complete source index remains in artifact `12730062824`. A local regression
replays the captured log and authenticates its bytes before comparing the
whole original list. The temporary CI capture step was removed after this
proof was retained; the current-source law continues to run on every change.
