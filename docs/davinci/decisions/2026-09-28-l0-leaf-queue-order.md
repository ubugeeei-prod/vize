# L0 leaf queue order (2026-09-28)

## Decision

The maintainer placed [#7043](https://github.com/ubugeeei-prod/vize/pull/7043) in the merge queue at 08:29:35 UTC while [#6832](https://github.com/ubugeeei-prod/vize/issues/6832) was open. Deliver this independently validated L0 package, id, and side-table slice before #6832 closes. This is an exception for #7043 alone. The roadmap order remains the default for later slices, and #6833 and #6834 stay open until their remaining work is done.

## Merge conditions

- Refresh #7043 against current `main` and any preceding queue entries. A stale `UNMERGEABLE` entry must be removed before pushing a repaired head, then re-enqueued.
- Run the ordinary PR checks on the exact repaired head. In the protected merge queue, keep the instruction ceilings and full suites; do not raise budgets.
- Use squash merge and verify both terminal merge-queue checks and the resulting commit on fresh `main` before marking the PR delivered.

The package alias resolves to physical `vize_l0`, which currently re-exports Carton's public storage API. The independent storage carve-out remains [#6834](https://github.com/ubugeeei-prod/vize/issues/6834). The rest of the substrate move and deletion remain [#6833](https://github.com/ubugeeei-prod/vize/issues/6833).

## Product-path boundary

The #7043 diff against current `main` changes no live `vize_atelier_dom` or `vize_atelier_sfc` source. It changes native SSR slot planning in `crates/vize_atelier_ssr/src/l4/emit/slots.rs` to keep instruction counts within the existing ceilings, without switching product-path selection. The workspace `vize_l0` alias changes from Carton to the physical L0 package, but L0 immediately re-exports `vize_carton::*`, so existing compiler storage imports still use Carton's implementation. The only changed legacy parser file is a test-only `to_string()` to `to_owned()` adjustment. Davinci's `id`, `side_table`, and preserved-pass paths re-export their moved L0 definitions. This slice does not replace a compiler product's legacy execution path; [#6880](https://github.com/ubugeeei-prod/vize/issues/6880) remains a prerequisite for that later switch.

## Instruction-count recovery

The exact #7043 head `6e6570af4` exceeded three ceilings in [run 36401449923](https://github.com/ubugeeei-prod/vize/actions/runs/36401449923): DOM large by 2 instructions, SSR medium by 108, and SSR large by 4. All three repeats agreed. The same workflow on main `b4bed2e` [passed](https://github.com/ubugeeei-prod/vize/actions/runs/36401828737). Callgrind attributed the SSR medium difference to repeated growth of slot-child ranges; the other two differences appeared in string-copy library calls. Pre-size the known slot-child and captured-piece ranges so those compiler paths avoid repeated growth. Keep the budgets unchanged and require a new exact-head measurement before re-entering the queue.

The first allocation attempt `21095ad7` [failed](https://github.com/ubugeeei-prod/vize/actions/runs/36403094200) different ceilings (DOM medium +110, SSR large +173, L1-to-L2 DOM surface +7): eager default-slot allocation also ran for named-only content, and a capture-bucket reserve affected unrelated cases. Remove that reserve and allocate default-slot capacity only when a default child exists, capped at eight ranges. Keep exact capacity for an explicit own slot. Measure again before queueing.

The narrower `226fcb48` attempt [still failed](https://github.com/ubugeeei-prod/vize/actions/runs/36403819571): DOM large +2, SSR medium +677, SSR large +151. Revert the conditional default-slot reserve. The emitter was also cloning every default range list to build a temporary slot description, including the dynamic-slot path; pass the borrowed ranges and metadata directly to emission instead. This removes that copy without changing the emitted program. Re-measure the exact head before queueing.

Borrowing default ranges at `52d0d601` [failed](https://github.com/ubugeeei-prod/vize/actions/runs/36404944603) five ceilings, and after synchronizing current `main` at `0e14eebc` [failed](https://github.com/ubugeeei-prod/vize/actions/runs/36405572546) four SSR ceilings. The wrapper also grew `slots.rs` past the 350-line source ratchet. Revert all three allocation experiments to the original emitter and measure the L0 move against current `main`. The original same-base misses were small (DOM large +2, SSR medium +108, SSR large +4), and current `main` includes the merged #7044 optimization; no budget is raised.

The reverted emitter at `69c875fa` [failed](https://github.com/ubugeeei-prod/vize/actions/runs/36405984390) only SSR medium +108 and SSR large +4; DOM large passed. Callgrind attributes medium's excess to nine default-range `Vec` growth events, while large's four extra instructions are in `memcpy`. Reserve default ranges only for short component child lists (two to four) whose first child is not a named slot carrier. This targets ordinary default content without allocating for the large named-slot groups. Keep `slots.rs` below the 350-line ratchet and remeasure exact head.

The two-to-four-child candidate `0e9f1180` [failed](https://github.com/ubugeeei-prod/vize/actions/runs/36406622755) SSR medium +200 and large +1. Callgrind shows it avoided only one of medium's nine vector growths; the other eight ordinary components have one child. Include one-child components in the reservation while retaining the named-slot guard, then measure again.

The one-to-four-child candidate `9404e6c1` [missed](https://github.com/ubugeeei-prod/vize/actions/runs/36407144299) only SSR large by five instructions, while SSR medium measured 192 below its ceiling. Replace the inclusive-range membership check in the hot slot classifier with direct integer comparisons and remeasure; keep the reservation rule and budgets unchanged.

Direct comparisons at `9512cedd` [still missed](https://github.com/ubugeeei-prod/vize/actions/runs/36407708076) SSR large by five instructions. Ordinary Clippy rejected its guarded `children[0]` access under `indexing_slicing`. Use `children.first()` to make the nonempty proof explicit; remeasure after the required safety fix.

The large fixture has many named and dynamic slot carriers. `component_slots` computed `child_end` before classifying every child, although only an ordinary default child uses that scan. Move `child_end` into the default branch. This removes work from the large named-slot path without changing accepted input or output; retain the passing medium reservation and verify both CI and the strict instruction counts.

Head `2513d2fa` passed [exact-head instruction counts](https://github.com/ubugeeei-prod/vize/actions/runs/36410624034) and [ordinary Check](https://github.com/ubugeeei-prod/vize/actions/runs/36410624165). Its first queue entry at position six was `UNMERGEABLE` only because `Cargo.lock` conflicted with the preceding virtual group head after #7107. Remove the queue entry so it cannot block later PRs; synchronize the lockfile after the preceding entries merge or leave, repeat exact-head validation, and requeue.
