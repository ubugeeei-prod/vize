# Deterministic webcam countdown test clock

The protected Check for #8121 failed the existing countdown case: expected `Taking photo in 1`, observed `Photo taken`. The complete raw failure is retained locally (SHA256 `4a354e68d12338a4e9ba8a2e76882e50b535fb48464ec47c0da50e7127a3a198`); the other 493 files and 3807 tests passed. This establishes a test scheduling defect, not a production webcam regression.

Issue [#8138](https://github.com/ubugeeei-prod/vize/issues/8138) and the [paired decision](https://github.com/ubugeeei-prod/vize/issues/8138#issuecomment-6019906810) record the repair. Two sequential 5ms component timers cannot be sampled reliably by sleeping 8ms on a loaded runner. Move only that complete case and the unchanged mounting fixture into a bounded dedicated test module in a move-only commit. All remaining original test bytes and every production file remain unchanged.

Use the existing Vite+ `vi` clock after real camera initialization. Advance each original 5ms countdown tick independently, retain all original assertions, explicitly assert intermediate `1` with no capture and the complete `Photo taken` transition, and verify stop cancels the next timer. The existing async flush is driven under the same fake clock; `finally` restores real timers before the original component/canvas/URL cleanup.

Validation uses the existing selected UI checks and whole hosted UI suites. No dependency, production source, timeout, golden, instruction ceiling, test input or default policy changes. Repaired-source execution, required Actions, genuine composition of the independent security fix, protected signed merge and release inclusion are pending.

## Actual security-main replay

The [paired replay decision](https://github.com/ubugeeei-prod/vize/issues/8138#issuecomment-6020554308) binds actual signed security main `ef84821d30fa0d8538b2472fef34418e75380523`. Replay the original two commits once, retaining the move-only history, complete fixture, all seven original assertions, both 5ms ticks and clock cleanup. No production or input changes.

Previous source `b483839efbc5a413d8e9d7fe06eddc25bf61b015` passed both whole hosted UI suites (495 files and 3808 tests each), including this exact countdown case. Complete raw stream SHA256 `e43fb4631d9f044492be45889f3b3adde6b24103d956863df162cddd2570a225` and exact-tree proof SHA256 `433f042621205ba7a2c50edc5ead91581e71c3a606b12036b91ff97da0ad4a3c` remain immutable historical evidence. Fresh replay-source Actions and protected signed delivery are required; installed/public acceptance is unclaimed.
