# Practical composable documentation demos

Tracking: [#8374](https://github.com/ubugeeei-prod/vize/issues/8374),
[#6101](https://github.com/ubugeeei-prod/vize/issues/6101).

## Decision

Keep the existing source-owned UI basic examples and add six complete Vue SFC
examples beside the composable package: disclosure, bounded quantity, debounced
local search, blur validation, undo/redo, and offset pagination. Each demonstrates
an observable purpose, relevant transitions, public imports, and accessible
native controls. Every UI/composable entry gets minimal installation and project
context; the English/Japanese composable hubs link the practical examples.

The generator displays the entire same SFC packet that the preview compiles.
SHA-256 binds its source to the live iframe. The preview build typechecks these
SFCs against the current workspace source, compiles them with the existing UI
Vue/Vite compiler configuration, renders them through Vue's server renderer,
and hydrates the matching source in Chromium. SSR and browser builds use fresh
compiler-plugin instances; a dev-server plugin instance must not be reused by a
production build. Hydration must preserve all element identities and the full
initial markup, with no browser warnings or errors.

The browser checks assert all six examples' actual initial/interacted behavior
at 720px and 360px, including Enter/Space, keyboard blur, boundaries, delayed
search, flush/cancel, error/valid/reset, batching, redo invalidation, and page-size
clamping. Virtual browser time makes the real debounce timer deterministic.
Twenty-four actual PNGs and their hashes accompany the source hashes and
individual interaction receipts. These captures are illustrations, not VRT
baselines or a complete accessibility audit.

The first source checkpoint rejected ten inline-call handlers through the
unchanged opinionated `vue/v-on-handler-style` gate. Use explicit inline arrows
for composable controls; passing a method with an optional argument directly
could treat the native event as its argument. Keep the failed run as evidence
and require a fresh source run after this correction. Page-size changes are
also exercised with the keyboard, and count labels handle singular states.

After SSG, the existing docs acceptance entrypoint checks sixteen rendered
English/Japanese routes and local assets, exact complete displayed source,
iframe source/hydration identity, and the same six interaction laws. It supports
`--site` for the deployed site. Actions runs these checks in the existing Docs
build, with strict preview TypeScript inside the existing Check job; no new
workflow job or product pipeline stage is added.

## Delivery and remaining acceptance

- Local observations are not hosted Actions, protected merge, or deployment
  receipts. Obtain fresh exact-head Docs and Check runs, actual queue merge,
  and deployed browser acceptance before calling this slice delivered.
- Live interaction checks cover these six composable entries and the existing
  eight featured UI families. The other source-generated entries remain outside
  those interaction checks. Their complete practical/state coverage is still
  required by #8374; keep that issue open until its full acceptance is met.
- #6101 remains open for the broader compile/typecheck/lint/format/render,
  unsupported-claim/default coverage, tooling/editor metadata, and verified
  release-note matrix across all named products.
- No upstream n8n or provider writes are part of this change.

## Browser and asynchronous effect extension

Add six further source-owned SFCs: local async profile loading, pausable review
ticks, delayed reminders, persisted reading progress, responsive guide layout,
and contextual focus guidance. Keep the first six laws intact and dispatch the
new effect laws from small owned modules. The full preview group now contains
twelve examples, forty-eight initial/interacted desktop/mobile PNGs, and
twenty-two rendered documentation routes.

Use actual producers and browser capabilities. Virtual time advances real
intervals/timeouts and proves pause, reset, deadline replacement, cancellation,
single delivery, retry, and ignored stale async completion. The local sample
producer owns and disposes its simulation timers; ignoring stale results is
not network cancellation. No producer is invoked during setup.

Storage checks use the origin's localStorage and reload the actual application,
including reset leaving the key absent. Media-query checks resize the viewport
and replace the reactive query while inspecting actual grid columns. Focus
checks follow the document's active element through Tab, focus, and blur.
Hydration preserves the server fallback before post-render browser reads.
Capability failures remain visible in the storage example.

This is a dependent slice of the first practical-demo PR and must be registered
in a native GitHub Stack before queueing a contiguous passing prefix. Its exact
source Actions, protected merge, and deployed receipts remain required.
The remaining public catalogue and broader #6101 acceptance stay open.

## Form and navigation state extension

Add four practical SFCs for native form submission with local validation and
reset, an editable multi-step attendee/session/review flow, capped session
selection retaining stable keys across local data replacement, and cyclic
reading tips. Keep the existing twelve laws unchanged. The group now contains
sixteen complete source packets, sixty-four actual captures, and twenty-six
rendered documentation routes.

The examples use public composable imports and source-defined behavior; they
do not change package APIs or fabricate submission/network effects. Each new
law exercises meaningful native keyboard interactions, explicit boundaries,
and retained/reset state. The same SSR markup, element identities, displayed
source hash, diagnostics, and mobile overflow requirements apply.

Register this dependent extension in the same native Stack and merge only a
contiguous exact-head passing prefix. Remaining catalogue acceptance and #6101
stay open; require actual protected merge and deployed browser proof.

Read-only review found misleading state text and a nondiscriminating batching
input. A field's empty recorded error does not validate subsequent edits; use
neutral status and exercise valid-to-invalid re-edit before and after blur.
Unavailable storage keeps writable in-tab state; describe it accurately and
prove selection/reload in actual Chromium with localStorage disabled, with two
additional mobile captures and a separate receipt. Batch assertions must use
whitespace-bearing input so both grouped assignments change the value and Undo
restores the exact original whitespace.

The first form-extension source run rejected two static inline styles in the
selection example. Move them to source-owned scoped classes and retain the
unchanged opinionated gate. Keep the review corrections in the top layer and
queue the corrected complete Stack; earlier prefixes must not ship the known
misleading states.

## Scoped DOM interaction extension

Prepare four further public-import SFCs for individual/group focus, inline
outside dismissal, element-scoped review shortcuts, and native input
observation. This brings the group to twenty complete source packets, eighty
initial/interacted captures plus the two disabled-storage captures, and thirty
rendered routes. Keep all earlier interaction laws and source identities.

Use real template-ref targets and browser events. Distinguish live flags from
frozen observations after focus tracking stops. Outside dismissal includes the
trigger ignore list, pointer drag origin, native keyboard click, recreated
conditional target, and explicit example-owned Escape/focus-return behavior.
Shortcuts follow only the chosen native button, suppress repeated keydown, and
stop without removing the independent native navigation controls. Input
listeners detach on retarget and pause, then resume on the current target while
both v-model drafts remain editable. No example saves or sends data.

Preserve strict initial SSR markup and element identity checks. Defer textarea
autosize until its genuine mounted style mutation can be represented without
bypassing hydration assertions. The qualified sixteen-example Stack remains
immutable while its merge and production proof run. Publish this prepared
independent slice only after rebasing onto the coherent delivered main; do not
add it to the current queue during parent delivery. The remaining catalogue and
broader #6101 acceptance stay open.

The package build invokes rendered acceptance from `docs/`; a relative default
evidence path incorrectly writes under `docs/docs-render-evidence`, outside the
existing repository-level upload. Resolve that default from the known docs
root so the same artifact retains source/image hashes and interaction receipts
from the post-SSG pass. Explicit output paths remain caller-selected. Require
the actual Actions artifact's composable receipts before delivery credit.
The existing Build docs step requires both post-SSG receipt files before the
render-evidence upload; this fails with the previous package-relative default.

The before-fix [exact-head Docs run](https://github.com/ubugeeei-prod/vize/actions/runs/38028442319)
produced render-evidence artifact 11661933163 with 656 ZIP entries and no
component-preview files or composable receipt JSONs. Its first-pass captures
remain in the built public assets; the missing second-pass receipt is a
separate artifact-retention defect. The new existing-step assertions must
reject that artifact layout and a fresh Actions artifact must contain both
receipts after the path correction.

The first DOM-extension source Check rejected Escape on a static section via
the unchanged accessibility preset. Put the example-owned Escape handler on
its actual native input and button controls. Exercise both controls' Escape
delivery and real trigger-focus return; preserve the non-modal region, source
alignment, and all earlier interaction laws. Retain failed Check 38033519431
and require a fresh exact-head native gate, Docs build, and browser receipts.

The new native Escape handlers intentionally extend the natural modified-v-on
corpus. Regenerate its per-area inventory with the existing generator and
review the three `@keydown.esc` spellings in `npm--compose.tsv`; retain the
two-entry options/event/key capacity limits. Exact-source Check 38033941359
identified the omitted inventory, and the unchanged live-scan/storage test
passes after regeneration. A separate canonical-fixture checkout assertion
failure remains in the CI repair lane and must not be hidden by this update.

## Clipboard and element observation extension

Paired [issue decision](https://github.com/ubugeeei-prod/vize/issues/8374#issuecomment-6096095990).

Prepare four further complete public-import SFCs for editable install-command
copy/read, native guide scrolling, selected-panel size observation, and featured
card visibility in a bounded native viewport. The source group now contains
twenty-four SFC packets, ninety-six normal initial/interacted desktop/mobile
captures plus two disabled-storage captures, and thirty-four rendered routes.
Keep the previous twenty examples and their interaction laws unchanged.

Use actual browser capabilities and CSS geometry. Clipboard actions explicitly
write or request a read only after native button activation; support does not
promise read permission. Prove exact whitespace-preserving clipboard contents,
the replacement feedback deadline, a real denied read retaining the prior text,
and permission restoration clearing failure. Browser permission grants are
additive, so reset the context before granting only write permission. Wait for
the second asynchronous copy's displayed text before advancing its real timer.
The post-SSG checker also exercises this behavior inside the actual same-origin
documentation iframe, rather than assuming top-level clipboard proof suffices.

Scroll the actual readonly textarea through writable y, scrollTo, and native
PageDown. Edge controls follow rendered dimensions. Native keyboard animation
must finish with scrollend before comparing its final offset with the readout;
advancing JavaScript timers alone does not finish the browser's scroll animation.
Size callbacks report real contentRect, follow the selected template target,
disconnect on replacement, and retain the last delivered readout after stop
while native checkboxes continue resizing both panels. Visibility callbacks use
an actual ancestor root and 0.6 threshold. isActive means requested observation;
the example explicitly ignores inactive queued entries to retain its paused
readout. Resume observes current geometry; stop permanently disables reconnect.

Retain the full initial SSR markup and all element identities without hydration
exceptions. All browser hosts remain real; no synthetic clipboard or observer
is supplied to an example. Autosize stays deferred under the prior decision.
Keep this next slice unpublished during the current twenty-example admission;
publish only from a coherent delivered parent/main, obtain fresh exact-source
Actions and protected actual merge, and require real deployed browser proof.
The complete #8374 catalogue requirement and broader #6101 remain open.

A repeated follow-on capture observed the denied native query and failed read
without the reactive permission readout reaching denied. Preserve that failed
observation; eight independent native repetitions passed and recorded actual
prompt/denied/granted changes, so no provider defect is inferred from it.
Explicitly await the initial granted permission query before revocation in both
direct and embedded checks. Retain the exact denied/restored readout assertions,
actual clipboard operation results, and the original strict hydration laws.
Require a fresh full capture and Actions before publishing this prepared slice.

## Delivery from the merged twenty-example parent

Replay only the four clipboard/scroll/size/visibility examples onto actual main
`d55b8238f89bab4418a1a961bd8db1c84911cb42`, after #8461 actually merged. Preserve
every previous twenty-example SFC byte and all existing interaction/hydration
laws. This is an independent PR; the later 28/32-example preparations remain
separate and do not supply delivery credit. Fresh source Actions, full Docs,
protected actual merge and official deployed browser proof remain mandatory.

Retain the later preparation's failed native 360px clipboard observation:
the read correctly reported permission-denied while the subscribed readout
remained prompt after an earlier diagnostic query. Move that same native denied
query and assertion after the unchanged subscribed denied/error readout. Keep
every denial, retained-text, restored-permission, timer, whitespace and hydration
assertion. Failure-only diagnostics capture the actual DOM before any extra
query. The previously recorded bounded order comparison does not establish
general stability; this replay requires its own complete fresh Actions capture.
No package API changes or provider-fix claim are made. #8374 and #6101 stay open.

The first exact-head [Docs run](https://github.com/ubugeeei-prod/vize/actions/runs/38142919091)
fails the new native PageDown idle assertion. Keep its real scrollend, final
offset, idle and surrounding-document checks. Complete the preceding native
reset before attaching PageDown's completion listener, then advance the existing
200ms idle fallback after genuine PageDown completion. Preserve the 10-second
native completion deadlines and capture the actual target/output on failure.
This is driver sequencing; no product scroll defect is inferred from that run.
The source [Check](https://github.com/ubugeeei-prod/vize/actions/runs/38142914140)
also rejects the size example's descendant CSS selector through the unchanged
opinionated `css/prefer-nested-selectors` rule. Nest the same input declaration;
retain actual content-box dimensions and native keyboard controls. The local
24-example capture passes after these corrections, but fresh complete source
and Docs Actions remain required before protected queue admission or delivery.

The next exact-head [Docs run](https://github.com/ubugeeei-prod/vize/actions/runs/38143622223)
passes all twenty-four first-pass browser laws, genuine SSG, and 1423 OG images,
then fails the repeated post-SSG scroll bottom assertion. Await each actual
geometry-derived offset and matching reactive readout before the next native
Enter activation; keep the original 25-step bound and exact final bottom,
disabled-control, reset, PageDown, idle and surrounding-document assertions.
Record each completed native step for failure diagnosis. No product scroll
change, retry, deadline expansion or previous-run acceptance is introduced.
The failed run's existing render-evidence artifact remains preserved. Require
fresh complete source and Docs qualification on the corrected head.

A fresh local full capture after per-step synchronization fails the added
reset-scrollend requirement. An independently instrumented direct scroll case
passes at both widths; that does not establish the full-capture cause. The
public scroll contract supports a 200ms idle fallback rather than promising
scrollend for every programmatic reset. Require actual zero offset and the
observed idle readout after that unchanged clock interval before PageDown.
Remove only the new reset-event dependency; keep the original real PageDown
scrollend/deadline and every prior edge, geometry, idle and hydration assertion.
Preserve the failed local observation and require complete fresh Actions; do
not infer a product defect or claim general stability from the direct cases.
