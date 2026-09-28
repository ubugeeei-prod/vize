# L1 tokenizer source move: protected instruction gate (#6835)

## Decision and current state

Keep [#7136](https://github.com/ubugeeei-prod/vize/pull/7136) open and out of
the protected merge queue. Its ordinary PR Actions pass, but its merge group
failed the immutable instruction ceilings. Do not raise budgets, change the
measurement method or relax the source-length ratchet. The existing compiler
parser stays on its original product path; #6880 still gates a route change.
The agreed L1 ownership and Armature dependency direction remain the target,
and #6835 remains open. Independent Davinci PRs can merge while this move is
investigated.

## Measurements

All instruction runs measured the same 100 pinned probes three times on an
exact commit. The initial protected merge group used `201ef776` on
`ac01d873`; the final package experiment rebased #7136's commits on fresh
`a4d89982`. Counts below are probes above the unchanged ceilings, not a
comparison against a fresh-main control measurement.

| Candidate | Exact Actions run | Above ceiling | Result |
| --- | --- | ---: | --- |
| #7136 L1 re-export and dependency inversion | [36438442532](https://github.com/ubugeeei-prod/vize/actions/runs/36438442532) | 30 | Protected queue refused the merge. |
| MathML lookup shortcut | [36441565455](https://github.com/ubugeeei-prod/vize/actions/runs/36441565455) | 27 | Backend codegen shifts remained. |
| Armature compile host using sibling source path | [36442556112](https://github.com/ubugeeei-prod/vize/actions/runs/36442556112) | 17 | Restored prebuilt-AST codegen, but sibling path is absent from a published Armature package. |
| Compile host plus MathML shortcut | [36443453328](https://github.com/ubugeeei-prod/vize/actions/runs/36443453328) | 27 | Gains did not combine. |
| Packageable Armature source snapshot, optional L1 edge | [36445737713](https://github.com/ubugeeei-prod/vize/actions/runs/36445737713) | 17 | Local package, byte parity, tests and Clippy passed; fused compiler probes still failed. |
| Small dynamic-prop deduplication without a hash table | [36448073726](https://github.com/ubugeeei-prod/vize/actions/runs/36448073726) | 25 | Output parity passed, but fused compile misses remained. |
| Common HTML tags before the PHF lookup | [36448889203](https://github.com/ubugeeei-prod/vize/actions/runs/36448889203) | 23 | Fused compile improved, but independent codegen and generate probes exceeded ceilings. |
| Common HTML tags with an out-of-line helper on then-fresh `371489b9` | [36449943926](https://github.com/ubugeeei-prod/vize/actions/runs/36449943926) | 10 | All fused compile probes passed; six DOM codegen and four Vapor generate probes still exceeded ceilings. |

The final experiment failed DOM compile in five fixtures and SSR/Vapor compile
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

## Next bounded work

The bounded PHF/SipHash and out-of-line experiments did not pass. Stop
variants. Keep #7136 dequeued while independent Davinci changes merge, and
bring the measured tradeoff to the maintainer: defer the source move, or
authorize a separately reviewed codegen/generate optimization tied to the ten
remaining probes. Either path keeps immutable ceilings, byte/error-recovery
parity and the Cargo package check. Only after an exact-head candidate
passes should #7136 and its native Stack descendants be rebased and sent
through PR Actions and the protected queue. Do not merge a temporary duplicate
production tokenizer or mark #6835 complete.
