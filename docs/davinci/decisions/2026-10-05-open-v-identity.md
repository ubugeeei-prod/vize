# Open V identity (2026-10-05)

Tracking issue: [#7848](https://github.com/ubugeeei-prod/vize/issues/7848).

The existing skewed V used fine speed trails and a padded landscape canvas.
Those details disappeared at icon sizes, and the docs, playground and galleries
carried different shapes. Adopt a compact V formed from two solid planes with
an open diagonal between them. Keep warm ink `#121212` and paper `#e6e2d6`.

## Assets and use

- `assets/logo-mark.svg` is the canonical 64 × 64 square mark;
  `logo-mark-light.svg` carries identical geometry in paper.
- `assets/logo.svg` and `logo-light.svg` are separate 160 × 64 horizontal
  lockups. Their Helvetica Neue wordmark is outlined, so rendering does not
  require a font. The PNG exports are 600 × 240.
- Docs and playground public marks link to the canonical assets. Icons,
  favicons and headers use the square mark; headers retain their existing text.
  The README uses the horizontal lockup with a dark theme source.
- Playground and both gallery header implementations use the same two paths
  with `currentColor`. The playground Open Graph image uses the outlined
  lockup, with its descriptive text in the existing system font family.
- The VS Code icon is a 512 × 512 paper tile with the ink mark. Its background
  preserves contrast in light and dark editor and marketplace surfaces.

The mark has no fine strokes, gradients, filters or font dependencies. Keep the
64 × 64 view box and its built-in clear space when placing it. Use the mark for
small interfaces and the lockup where the name needs to be visible. Do not
stretch either asset or use a social image as a favicon.

## Verification and delivery

Before source changes, three vector studies were rendered and compared at 16,
24 and 48 pixels in ink and paper. The open V retained its diagonal opening at
16 pixels and kept the previous mark's two-plane character.

Chrome 152.0.7977.65 then loaded all 13 actual asset placements in a bounded
review page: both lockups, 16/24/48-pixel marks, header marks, editor tiles and
the social image. Both favicon media choices resolved to the square marks,
and the editor PNG retained its actual 512 × 512 dimensions at a 96-pixel
placement. The two existing gallery template security tests and formatting of
the affected presentation sources pass locally. This asset review does not
establish deployed page layout or release completion.

PNG exports are rendered from SVG with `rsvg-convert` 2.56.3. Review the actual
assets in both themes, run affected JavaScript checks on Actions, and use the
protected merge queue for full validation. The docs build and GitHub Pages
deployment must complete for the merged source, followed by a live asset and
page check. Editor distribution requires the release owner's package and
publication checks. Until those terminal checks pass, delivery is unfinished.
