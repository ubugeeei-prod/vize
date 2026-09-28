# L1 tokenizer source move: protected instruction gate (#6835)

## Decision and current state

Keep [#7136](https://github.com/ubugeeei-prod/vize/pull/7136) out of the
protected merge queue until its updated exact head passes ordinary Actions.
The first merge group failed immutable instruction ceilings; a source-level
candidate now passes all 100 on the #7150 main base, but it has not yet passed
the PR and cumulative merge-group gates. Do not raise budgets, change the
measurement method or relax the source-length ratchet. The existing compiler
parser stays on its original product path; #6880 still gates a route change.
The agreed L1 ownership and Armature dependency direction remain the target,
and #6835 remains open until the actual merge and remaining parity work.

## Measurements

All instruction runs measured the same 100 pinned probes three times on an
exact commit. The initial protected merge group used `201ef776` on
`ac01d873`; the final package experiment rebased #7136's commits on fresh
`a4d89982`. Counts below are probes above the unchanged ceilings, not a
comparison against a fresh-main control measurement.

| Candidate                                                                                   | Exact Actions run                                                             | Above ceiling | Result                                                                                                   |
| ------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | ------------: | -------------------------------------------------------------------------------------------------------- |
| #7136 L1 re-export and dependency inversion                                                 | [36438442532](https://github.com/ubugeeei-prod/vize/actions/runs/36438442532) |            30 | Protected queue refused the merge.                                                                       |
| MathML lookup shortcut                                                                      | [36441565455](https://github.com/ubugeeei-prod/vize/actions/runs/36441565455) |            27 | Backend codegen shifts remained.                                                                         |
| Armature compile host using sibling source path                                             | [36442556112](https://github.com/ubugeeei-prod/vize/actions/runs/36442556112) |            17 | Restored prebuilt-AST codegen, but sibling path is absent from a published Armature package.             |
| Compile host plus MathML shortcut                                                           | [36443453328](https://github.com/ubugeeei-prod/vize/actions/runs/36443453328) |            27 | Gains did not combine.                                                                                   |
| Packageable Armature source snapshot, optional L1 edge                                      | [36445737713](https://github.com/ubugeeei-prod/vize/actions/runs/36445737713) |            17 | Local package, byte parity, tests and Clippy passed; fused compiler probes still failed.                 |
| Small dynamic-prop deduplication without a hash table                                       | [36448073726](https://github.com/ubugeeei-prod/vize/actions/runs/36448073726) |            25 | Output parity passed, but fused compile misses remained.                                                 |
| Common HTML tags before the PHF lookup                                                      | [36448889203](https://github.com/ubugeeei-prod/vize/actions/runs/36448889203) |            23 | Fused compile improved, but independent codegen and generate probes exceeded ceilings.                   |
| Common HTML tags with an out-of-line helper on then-fresh `371489b9`                        | [36449943926](https://github.com/ubugeeei-prod/vize/actions/runs/36449943926) |            10 | All fused compile probes passed; six DOM codegen and four Vapor generate probes still exceeded ceilings. |
| Inline generated-name sets and small prop deduplication                                     | [36451342550](https://github.com/ubugeeei-prod/vize/actions/runs/36451342550) |             2 | Only Vapor generate stress-wide +28 and stress-interp +44 remained.                                      |
| Lazy `Option<NameSet>` plus bounded large-prop capacity                                     | [36452840301](https://github.com/ubugeeei-prod/vize/actions/runs/36452840301) |             4 | New Vapor lower misses appeared.                                                                         |
| Lazy `Option<NameSet>` alone                                                                | [36453809416](https://github.com/ubugeeei-prod/vize/actions/runs/36453809416) |             4 | The same four misses proved the lazy layout caused the regression.                                       |
| Best inline set plus bounded capacity, rebased after #7147 merge                            | [36457549037](https://github.com/ubugeeei-prod/vize/actions/runs/36457549037) |             2 | The same exact wide +28 and interp +44 Vapor generate ceilings remained.                                 |
| Build fallback spans only when a source map is requested, on #7147 main                     | [36459529667](https://github.com/ubugeeei-prod/vize/actions/runs/36459529667) |             0 | All 100 ceilings hold; three executions match exactly. PR and merge-group verification remain.           |
| Same cumulative source after #7132 merge                                                    | [36461474662](https://github.com/ubugeeei-prod/vize/actions/runs/36461474662) |             4 | Vapor lower small, deep, interp and generate interp exceeded their ceilings.                             |
| Cache computed-prop mode once per element and skip empty delegate append, after #7150 merge | [36469060988](https://github.com/ubugeeei-prod/vize/actions/runs/36469060988) |             1 | Only Vapor lower stress-deep remained +33; #7150 did not alter that result.                              |
| Skip computed-prop scan for one static attribute, after #7150 merge                         | [36469732014](https://github.com/ubugeeei-prod/vize/actions/runs/36469732014) |             0 | All 100 ceilings hold in three exact repeats; ordinary PR and merge-group verification remain.           |

The packageable snapshot failed DOM compile in five fixtures and SSR/Vapor compile
in six each, up to 13,837 instructions above a pinned ceiling. It matched the
earlier compile-host experiment in 98 of 100 measurements. Callgrind showed
unchanged exclusive tokenizer and L1 Recorder counts in the first queue run;
PHF/SipHash generic code and, in the original re-export variant, prebuilt-AST
backend codegen changed with the ThinLTO layout. The packageable snapshot
removed the prebuilt-AST misses but not the 17 fused compiler misses.

The packageable snapshot also fails the unchanged source-length ratchet: Git
sees the 901-line `states.rs` and 826-line `tests.rs` in L1 as new copies once
the old Armature paths are restored. No copy exception was added. Splitting
both source trees symmetrically into files under 350 lines would be necessary
if a later, measured variant justifies keeping the snapshot; it would need
another full instruction run because source layout can change code generation.

The two subsequent source experiments stayed off #7136. Dynamic-prop
deduplication preserved output and improved five ceilings, but did not address
the 17 fused compiler misses. Direct matches for common HTML tags preserved
the exported PHF set and greatly reduced fused compile instructions (DOM
stress-interp by 69,436), yet DOM codegen stress-deep rose 47,654 and Vapor
generate stress-interp rose 22,294. Neither passes the global gate. These
cross-stage movements show that local hot-path savings alone are insufficient
under the current ThinLTO layout.

Callgrind attributed the HTML fast path's prebuilt-AST codegen change to
`CompactString::Repr::push_str` becoming out of line more often even though
the tag helper was never called. The bounded out-of-line helper experiment on
then-fresh `main` eliminated every fused compile miss, but DOM codegen still
exceeded ceilings by 3–1,194 instructions and Vapor generate by 6–132. Three
repeats agreed. It was kept off #7136; the global result is still red.

The later inline generated-name-set implementation removed eight of those ten
misses while preserving output parity. Lazily wrapping the sets in `Option`
made Vapor lower regress in three fixtures, independently of the bounded
large-prop capacity; do not include that layout. The best source candidate
with the bounded 128-entry initial capacity was rebased on #7147's actual
merge commit `b801e023` and missed exactly two Vapor generate ceilings:
stress-wide 710,672 > 710,644 and stress-interp 1,856,542 > 1,856,498.
Callgrind attributed the residual to generation entry/drop and empty source
span storage. `generate_vapor_with_spans` constructed four empty maps even
when neither authored spans nor a requested source map could use them. Make
the fallback only when the map is requested and no spans were supplied. The
source-map suite passed 18/18 and Vapor artifact check 1/1; exact diagnostic
Actions then measured 100/100 on `b801e023`, with three identical executions.
After #7132 merged, that cumulative source missed four ceilings. A separate
Vapor-only precursor on main missed four ceilings as a pair, one as an inline
wrapper alone, and three as lazy spans alone; it was not made a PR. The best
cumulative source after #7150 merged missed only Vapor lower stress-deep by
33 instructions. That fixture has 48 elements with exactly one authored
static attribute. `uses_computed_props` scanned that sole attribute even
though no computed `v-bind` can exist. An exact singleton-attribute early
return preserved all 266 Vapor library tests, the compiler fix-history and
artifact tests, and strict Clippy; the cumulative Actions run then passed
100/100 with three identical executions. This is evidence for updating the
existing PR head, not for bypassing its ordinary or protected queue gates.

## Next bounded work

Update #7136 with the original move-only commit intact and each measured
source optimization in its own commit. Run exact-head ordinary Actions. The
independent #7152 queue entry must finish first; then rebase once on fresh
main, remeasure all 100 and use native Stack prefix merge through the protected
queue. Verify actual `mergedAt` and fresh main before restacking children. If
the cumulative gate changes the result, record the exact failed probes before
another decision. Keep byte/error-recovery
parity, Cargo package checks and the unchanged source-length ratchet. Do not
mark #6835 complete until the shared production lexer and remaining dependency
work land; #6880 still gates compiler route replacement.
