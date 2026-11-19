# Handoff: perf/atelier-ssr-slot-name (split from #13344; 2026-11-19)

Remove this `.handoff/` file before merging. No PR is open.

## Goal

Recover `atelier_ssr_codegen_large` (+0.10% since #13328, 802,276 on main). The cost is in `slot_entry_name`, which builds an owned joined copy (`static_slot_name`) and then quotes it.

## Current head

The head is variant s10. `slot_entry_name` moves from `component.rs` (over the 500-line limit, so it may not grow) to `slot_fn.rs` and gains a fast path: a static name without modifiers is quoted straight from the argument.

## Screens

All screens used the `perf/pb-driver` scratch workflow, with base 9c7d4cba55 (base run 44468615200).

| Variant | codegen_large | Other rows | ci-opt bytes | release bytes |
| --- | ---: | --- | ---: | ---: |
| s2: `SlotName` enum borrowed in `ComponentTemplateSlot` too (664fa93389) | −832 (−0.104%) | all identical | +256 | **+256** |
| f7: `quoted_static_slot_name` helper returning `Option<String>` (9f7ebce521) | −397 | all identical | +128 | **+128** |
| s6: fast path inline in `component.rs` (e10448f165; +8 lines, fails the length gate) | −474 | all identical | 0 | **+128** |
| s10: s6 moved to `slot_fn.rs` (899ae54213, this head) | −474 | all identical | 0 | **+128** |
| s12: `quoted_js_string` escapes in place (1e9ad5d29d) | +923 | 10 SSR rows up | −128 | +512 (rejected) |

Every shape that recovers instructions adds 49–161 B of `.text`. That rounds to +128 B or more in the stripped release binary, which the maintainer's release-size rule (must not grow) does not allow. Two ways forward:

- (a) Pair the fast path with a release-size reduction inside `vize_atelier_ssr`, e.g. drop the now-unneeded `static_slot_name` call in `slot_entry_name` by handling dotted names with a shared out-of-line joiner. Then probe release again.
- (b) Ask the maintainer whether a +128 B release cost buys −0.06% to −0.10% on one row.

## Verification done

These ran on f7, whose output equals s10's by construction:

- clippy with CI flags passes.
- The full `vize_atelier_ssr` suite passes once `vp install` provides `happy-dom`.
- The assertion lint and the source-length check pass.

## Still to do

- Byte-identity dump for the final shape.
