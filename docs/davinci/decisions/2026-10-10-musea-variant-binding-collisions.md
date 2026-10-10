# Musea authored variant binding collisions

Issue: [#8478](https://github.com/ubugeeei-prod/vize/issues/8478).
Paired [decision](https://github.com/ubugeeei-prod/vize/issues/8478#issuecomment-6095930584).

Distinct names `State enabled` and `State-enabled` both became the generated
binding `StateEnabled`. Public npm 0.440.0 fails a genuine native Vite production
build with a duplicate-binding parse error. Changing only the second name to
`State disabled` builds 114 files; plain HTTP/Chromium renders different Space
and Dash labels with native scoped CSS. The fixture, whole error/output and
module hashes are retained. The original public consumer's 2,061 files/symlinks
remain unchanged before/after the build and browser controls. This macOS
product diagnostic is separate from Linux release acceptance and source proof.

## Decision

Use one pure two-pass `variantComponentNames` plan. Preserve every unique
Pascal-normalized binding byte-for-byte. For every member of a collision, use
`__MuseaVariant_<full SHA-256 of the JSON-serialized exact authored name>`.
JSON serialization also distinguishes lone-surrogate API strings. These valid identifiers
are independent of order, unrelated variants and project relocation; their
leading underscores cannot come from the ordinary Pascal normalization. Reject
duplicate exact authored names rather than emitting ambiguous exports.

Compute the plan once per art-module emission, then reuse it for every variant
and the selected default. Virtual previews, development fallback and props
override use the same helper. Avoid a mutable cache: HMR adding or removing a
collision must change both the producer and the consumer consistently. Vue 2
receives the same selected binding. Each independently requested preview computes
a linear plan from the current variants, so a full N-preview build can compute
N plans; whole-build linear lookup cost is not claimed. Sharing plans safely
requires a parser-owned revision or immutable variants boundary, rather than a
cache keyed only by a mutable array. Variant module IDs retain their authored
names and the existing percent encoding.

The grandfathered art-module file shrinks by reusing the identical existing
`componentNameFromSource` export. Ordinary Vue 2/Vue 3 module SHA-256 controls
were captured from the frozen pre-fix source and remain exact. Unit laws cover
punctuation, Unicode normalization, numeric names, defaults, duplicate rejection,
all preview consumers, props overrides and HMR membership changes.

## Genuine native build and HTTP law

A separate source-native browser law copies the original red and ordinary
fixtures, calls real Vite build with the actual Vize/Musea plugins and requires
the built gallery. A plain server serves only emitted bytes. Chromium checks
all four preview URLs, exact authored names, their distinct Space/Dash labels,
scoped native CSS and no unresolved host element. It retains exact input hashes,
manifest, whole build failures, response bodies/hashes, DOM and screenshots.
The six literal Self/colon/globals/Document laws and original public acceptance
helpers stay in the native job unchanged.

This slice handles variant-to-variant normalization collisions. A unique variant
can still collide with an authored art-level import, and an author could reserve
a generated fallback binding themselves; arbitrary module namespace allocation
is outside this issue. Do not describe the variants-only plan as universal
binding collision support.

## Delivery

Prepare this source slice separately from #8470. Preserve all current PR heads,
compiler collectors, differential expected outputs and release authority. Native
source Actions success is an intermediate milestone. Genuine exact-head checks,
protected actual merge and subsequent public release qualification are still
required before reporting delivery.
