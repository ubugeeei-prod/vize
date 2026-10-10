# Gallery VRT project options

Issue [#8477](https://github.com/ubugeeei-prod/vize/issues/8477), paired with
[the implementation decision](https://github.com/ubugeeei-prod/vize/issues/8477#issuecomment-6095775268).

The gallery's **Run VRT** action previously discarded `musea({ vrt })`, even
though the CLI used those options. Pass the authored screenshot configuration
through the existing plugin context. Resolve relative snapshot directories
against the Vite root, preserving absolute paths and the default directory.
Retain the actual public preview route and mandatory mounted-preview handshake.
The separate accessibility pane owns accessibility audits; its options are not
the screenshot runner's boolean flag.

Actual native Actions [38038583206](https://github.com/ubugeeei-prod/vize/actions/runs/38038583206)
at `456c533ffe90fbb960d96958b4f809e634c7b32e` retains the original failure:
two default desktop/mobile captures and `.vize/snapshots` instead of the authored
single 320x180 viewport and `reviewed-baselines`. The native inline rendering
controls passed; the API law failed `2 !== 1`.

The successor uses the real source-built native compiler, Vite gallery, gallery
button, Playwright capture and physical PNGs. Require the authored viewport and
directory, repeated comparison, a real source color change under the authored
threshold, baseline update and a clean subsequent comparison. Fresh full source,
native/browser, protected actual delivery and publication remain required.

[The watcher decision](https://github.com/ubugeeei-prod/vize/issues/8477#issuecomment-6095868116)
retains the first corrected source `6087698fb2bb9215ecc49a9c2813b9de5c29fcbb`, which
proves the authored first capture and repeat, then retains generated report HTML
triggering a real gallery reload. The production watcher now excludes only the
owned `.vize/reports` directory. No test-only watcher override is used. The
actual gallery Document must survive repeat captures; an authored Art edit
still performs its normal navigation before the test re-enters VRT.
Native [38039242572](https://github.com/ubugeeei-prod/vize/actions/runs/38039242572)
at `61253ff217f391281993a516aee261a4cb95b69c` passes three controls with no skips:
one 320x180 capture, zero-diff repeat, a physical color change producing
9.215277777777779% difference under the authored threshold, baseline update,
then another zero-diff comparison. Full source/protected and publication
acceptance are still pending.

Two unfinished follow-ups stay separate: per-Art reports use basename-derived
filenames that can collide, and a hosted browser VRT action requires a Node
screenshot service. The existing hosted CLI remains available.
