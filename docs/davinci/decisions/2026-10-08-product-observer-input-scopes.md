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

## Full JS qualification before further Docs source routing

The targeted raw-log audit did not reproduce a UI hang. Four natural source
runs retain the complete ordinary 16-package tests and original-project
transport before the UI step:

| PR    | Exact Check run                                                               | Ordinary package tests | Complete transport | UI step |
| ----- | ----------------------------------------------------------------------------- | ---------------------- | ------------------ | ------- |
| #8295 | [37749772565](https://github.com/ubugeeei-prod/vize/actions/runs/37749772565) | 274.451 s              | 849.670 s          | 144 s   |
| #8296 | [37750044241](https://github.com/ubugeeei-prod/vize/actions/runs/37750044241) | 280.089 s              | 819.603 s          | 140 s   |
| #8292 | [37750746719](https://github.com/ubugeeei-prod/vize/actions/runs/37750746719) | 273.862 s              | 814.351 s          | 139 s   |
| #8300 | [37751803829](https://github.com/ubugeeei-prod/vize/actions/runs/37751803829) | 271.797 s              | 804.897 s          | 140 s   |

The complete campaign is necessary qualification. The native licensed replay,
parent reference, both pinned real Oxlint hosts, all original rules/options and
physical source custody remain intact. Ordinary package tests do not duplicate
that campaign. The debug/history addon and the later CI/frontend addon have
different recipes and cannot substitute for one another.

The existing main-push Check omitted both full JS jobs and full tooling; its
manual/scheduled package test omitted the transport campaign. Before a further
Docs source split, main pushes must run the existing full `test-js-packages`,
`build-js-packages` and `test-scripts` jobs. Scheduled/manual runs retain those
same jobs. The package test calls the unchanged public-history composite;
the separate build job retains Fresco declarations/consumer types, its own
native `build:ci`, UI, compose conformance, package builds and shared build
archive. Main also runs the existing full `check:ci` path. No build artifact
download edge replaces a physical source producer.

The separate `full-js-report` strictly requires those three jobs plus
`check-js`; failure, cancellation, skips and missing results cannot pass.
The existing source/PR rollups and their no-skipped-job requirement remain
unchanged. Both authoritative Rust and compatibility JS release inventories
require `test-js-packages`, `build-js-packages` and `full-js-report`, retaining
`test-scripts` and all 12 SemVer jobs. Old source/main green supplies no
transfer to a release source without these genuine workflow/tool bytes and
exact-source successful job evidence.

TODO: after the native TypeScript planner foundation and actual typed Docs
graph compose, review a literal site-Docs script/theme/config/build-workflow
input inventory and a finite offline source cohort. New unreviewed executable
paths, unknown workflows, mixed product/fixture changes, `.gitattributes`, and
Rust-consumed plan documents must retain conservative routing. The original
full tooling, native retirement law, browser/build qualifications and release
oracles remain mandatory in their full contexts. No broad unscoped-test
pruning or oracle, host, corpus, timeout or budget relaxation is authorized.

After genuine replay on #8306 `23dc3c032a5a1629ac3efd0c034aff9300f3228e`,
the new full report consumes its actual erasable TypeScript gate. Its pinned
Node setup precedes that new shell caller and installs no report dependencies.
The focused full-job/release-inventory and adjacent native-TS gate controls
pass 17/17 on both Node 22.18.0 and 24.14.0; all three existing bounded strict
native compiler projects also pass unchanged. Actual Rust verify-only refusal
controls and exact-head/full/protected/actual delivery remain pending Actions.
No two-minute claim or #6830 closure follows from a
local runtime, selection count or fast sub-step. Natural source Actions must
measure the resulting gate and the whole Check, including residual parallel
jobs. The public audit and decision are paired on
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6053136181).

## Original bug closure and publication

The maintainer pointed out that open Issues and unpublished changes were not
being completed. Original report acceptance and release publication are distinct
outcomes. Preserve the original failing reproductions, passing regressions and
protected merge gates. Do not silently add supplementary agent investigations to
the acceptance of an already corrected bug.

On 2026-10-08, close these six original reports as fixed on main:

| Report                                       | Delivered fix   | Actual protected Check |
| -------------------------------------------- | --------------- | ---------------------- |
| #7924 directive formatting drift             | #8109, ae79a5ad | 37428993702            |
| #7871 preserved template text                | #8174, fe94b4d0 | 37674801258            |
| #7929 unary expression comment               | #8193, 102535bc | 37656842645            |
| #7868 typed arrow hugging                    | #8199, 9d87650e | 37656845042            |
| #8010 library-safe event rename              | #8262, f20f9d9c | 37758274405            |
| #8011 complete event/slot/alias transactions | #8262, f20f9d9c | 37758274405            |

The closing comments link each actual fix and successful protected Check. The
LSP receipts preserve every original event and slot endpoint under LF/CRLF,
26 alias transactions and six definition sessions, all 114 retained transactions
and the 22 library-authority transactions. Formatter receipts preserve the
reported whole outputs, repeated passes and runtime text semantics.

These reports did not require a registry-installed custody campaign. Additional
installed ownership, package-adaptation and kernel-trace work remains unfinished
supplementary work. This decision grants no public package or release credit.
The latest verified public version is 0.435.0; the next official minor release
must still pass its existing source, native, semver, release and publication gates
under #6239. Actual new release-gate failures remain blockers. A failure observed
only in another PR does not create a historical-cause prerequisite for starting a
fresh officially gated candidate.

### Exact source failure and bounded correction

The first parity source `b6d2e15e046f7f51e9cf0f78e19e8f726e99a086`
[Check 37763878522](https://github.com/ubugeeei-prod/vize/actions/runs/37763878522)
authentically failed tooling worker 2/4. Its old workflow law still required
inline `test:js` after the existing unchanged history composite became the
caller, and the original growth law rejected `check.yml` (708 versus 690)
and the release runner test (476 versus 423). Those failures grant no retry
or predecessor-success credit. Worker 3 genuinely ran and passed the actual
Rust verify-only callback covering all three new jobs and six refusal states.

Extract that exact callback into a bounded test file, restoring the original
runner to 423 lines. Move the existing JS workflow law to a bounded file and
update it to require the real composite call and its unchanged package-test
command/cache key. Extract only the full package job's existing six setup
steps and the full report's pinned Node setup plus strict TypeScript caller
into bounded composites, restoring `check.yml` to 690 lines. Checkout stays
in each physical job; pins, cache keys, install command, full-event predicates,
four report dependencies and the unchanged history action remain intact.
The complete needs object is passed through an action input and environment,
never interpolated into shell source. No required gate or length ceiling is
relaxed. Fresh successor Actions and actual protected delivery are required.

Further Docs source-routing work is frozen by the maintainer's priority shift
to closing original bugs and frequent successful publication. The design TODO
above remains unfinished; this correction activates no new selector.

The corrected workflow/release controls pass 25/25 on actual Node 22.18.0
and 24.14.0; all three existing bounded native TypeScript projects pass.
Expanding both composites reproduces the whole previous Check YAML object
exactly, with only required explicit composite shells and the equivalent
needs-input/environment binding normalized. The original growth exclusions
are unchanged, and every changed/new included file retains its original cap.
Root's closure update changes only canonical rows 187 and 286; the other
348 rows remain exact. These local checks do not qualify the new source head.

The corrected `ccfd3f69cf05c36377de7e2bf6cd7924c8ea2695` source exposed
one authentic Advanced Security refusal in the new package-setup composite:
the copied old dtolnay stable pin resolves but no longer belongs to reachable
upstream history. Replace only that new action pin with the repository's
existing current stable `89b12181fb390509a0842a86cc55eeb8eb928c1d`.
Read-only official action comparison retains the default stable toolchain and
install options, with upstream checksum-update retry and non-host handling.
No upstream write, version promotion, gate or budget change is made. The
whole-expansion proof above remains historical for the prior pin; the final
source differs only by this explicit reachable-pin correction and still
requires fresh exact-head Actions.

Fresh `b63908e5e57351c54d053bb625f1a71381e413dd` passes its actual security
check and executes all 18 Rust refusal cases, but authentically fails the
release input-scope catalogue's old physical-file count. The exact callback
move adds one release test: 32 physical files, the original 27 audited scoped
files, and five conservative unscoped files. Update only that exact count and
literal unscoped filename witness. Retain all four older unscoped entries,
complete import assertions, source-input selection and full merge selection.
No selector implementation or required scope is changed; the new callback
stays broad. The raw first failure remains evidence, and fresh corrected-head
Actions are still required.
The unchanged four release-scope controls now pass on Node 22.18.0 and
24.14.0, including full release and merge selection. The live provider is
still `23dc3c032a5a1629ac3efd0c034aff9300f3228e`; the observed source checkout
merged with the planner's fresh main comparison cut, which grants no reason
to rebase onto a computed merge commit or transfer source success.
