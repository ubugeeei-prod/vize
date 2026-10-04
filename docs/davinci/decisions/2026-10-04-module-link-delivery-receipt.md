# Module-link source and delivery qualification (2026-10-04)

Tracking: [#6871](https://github.com/ubugeeei-prod/vize/issues/6871), [#6883](https://github.com/ubugeeei-prod/vize/issues/6883); native Stack [#7783](https://github.com/ubugeeei-prod/vize/pull/7771). This supplements the [context decision](./2026-10-04-module-link-context.md) with actual source evidence and delivery limits.

## Original context source

Frozen parent source `90fcfb246bcbc514916ae8270148e0b7caa133a3` has genuine released-main `caee1656927e1232f4aaabff2328246decb8bc58` ancestry. [Check 37191134372](https://github.com/ubugeeei-prod/vize/actions/runs/37191134372) is terminal SUCCESS. The actual Builder checkout `bd29be175e6892a78849136a2c3006cbedde6c21` has those two inputs and tree `89455c6c4d8279dd08cb6ecf87c1ef5f06c50385`, exactly the frozen source tree. Incoming released versions and all 33 owned implementation/test/workflow paths stay byte-exact against the previous reviewed context source.

Raw [Builder 111403449194](https://github.com/ubugeeei-prod/vize/actions/runs/37191134372/job/111403449194) executes 152 SourceProject, 43 RPC, 21 normal module-link and 11 genuine no-default module-link cases, all passing. The module-link profiles execute 32 cases representing 22 distinct controls with ten shared names. Genuine `--no-default-features --features experimental-source-navigation` check and feature-enabled all-targets Clippy with `-D warnings` both finish successfully; the latter retains the normal feature profile. This does not claim no-default Clippy execution.

The four actual default Rust JUnit ZIPs were downloaded, matched to their Actions SHA-256 digests and checked for valid ZIP CRCs:

| Shard | Artifact ID | Cases |
| ----- | ----------- | ----- |
| 1     | 11298604007 | 3,933 |
| 2     | 11299456271 | 3,833 |
| 3     | 11298613882 | 3,781 |
| 4     | 11299182929 | 3,994 |

Those archives contain 15,541 cases with zero failures, errors or skips. Separate existing Builder doctests include ignored cases; the archive count does not describe every test command. The large compiled-binary archive was not independently downloaded. The [public source audit](https://github.com/ubugeeei-prod/vize/pull/7771#issuecomment-5978243388) retains exact checkout and artifact provenance.

## Original physical target source

Frozen child source `2d3e6dc21b45a13756ad0cbf4da81934c65938bf` retains the genuine parent `90fc` and released-main `caee` ancestry. [Check 37191135172](https://github.com/ubugeeei-prod/vize/actions/runs/37191135172) is terminal SUCCESS. Actual Builder checkout `c4135d8caab8d3fd7795eb11da25d102ad55dc50` combines the child with the projected parent `bd29`; its tree `11bd3a795c3a916325b3b17a6a22c1953d6bee7a` exactly equals the frozen child source. All 20 owned non-doc implementation/test/manifest paths remain byte-exact against the previously reviewed physical source; the lockfile preserves the released workspace versions and existing Unix libc pin.

Raw [Builder 111403447455](https://github.com/ubugeeei-prod/vize/actions/runs/37191135172/job/111403447455) passes 164 SourceProject, 43 RPC, 50 normal and 23 genuine no-default module-link cases, plus minimal check and feature-enabled strict all-targets Clippy. The new physical controls execute 29 normal and 12 minimal cases: 41 profile executions, 30 distinct Linux controls and eleven shared names. The original parent controls remain in both profiles. The separate non-Unix refusal law is unexecuted in these Linux results.

The child independently authenticates its own four default Rust JUnit ZIPs by Actions SHA-256 and ZIP CRC, with the same 15,541 aggregate and zero failures, errors or skips:

| Shard | Artifact ID | Cases |
| ----- | ----------- | ----- |
| 1     | 11298868354 | 3,933 |
| 2     | 11299580961 | 3,833 |
| 3     | 11299157895 | 3,781 |
| 4     | 11298673788 | 3,994 |

These are source archives, not transferred parent/predecessor results or compiled-binary execution proof. Source native CLI phase run [37191134795](https://github.com/ubugeeei-prod/vize/actions/runs/37191134795) has terminal SUCCESS metadata; its full raw campaign artifacts are not independently qualified here.

The [physical decision](./2026-10-04-module-link-target-observation.md) grants an explicit point observation using retained original root/per-component/target handles, same-server event epoch and original query snapshot/cancellation owner. Fresh recheck retains earliest source/context/event refusals before IO outcomes, then whole-batch publication refuses without a partial prefix. Caller `./` candidates grant physical requests only, not original quoted module literal/span/Occurrence authority. Unknown watcher coverage is explicit; requesting a stronger coverage grant refuses. Minimal builds refuse NativeUnavailable and non-Unix builds refuse UnsupportedPlatform without a legacy resolver shortcut.

"Bounded" describes the explicit path domain and point-in-time contract, not numeric resource ceilings. There is no admitted batch-count, path-depth or byte cap; actual work and retained handles scale with the sum of traversed path depths. A path can change after successful physical observation. A future consumer still needs its original literal/Occurrence provider, resource-admission policy and genuine watcher/default/edit/full-packet integration, with real semantic and performance gates.

## Actual signed delivery

Both native Stack layers actually merged at `2026-10-04T09:53:06Z`; each actual merge is the exact protected candidate with a valid GitHub signature:

| PR    | Actual signed merge / parent                                                                                   | Exact protected Check                                                                 |
| ----- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| #7771 | [`50c944da`](https://github.com/ubugeeei-prod/vize/commit/50c944da9775380afc8c245c02a21765b076788b) / `caee`   | [37192659119](https://github.com/ubugeeei-prod/vize/actions/runs/37192659119) SUCCESS |
| #7782 | [`69cab7b3`](https://github.com/ubugeeei-prod/vize/commit/69cab7b3436139c1e4717da0c3aef4844d21141e) / `50c944` | [37192659855](https://github.com/ubugeeei-prod/vize/actions/runs/37192659855) SUCCESS |

Each complete actual merge tree equals its frozen source tree above. Both exact candidate Musea/Nuxt runs also pass. Each Check has 39 terminal jobs: 24 success and 15 alternate-policy skips, with all four actual full Rust shards successful. Eight independently downloaded candidate JUnit ZIPs match their Actions SHA-256 and ZIP CRC: each candidate contains **15,559** cases (shards 3,938 / 3,839 / 3,786 / 3,996), zero failures/errors/skips in those archives. This is distinct from each source's 15,541-case archive receipt. Separate existing doctest ignores and undownloaded compiled binaries remain outside the archive claim.

Raw candidate Builder executes parent 152 / 43 / 21 / 11 and child 164 / 43 / 50 / 23 again, with genuine no-default check and normal-feature strict all-targets Clippy. Child profiles retain 52 distinct module controls with 21 shared names. Each authenticated performance ZIP pair reconciles unchanged **100 stage + 4 formatter ceilings × 3 identical executions**, all 312 named raw Ir windows, 39 zero termination windows, exact authored fixture/window identities and matching methodology. Separate CLI phase evidence is not included in that 104 count; no 10x or physical-provider resource bound is inferred.

Observed main `77a8c401fa1ca72464de7769dffc41862d05c089` is a genuine descendant of both actual merges. It retains 54 of the 55 union-owned blobs byte-exact; the shared canonical record preserves the complete child Maestro paragraph alongside incoming formatter decisions. Final queue/auto-merge entries are null. Qualified public delivery audits: [parent](https://github.com/ubugeeei-prod/vize/pull/7771#issuecomment-5978786408), [child](https://github.com/ubugeeei-prod/vize/pull/7782#issuecomment-5978786608).

Both historical signed merge bodies omit the requested literal reporter footer; the frozen source commits and descriptions supplied none. This establishes unsupplied credit, not GitHub normalization. This separate meaningful documentation change supplies verified reporter credit prospectively and does not rewrite those sources, merge bodies or repository settings. Its own eventual signed footer must still be read back; same-primary normalization, if observed, leaves literal-footer retention unmet.

## Remaining product and successor gates

The parent context provider owns actual applied root/checker/load-origin, same-server checked generation and original-result-host source-to-context publication. Foreground/input termination, equal foreign owners, cancellation, ABA, overflow and unwind remain real refusal controls. Its context ticket alone grants no original module literal/Occurrence, physical target, edit or native DocumentLinks packet authority. Existing legacy request behavior stays separate; this is not a default product replacement.

Fresh successor Actions and each changed protected candidate's full Rust suites and unchanged 100 stage plus four formatter instruction measurements remain required; dependent code successors must retain genuine native Stack membership/order. Historical source or accepted predecessor results cannot accept a refreshed candidate. Actual signed merge, source-owned blob custody and final commit footer must be read back before reporting the successor delivered. Release publication and its explicit admission thaw are separate gates.

[#6883](https://github.com/ubugeeei-prod/vize/issues/6883) remains OPEN: this bounded provider does not complete full fix-history replay, native edit/default LSP, all Vue/JS/TS/JSX/TSX consumers or the typechecker 10x target. The canonical and existing context records remain within their unchanged 350-line limits; this companion adds no production code, pipeline stage, serialization or performance-budget change.

The linked reporter is the same primary author, `ubugeeei`, with verified public identity `71201308+ubugeeei@users.noreply.github.com`. The frozen source commits and PR description did not supply a literal `Co-authored-by` trailer. The meaningful receipt commit supplies that identity without rewriting source history or inventing a distinct reporter. A later signed squash may normalize same-primary credit; only actual commit readback can establish whether the requested literal footer survived.
