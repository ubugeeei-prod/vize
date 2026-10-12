# Docs visual readability

The maintainer requested clearer light-theme code colors, a sidebar that joins
the page background, removal of decorative left rails, and a more visible
background shader. This bounded reader-quality slice pairs #8374 and #8365.

## Product behavior

Light code uses separate purple keywords, green strings, orange numbers,
teal types, blue functions and red-purple tags/properties. Native annotation
backgrounds still require the existing 4.5 contrast floor. An initial teal
candidate genuinely failed at 4.27; the corrected palette passes the unchanged
contrast law, including focused, dimmed and semantic code lines.

Desktop navigation shares the page's transparent surface. The mobile drawer
uses the actual theme background opaquely so underlying prose does not appear
through its links. Existing spacing, fonts, authored casing, keyboard disclosure,
active-link emphasis, focus outlines and semantic code annotations remain.
Decorative navigation, TOC and blockquote rails are removed; callout titles and
their existing semantic colors remain available.

The real canvas was hidden behind opaque content surfaces and had opacity 0.08
on articles. Transparent layout/main surfaces expose the same authored canvas;
opacity is 0.24 on articles and 0.48 on homepages. In the dark fragment branch,
pattern strength belongs in alpha rather than multiplying RGB a second time.
Transparent Three.js draws store premultiplied RGB, so the native canvas now
declares premultiplied alpha for matching browser composition. The previous
straight-alpha context rendered nearly black even with genuine dark=1 frames:
an observed native framebuffer sample was RGBA [18,18,16,19]. The final actual
PNG shows the veins clearly while keeping readable prose. All six noise layers,
shader timing, pointer/scroll behavior and reduced-motion repaint rules remain.

## Qualification and delivery

The original public product refuses the new native visual law: its callout rail
is 3px, article canvas opacity is 0.08, and WebGL premultipliedAlpha is false.
The existing theme gate adds native TS/Vue palette controls and observes actual
computed hue separation, loaded rails/surfaces and canvas context. Canvas size
and context are structural evidence; actual forwarded GPU draws and original
PNGs supply drawing evidence. No CSS is injected to make a product pass.

Local native SSG theme packaging plus preserved public page bodies passes eight
light/dark desktop/mobile guide/home views, genuine theme controls, existing
contrast/casing laws, fonts and overflow. Both existing WebGL motion and
serial/parallel custody browser laws pass. These local observations do not prove
official publication: exact-head Actions, the full protected queue, actual merge,
the merged-source official Docs campaign and stable-public visual checks remain
required. Intermediate refusals and GPU packets are retained separately.
