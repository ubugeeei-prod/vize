# Retain whole product observers for their actual inputs

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The documentation-only source checks for
[#8263](https://github.com/ubugeeei-prod/vize/actions/runs/37729733431) and
[#8265](https://github.com/ubugeeei-prod/vize/actions/runs/37729988551) correctly
skip Rust, JS packages and the canonical corpus. Their independent tooling
selector nevertheless selects 752 of 909 files: 718 retain conservative broad
inputs, 34 match documentation contracts, and 112 runtime files are deferred
to the full merge suite. The four workers each prepare the current CLI/oracle
in about 70–72 seconds and the current native addon in about 45–46 seconds.
The complete source workflows take about seven to eight minutes.

Two selected whole product observers do not consume release prose or decision
records. The Art public-boundary observer takes 41.018 seconds on worker 1; the
original #7893 DOM/SSR/Vapor CLI and Vue-runtime observer takes 10.182 seconds
on worker 4. Their owned input contracts remain source-built CLI/NAPI receipts,
literal original fixtures, physical source-addon custody, current package
builds, pinned host/runtime dependencies and every complete diagnostic/runtime
comparison. Their bodies, goldens, source authority and outputs stay intact.

Only these two observers receive the audited product input scope. It retains
every existing source, fixture, test, helper, tool, editor, example, playground,
workflow, build configuration, manifest and lockfile input. Plan documents
remain inputs because product Rust code embeds complexity and budget tables.
The selector continues adding complete recursive literal local imports; an
incomplete import closure falls back to the original broad inputs. Manually
spawned runtime helpers and preloaded custody modules are covered by the
retained `tests/**` inputs. Unknown paths and workspace task changes continue
to select the broad PR suite.

All other unscoped files keep their conservative inputs. The full merge tier
unconditionally retains every discovered test, including both original
observers. No source check, corpus vector, required gate, process timeout or
resource budget is removed or relaxed. This small dependency correction does
not complete #6830 or claim the two-minute PR goal; fresh exact-head Actions
and protected delivery are still required.

The separate source PR
[#8255](https://github.com/ubugeeei-prod/vize/actions/runs/37725186524) spends
about 284 seconds in ordinary JS tests and 828 seconds in the original-project
Oxlint transport qualification. The latter includes distinct complete native,
parent-reference and current dual-host n8n replays. The ordinary package test
list does not repeat the licensed campaign. These phases retain their source
custody and whole vectors; this change does not remove them.
