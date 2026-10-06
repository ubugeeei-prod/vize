# Musea code presentation and source custody

Paired with [#8083](https://github.com/ubugeeei-prod/vize/issues/8083).

## Decision

The gallery preserves each code string and clipboard payload. It does not
reformat template text, JavaScript literals, `<pre>` contents, tabs or significant
spaces. `usageScript` and `indentUsage` remain unchanged: code presentation is
separate from compiler/formatter behavior.

The outer `pre` owns its padding and scrolling. Highlighted code has no extra
padding, wrapping or inner scroll area. Long lines remain intact and reachable;
tabs render at two columns. Source, usage, props JSON and expanded action JSON
share the same font, line height and keyboard-focusable scroll treatment.
Props grid tracks can shrink; its full preview/source column stops sticking on
narrow layouts so users can reach all content. Documentation already preserves
code with one outer scroll area and two-column tabs; it uses the corrected
highlight colors. The playground uses a separate `CodeHighlight` renderer and
is not changed by this gallery fix.

Light Highlight.js selectors apply only to an explicit light theme, an unthemed
root, or system mode preferring light. Explicit dark and system-dark retain the
dark token palette. Light comments use the gallery's readable muted text color.

## Original evidence and regression

Audited base: `143c1d4a9fbe6530cdd06f1f001b816f39c1d00a`.
The source gallery's original Button fixture plus a long property value showed
12px padding on both `pre` and `code`, word-breaking usage, and light token colors
inside dark mode. The unchanged base also fails the new browser law at the
extra `code` padding. This is a frontend defect, not a measured compiler defect.

The differential UI fixture in
[`code-display.json`](../../../tests/_fixtures/differential/musea/code-display.json)
retains tabs, Unicode, meaningful preformatted spaces, long lines and tall code.
The existing Button source remains unchanged. Real Chromium mounts the actual
source gallery and supplies explicit fixture API responses; its preview is
labelled as a fixture. This qualifies UI rendering, not native parsing or
preview compilation. Whole source and usage/JSON text, real clipboard reads,
keyboard scrolling, the final glyph of the longest line, vertical reachability,
light/dark/system colors and 1280/768/390/320px widths are mandatory controls.

The existing Musea browser Actions lane runs this law and retains original
observations and screenshots, including partial observations on failure. No
screenshot baseline, old fixture, compiler output, instruction budget, pipeline
stage or dependency version changes. Local browser evidence is preliminary;
exact-head Actions, protected candidate and actual signed merge own delivery.

## Remaining work

This does not add a formatter or normalize authored code indentation. Playground
renderer changes, editor formatting and native preview integration require their
own evidence if a separate defect is reported. Native-stage completion, whole
compiler parity and performance gains receive no credit from this UI law.
Release and published-package verification remain required after actual merge.

The keyboard control waits for stable native scrolling frames before resetting
and capturing a panel. Its first local pictures were taken while Chromium's
arrow-key scroll animation was still moving; non-keyboard gallery inspection
confirmed source insets were correct. This repairs screenshot custody, without
changing the UI, whole-source or scroll-reachability expectations.

## Security-main replay

Historical head `95188afcbc05e5305711ff8b62f16a9489094494` passed the
whole 8 template / 8 usage Chromium observations in run `37409961772`,
with official screenshot artifact `11388962023`. Its ordinary source Check
`37409962074` failed the unchanged production security audit; no source-green
or protected instruction acceptance is attributed to that run.

The independent security repair #8080 actually merged as signed
`48cb1d4f35ebd0aa3824817c67752b5e4d0a9dbb`. This branch replays onto
that actual main while preserving all ten non-document blobs, the raw generator
and original Button, and both original commit author/date/body/footer records.
Every incoming canonical line remains complete. Fresh configured Actions and
protected candidate execution must qualify this composition; the previous
Chromium artifact remains historical. Actual merge and installed published
gallery verification still own delivery.
