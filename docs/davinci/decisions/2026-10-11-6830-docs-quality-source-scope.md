# Keep authored More coverage changes in their actual Docs consumer scope

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Classify only `docs/quality/more-guides-coverage.json` as a known Node Docs
tooling input in both source planners. This authored coverage ledger controls
the existing More guides audit and browser route selection. Its current
consumers do not require workspace Rust tests, native JS package tests or
playground tests for an ordinary PR changing only this ledger and prose.
Keep all existing tooling checks, source-built tooling CLI and native Docs
rendering campaigns. New nearby contracts and scripts retain the conservative
unknown-input fallback; the protected merge queue still runs every full suite.

## Actual consumers and the accidental broad selection

The only runtime reader of the ledger is
[`more-guides-coverage.ts`](../../../docs/scripts/more-guides-coverage.ts).
Its complete local import closure is that reader and the authored Node
[`sitemap.js`](../../../docs/theme/i18n/sitemap.js). The existing
[`docs-more-guides.test.ts`](../../../tests/tooling/docs-more-guides.test.ts)
checks the entire English and Japanese audit, unchanged revised routes,
explicit native captures, and reading order. The existing
[`verify-more-guides-render.ts`](../../../docs/scripts/verify-more-guides-render.ts)
selects browser routes in the full
[`Docs build`](../../../.github/workflows/build-docs.yml), which still builds
the native package and Vite plugin before rendering. This classification
does not remove that real native consumer campaign.

The current workspace Rust source and Cargo manifests contain no reader or
embedding of this ledger or its Docs quality directory. A source-grounded
law visits every actual workspace member, Rust file and Cargo manifest and
guards that boundary. A separate law scans the actual npm product sources,
scripts and manifests for new ledger or reader consumers, refusing linked
source files that cannot establish that ownership. The reader's actual local
import closure is also checked. New workspace member syntax requires review
rather than silently excluding a member from this guard.

At main `d55b8238f89bab4418a1a961bd8db1c84911cb42`, the ledger reached the
unknown-input fallback in both planners. The seven changed paths in
[PR #8642](https://github.com/ubugeeei-prod/vize/pull/8642), source
`a6cbd6d3d0590719c627467ba3dbb8e891e6921e`, were five authored Markdown
files, this JSON and one Docs tooling law. The current source planner selected
all four suites; the same list without the JSON selected tooling only.
The unchanged tooling planner selected 907 tests and included both the More
audit and reviewed translations. The new classification retains that
selection and adds its own durable scope law.

## Original Actions observations

Successful ordinary [Check 38142919072](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072)
checked that exact #8642 source. Each duration below is the job's
`completed_at` minus `started_at`; these include setup, transfer and
scheduling within that boundary.

| Selected product job              |                                                                                          Job ID | Observed wall time |
| --------------------------------- | ----------------------------------------------------------------------------------------------: | -----------------: |
| Native JS package build and tests | [114480417702](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114480417702) |            1,092 s |
| Build affected Rust tests         | [114480417845](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114480417845) |              317 s |
| Rust shard 1                      | [114481437263](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114481437263) |              182 s |
| Rust shard 2                      | [114481437224](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114481437224) |              151 s |
| Rust shard 3                      | [114481437204](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114481437204) |              177 s |
| Rust shard 4                      | [114481437227](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114481437227) |              170 s |
| Rust source report                | [114482056327](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114482056327) |               19 s |
| Playground browser tests          | [114480417748](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114480417748) |              221 s |

Within the JS job, `Test JS packages` took 907 seconds, Fresco declarations
and consumer types 9 seconds, and UI conformance 148 seconds. These native
product campaigns were selected by the unclassified ledger rather than by a
changed package, compiler, fixture or playground source.

The original required tooling CLI producer still took 90 seconds
([114480417849](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114480417849)).
The four required tooling shards took 259/403/327/335 seconds
([1](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114480703930),
[2](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114480703848),
[3](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114480703854),
[4](https://github.com/ubugeeei-prod/vize/actions/runs/38142919072/job/114480703922)).
Those checks remain mandatory. The avoided product rows total 2,329 seconds
of aggregate job wall time across this historical run; they overlap and
**do not establish a measured end-to-end speedup** for the new source.

## Refusal and qualification boundary

The allowlist uses exact path equality. Unknown siblings, schemas, alternate
formats, nested paths, traversal spellings and the reader script itself
still select the full conservative source fallback. Mixed native, compiler,
workflow, Cargo or shared Davinci contract inputs retain their original gates
and affected Rust reverse-dependency closure. Real Git rename and deletion
laws retain both changed paths, including a new unknown destination.
Merge-group context unconditionally retains every source suite and workspace
crate. No workflow, cache trust boundary, command, corpus, threshold, timeout,
receipt or full rendering campaign is altered.

TODO: obtain fresh exact-head Actions for this planner change, then verify a
fresh ordinary ledger-only PR selects the same complete tooling plan and
the product-job skips. Preserve full protected queue and actual native Docs
rendering results before reporting this slice qualified or any causal gain.
