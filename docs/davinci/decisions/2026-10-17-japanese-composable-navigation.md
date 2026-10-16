# Japanese composable demo navigation

Related: #8372, #8374. This is one reader-facing slice; neither issue is complete.

## Problem and decision

The Japanese composable hub linked 124 working example pages through English task
labels. Its long introduction still described an older subset of demonstrations.
A reader could not choose a task in Japanese, and the wording did not distinguish
an available feature check from successful device or permission-dependent usage.

Review all 124 purposes against their maintained example metadata and replace only
Japanese task labels in the canonical generator. Explain the English destination,
shared complete SFC, Vue setup and scope cleanup in natural Japanese. State the
browser/device/permission boundaries before the full API catalogue.

Keep every English task label, public API name, destination, heading, example SFC,
preview compiler and interaction assertion. Other locales retain their existing
English demonstration labels.

## Verification and remaining acceptance

The existing reference-docs test requires exact catalogue/label correspondence,
Japanese task wording, unchanged English links and the stated support boundary.
Restoring the original English-label rendering fails on the first virtual-list
link. All 11 reference-docs tests pass with the corrected generator.

The existing post-SSG browser gate checks each exact Japanese label and destination
at 1440px and 390px, verifies native link focus, and retains initial/device-goal
screenshots in the existing render-evidence artifact, alongside unchanged routes,
previews and overflow checks.
Fresh Actions, protected actual merge, the official Docs build and public replay
remain required. Whole-site editorial review, English destination localization,
and each API's full successful device/permission coverage remain outside this slice.

## Retained capture correction

Source `324dfdf251` passed full Docs run `38925947595`, including all 124 labels
at both widths. Visual inspection of its four original screenshots showed correct
initial reading views but an unsettled focus-driven scroll: the desktop device-goal
capture was blank and mobile remained near the introduction. Keep that evidence as
a failed capture, rather than crediting it as device-goal reading acceptance.

Remove the queued PR before correction. After native focus, scroll the actual link
into view immediately, allow two native animation frames to paint, and require its
whole rectangle inside the viewport before capturing. Preserve the exact-label,
destination, native-focus, overflow, SFC and existing interaction assertions.
Requalify the fresh head in Actions and visually inspect the new images.

The fresh Linux run `38927709743` rejected the whole-link viewport assertion even
though the two-frame correction passed native before-controls and the exact prior
built Japanese HTML on local Chromium. Do not credit those local controls as Linux
acceptance. Focus the native link with `preventScroll` so it does not start another
scroll animation, perform the explicit native scroll, and wait for the actual full
link rectangle and scroll offset to remain stable for three consecutive frames.
Keep a bounded deadline and report the last observed geometry on failure. The full
viewport assertion and all earlier interaction/source checks remain unchanged.

## Integration with the new delivery-quote example

PR #9278 adds `async-resource` as the 125th live SFC example. Restack this review
onto its exact source `e68baea4571fb327067c3ecffad0e2e5e82dbac6` and add its
Japanese task label, retaining its complete SFC, native abort/timer/disposal checks
and both existing 125-entry assertions. Do not add a future-key compatibility branch
or relax exact catalogue/label correspondence. The first 124-example Check and full
Docs receipts remain historical proof; qualify the coherent 125-example head afresh
and follow the parent through actual merge, main synchronization and public replay.

GitHub rejected native Stack linkage while #9278 was queued and again after it
merged. Keep its branch and queue untouched: the review used its exact source as
a genuine child base, then replayed only owned changes onto actual merged main.

PR #9282 merged as `12e1d8e988011b5cfcf433e5d5f66b3ea71112a6` and changes the same
checker. Resolve only the shared filesystem import and adjacent route checks,
preserving its TOC pointer controls, route, evidence receipts and stylesheet fix
alongside the Japanese label, native-focus, geometry and existing preview gates.
The 125-example source `9398c5a472` passed full Docs `38932063431` before this
integration; the coherent head still needs fresh Actions and public acceptance.
