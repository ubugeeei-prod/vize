# Musea literal variant-key qualification

## Diagnostic scope

This isolated successor starts from report-ownership PR #8488 at
`3f60a4908241ab5554d219b403e5f10666ffe2f5`. It does not change that PR, its
ancestors, queue admission, the hosted-service worktree, or publication state.

The authored source fixture is
`tests/tooling/fixtures/musea/literal-variant-keys/{Controls,Keys}.art.vue`.
Its corpus record binds the exact source bytes outside the differential compiler
collector. Ordinary `Default` and `Custom theme` variants are positive controls;
`__proto__`, `constructor`, and `hasOwnProperty` are literal authored data names.

`src/literal-variant-keys.browser.test.ts` runs only in the source-native Musea
Actions job. It uses the actual Vize compiler, Musea plugin, built gallery, Vite
static HTTP server, real Chromium, gallery Run VRT button, API, and runner.
Physical build outputs, static manifest, native preview documents, actual API
responses, complete report bytes, PNG bytes, and browser errors are retained in
`artifacts/musea-native-literal-variants`. No renderer, API, or compiler is replaced.

The static law exercises each actual emitted native preview, then the ordinary
gallery and literal-key gallery. It separately records physical JSON keys and
browser globals so serialization cannot hide loss of an own property or change
the dictionary prototype. The development law captures and repeats both Arts,
proves distinct physical PNG ownership and zero differences, and requires the
VRT pane to retain the exact authored group names without inherited groups.

## Genuine failure

Issue [#8489](https://github.com/ubugeeei-prod/vize/issues/8489) records the
source-native failure at `7854bab7739ab93303de2a5210d03b95bc0323f9`, run
[38042404142](https://github.com/ubugeeei-prod/vize/actions/runs/38042404142).
Only the two new laws fail; the unchanged eight native and complete Chromium
controls pass without skips. All five directly served physical native previews
render. The ordinary gallery renders both controls, but static manifest and
browser globals omit the own `__proto__` URL; its gallery iframe times out.
Static page/console errors are empty, so no static exception is claimed.

Real dev captures are `Controls: 2 new -> 2 passed` and `Keys: 3 new -> 3 passed`,
with zero differences on repeats and distinct physical PNGs. Whole JSON/HTML
reports and raw API responses agree on names, owners and results. Literal-key
VRT groups are empty after both successful API responses. Two actual source
console errors retain `TypeError: groups[key].push is not a function` at the
compiled `VrtPanel.vue`; dev page errors are empty. Complete source/build/report
bytes and PNGs are retained in the run artifact.

## Decision

The [paired issue decision](https://github.com/ubugeeei-prod/vize/issues/8489#issuecomment-6096283612)
preserves authored literal names rather than rejecting them. Static preview
dictionaries and VRT groups use null-prototype records. Embedded static previews
use safe JSON data parsing when an own `__proto__` occurs; ordinary script bytes
remain unchanged. Static preview lookup accepts only an own string URL, so
inherited functions or prototype objects cannot become navigation URLs.

The genuine source-native browser laws, original input bytes, normal/custom
controls, runner/API implementation, report ownership, PNG identities and
zero-difference repeats remain mandatory. Pure embedding/lookup guards also
prove ordinary script bytes, safe literal escaping and absent inherited URLs.
The shared security serializer and generic report generators remain unchanged.

Fresh native qualification, coordinated stack composition, protected merge and
publication are unfinished. The hosted browser VRT service remains separate.
The genuine before static `gallery.html` also retains authored `__proto__`
section `id` and its actual navigation `aria-controls` as `[object Object]`.
The other literal names have their correct IDs. The
[paired navigation decision](https://github.com/ubugeeei-prod/vize/issues/8489#issuecomment-6096318021)
includes the one-line own-key correction in `ComponentView.variantSectionIds`
within this same defect. Its grandfathered line count does not grow. Fresh
static/dev laws require exact authored names, expected distinct section IDs and
matching controls, with real selection of the literal default in the static UI.
The additional diagnostic head `f8f5adc9188ade0f8d07630025b1ea59ebe82f60` retains
original affected runtime bytes and adds dev section/control observations.
