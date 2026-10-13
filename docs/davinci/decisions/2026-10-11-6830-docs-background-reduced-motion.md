# Docs background honors reduced motion

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The completed [Docs run 38127463921](https://github.com/ubugeeei-prod/vize/actions/runs/38127463921)
used `4ec3cfd32245499bdf2f1d912986c56d98157e32`, two workers and all
119 routes / 238 page-device jobs. General navigation took 5558.336 seconds.
Initial and theme captures accounted for 9268.852 of 11055.035 aggregate
job-worker seconds. Tile positioning took 4841.288 aggregate seconds over
2059 tiles; screenshot calls, including PNG writes, took 3054.862 seconds.
These sums describe overlapping elapsed operations, not CPU or exclusive
wall time. Whole layout snapshots/proofs took only 54.125 aggregate seconds.

## Decision

The production background's animation loop ignores `prefers-reduced-motion`.
The render contexts request reduced motion, but the fixed canvas continues
drawing at every animation frame, including at enlarged 8192px viewports.
Continuous WebGL repaint is a plausible contributor to the capture cost.
The historical timings alone do not establish GPU causality or a speedup.

Honor the actual browser preference in `docs/theme/vein.js`. Reduced motion
keeps the authored shader visible and freezes time and pointer velocity.
Actual scroll, pointer, resize, theme and system-color changes request a
single coalesced repaint of the current state. Runtime preference changes
cancel the pending frame and resume or freeze the original normal animation;
elapsed static viewing time does not advance that animation.

Keep the existing capture producer, separate awaited scroll/position checks,
complete source/font/link/layout/theme/pixel assertions, native negative
controls, two workers, all routes/campaigns and deadlines. This product fix
does not supersede the [capture-boundary record](./2026-10-11-6830-docs-capture-boundaries.md)
or the [rejected position co-location](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6108544583).

## Qualification and remaining evidence

The new [browser regression](../../../tests/tooling/docs-background-motion-browser.test.ts)
uses the [background qualifier](../../scripts/background-motion-qualifier.ts)
with the genuine production Three.js URL, complete inline background script,
actual native events and real WebGL. Observation forwards the original
renderer and native draw calls; it does not replace or suppress them.
The source qualifier records every input hash, the library payload, source
head and dirty state, workflow identity and complete desktop/mobile results.
It rejects source changes during qualification and a different workflow SHA.
Hosted tooling retains fresh packets under `target/differential/docs-background-motion/`
through the existing always-run differential evidence upload on success and
failure. Packets retain the source files, library/script/HTML bytes, native
operation vectors and available original PNGs; local temporary runs clean up.

The original background fails the idle-frame assertion. The candidate keeps
one initial static draw, no idle draws over eight native frames, four active
draws over four normal frames, and a frozen time after runtime reduction.
Scroll, pointer, theme fallback, actual theme pixels, enlarged viewport and
restoration remain checked. This is bounded local renderer-operation proof,
not a completed full-site campaign or a hosted performance measurement.

Run exact-head Actions before queue admission. The next required Full Docs
campaign must retain every route/device/campaign and capture receipt with
operation timings. Compare the same workload and report actual hosted wall
time separately; keep CI performance causality and savings unfinished until
that terminal evidence exists. Pair this record with a concise #6830 comment
when the reviewable PR is created.
