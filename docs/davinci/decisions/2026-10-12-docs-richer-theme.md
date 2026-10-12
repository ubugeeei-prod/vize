# Richer dark background and saturated light code

The follow-up to [Docs visual readability](./2026-10-12-docs-visual-readability.md)
addresses the maintainer's request for a noticeably deeper dark shader and more
vivid light code colors on the surfaces tracked by #8374 and #8365.

Light code uses six fully saturated hues. The existing browser visual qualifier
now requires at least 80% HSL saturation from the actual loaded native token
styles, alongside its six distinct hues and minimum 20-degree hue separation.
It rejects the original public palette. This observes rendered colors, not
literal CSS values. The native annotation qualifier still requires WCAG 4.5:1
for ordinary, focused, and dimmed code; dimmed lines retain readable opacity.

Dark veins use a deeper bronze color and stronger pattern alpha, with separate
dark opacity for the homepage and articles. RGB remains independent of pattern
alpha, preserving the earlier premultiplied composition fix. The six noise
layers, animation rate, pointer and scroll uniforms, native theme switching,
and reduced-motion idle behavior are unchanged. Light GLSL calculations are
unchanged; increase only homepage canvas opacity from 0.48 to 0.60 and article
opacity from 0.24 to 0.32 for a slightly darker background. Dark opacity remains
0.64 and 0.40. Natural letter casing, transparent desktop navigation, opaque mobile
drawers, and the removal of left decoration rails remain in place.

The first exact-head Actions run failed the existing source-length ratchet:
`style.css` grew to 1070 lines from 1058. The complete failed log is retained.
Move the entire canvas block unchanged into `shader.css` and include it in the
existing theme concatenation. The large stylesheet is now 1041 lines. The
ratchet and all original checks remain unchanged; rerun them on the new head.

The finite native preview uses installed Ox Content SSG asset packaging and
whole downloaded public guide/home HTML. At 1440 and 390 pixels, eight light/dark
views plus two native mobile drawers pass the existing contrast, layout, casing,
hue, and overflow checks. The minimum observed contrast is 4.82254 for light and
5.08058 for dark. Actual forwarded GPU draws and 49 immediate framebuffer RGBA
samples remain premultiplied and visible; eight idle reduced-motion frames add
no draws. The existing authored background-motion browser test also passes its
two ordinary/reduced-motion cases and native runtime updates.

The original public browser baseline is retained, including the deliberate
light saturation refusal. A first baseline observer left native control nodes
behind after that expected refusal; the corrected observer removes its own
annotation control before the next view. Both raw receipts are retained. Native
annotation/palette controls are qualification fixtures, not authored page code.

Preview evidence is not publication evidence. Complete exact-head Actions and
the protected queue, then verify the sole automatic Docs build/deployment source,
whole physically loaded public asset bytes, and actual public light/dark
screenshots with the same finite checks. Do not dispatch a second Docs build or
add a separate site-wide campaign for this style change. Keep #8374 and #8365
open for their remaining reader-facing work.
