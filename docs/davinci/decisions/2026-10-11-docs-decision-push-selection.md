# Select natural Docs builds from site inputs

Tracking issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Developer decision-only pushes currently select the full main Docs build even
when no site input changes. Keep the existing `docs/**` positive pattern and
exclude only `docs/davinci/decisions/**` immediately afterward. Another included
changed file still selects both original full jobs.

The reviewed site materializer copies `docs/content`; the theme, reference,
preview and Open Graph producers read their existing content, public assets and
source trees. None reads the excluded decision directory. The dedicated type
checker custody helper separately declares four decision paths as allowed
infrastructure and binds source trees and deltas. Its own workflow filters and
custody laws are unchanged. Manual renaming scripts and implementation README
links also keep their existing decision references.

Do not broaden the exclusion to `docs/davinci/**`. Native code includes
`docs/davinci/plan/budgets.toml`, and native tests include plan schemas and metric
specifications. All those paths remain included, as do site content, theme,
preview, assets, site tasks, playground, Musea examples and the Docs workflow.

[GitHub's ordered path filters](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushpull_requestpull_request_targetpathspaths-ignore)
exclude a matching path when a negative pattern follows its positive match.
The original branch filter and GitHub diff limits still apply. This changes only
natural main push selection: every full job, assertion, nightly schedule, manual
dispatch, concurrency identity and artifact upload remains exact. Exact-head
manual PR qualification and exact-SHA release dispatch remain required.

The source review joined all 29 retained source files to actual `ceb3677d`,
classified its 16 explicit decision references and reviewed 19 changed-file
expectations. An independent search found no decision reference in the declared
Docs producer closure. Fresh implementation starts from actual `67a3c4e8`, whose
Docs workflow bytes still match that source. These are source checks, not
executed Actions selector results or public-site acceptance.

The existing workflow contract test retains every original event, immutable
SHA concurrency, job and artifact assertion while adding this one ordered path.
Fresh source Actions, required checks and actual protected merge are still
required. Do not cancel any current run or change the public floor to obtain QA.
If a future Docs producer reads a decision file, update its natural selector in
the same change.
