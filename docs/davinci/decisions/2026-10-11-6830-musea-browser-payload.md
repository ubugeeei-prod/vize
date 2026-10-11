# Browser-safe variant navigation diagnostics

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The original Musea queue run `38135879860`, job `114459517219`, at
`872957a02d953bf4afe42f5c3e708f288ce43763` ran all 47 original cases:
46 passed, one failed, none skipped. The hosted first-target navigation wait
reached its original 30,000 ms deadline. Both existing before/after collectors
instead recorded `ReferenceError: __name is not defined`. The nested named
callback was transformed by tsx into a host helper unavailable in Playwright's
browser realm. That collector defect prevents state capture; it does not prove
why the original navigation wait timed out.

Keep the existing collector and all fields in their original order. Use direct
anonymous browser callbacks for bounded, first and last item reads, retaining
the bounded-read/timestamp/first-read/last-read order. The original wait, clicks,
deadline, assertions, hosted/native cases, corpus and producer remain unchanged.
Do not inject a browser `__name` global or change the gallery's observer policy.

An appended law in the existing timer-free navigation test executes the actual
tsx-serialized callback in an empty standard realm until its first input
operation. It defines no browser globals. A separate single controlled browser
execution checks the original serialization failure and complete corrected
packets for zero through four items, including null and bounded-overflow cases.
These controls prove diagnostic collection only and do not execute a product
build, native build or hosted VRT campaign.

Local controls retain all three original timer-free laws: the appended law
fails against the exact original callback and passes after repair. One actual
Chromium execution under tsx records ten original `__name` errors and ten
complete corrected packets across the five item counts and both original
phase labels. The corrected collector also preserves a real closed-page error.
Strict TypeScript, lint and formatting checks pass. These are controlled local
diagnostic results, separate from the original hosted failure and queue gates.

TODO: the sole #8617 publisher composes this minimal correction with the frozen
CLI changes after source review. The original hosted timeout remains unresolved.
Its original artifact has no final failure DOM/PNG, complete failure geometry or
raw IntersectionObserver event stream. Preserve that absence and the full
original artifact; an observer-batch source counterexample is not runtime cause
evidence. The existing protected suite must qualify any final composed source.
