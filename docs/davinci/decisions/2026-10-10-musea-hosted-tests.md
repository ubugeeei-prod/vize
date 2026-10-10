# Hosted Musea accessibility tests

Issue: [#8378](https://github.com/ubugeeei-prod/vize/issues/8378).

A static preview's HTML imports a runtime entry that dynamically imports the
variant module. Async `previewSetup` delays mounting further. The iframe `load`
event does not establish that `__museaInitAddons` has installed its command
listener. Dispatching `musea:run-a11y` at that event can silently lose the command.

Use the existing `musea:ready` message from the exact iframe and gallery origin.
The summary worker registers before navigating its slot, waits at most ten
seconds, and removes its listener on completion or failure. The component A11y
panel disables dispatch until its own iframe signals readiness and resets this
state when its preview URL changes. Generated preview modules remain unchanged.

Audit errors are failed tests even when their violations array is empty. The
summary retains the error text; successful audits still report their real axe
passes and violations. This applies to fresh runs, cached results and reruns.

The persisted hosted fixture has six variants, which exceed the four worker
slots. Its real Vite-built gallery and generated previews run over HTTP beneath
`/site/__musea__`, with delayed setup and the real axe bundle. It asserts three
clean audits and three `button-name` violations, two full runs, setup finishing
after HTML load, the component-panel audit, and six failed audits when the host
refuses the axe bundle. No Vite middleware or intercepted API response executes
at runtime. Vue component modules are explicit test fixtures; native Art
compilation is outside this browser contract. Actions retains observations and
a screenshot.

The original gallery timed out before completing the first run in this actual
built/hosted reproduction. Exact source Actions, protected queue qualification,
actual merge and released-package acceptance are separate required steps.

Static hosting cannot execute Node/Playwright screenshot comparisons or update
local baselines. Its VRT panel therefore explains `vp exec musea-vrt` and the
development-server requirement instead of presenting an inoperable button.
Hosted VRT execution and automatic global-combination capture remain unfinished;
this change does not claim to complete those features.

The browser regression acquires Chromium inside its guarded lifecycle. Nested
finalizers attempt browser and HTTP-host cleanup after launch, artifact-write
or browser-close failures. Audit steps, fixture bytes, predicates and timing
bounds remain unchanged.
