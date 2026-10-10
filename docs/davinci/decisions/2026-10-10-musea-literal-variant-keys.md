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

Source inspection suggests two failures: a plain preview dictionary loses the
`__proto__` own URL; the VRT pane's plain dictionary treats inherited properties
as result arrays. These are diagnostic hypotheses until genuine native Actions
evidence establishes the failure. No runtime fix, issue completion, merge, or
release is claimed. Any verified fix will receive its own focused issue and
paired canonical decision before implementation.
