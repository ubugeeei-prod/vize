# Musea VRT PNG diff styles

Issue [#10482](https://github.com/ubugeeei-prod/vize/issues/10482).

The public `comparison.diffStyle` option previously had no runtime consumer.
Every advertised value produced the same overlay PNG, and invalid values could
silently create a first baseline or succeed through the identical-image shortcut.
The configuration now selects one of three supported PNG presentations:

| Style               | Image contents                                                   | Dimensions                                 |
| ------------------- | ---------------------------------------------------------------- | ------------------------------------------ |
| `overlay` (default) | Existing highlights over semi-transparent grayscale              | Comparison width × height                  |
| `diffOnly`          | Existing highlights; unchanged pixels are transparent RGBA zero  | Comparison width × height                  |
| `sideBySide`        | Exact decoded baseline on the left, current capture on the right | Sum of input widths × maximum input height |

Side-by-side output preserves both images' RGB and alpha bytes; shorter inputs
have transparent bottom padding. Presentation does not change difference counts,
color sensitivity, alpha settings, anti-aliasing counts, or acceptance thresholds.
For mismatched dimensions, the existing 100% difference statistics remain based
on the maximum input width × maximum input height. Overlay and diff-only retain
the complete solid-red mismatch marker; side-by-side displays the two actual
input shapes. Zero-diff comparisons still create no diff artifact in any style.

```ts
musea({
  vrt: {
    comparison: {
      diffStyle: "diffOnly", // or "sideBySide" / "overlay"
      diffColor: { r: 255, g: 80, b: 0 },
    },
  },
});
```

`DiffStyle` exposes only the supported PNG styles. Animated output is unsupported.
Unknown JavaScript/configuration values throw the publicly exported
`UnsupportedDiffStyleError` with code `ERR_MUSEA_UNSUPPORTED_DIFF_STYLE` at runner
construction. Direct comparison also validates before reading inputs or taking
the equality shortcut. Capture revalidates before creating a browser context or
writing PNGs, so mutation of an already constructed runner's option object cannot
turn an invalid format into a new baseline or a passing zero-diff result.

The result's optional `diffStyle` identifies non-default PNG presentation. JSON,
gallery API responses, hosted immutable artifact URLs, and static HTML reports
retain it. Default overlay results omit this new metadata, and the complete
encoded overlay PNG remains byte-identical to the previous comparator. HTML
reports label a side-by-side diff artifact `Baseline / Current`, and transparent
output `Diff only`; existing overlay labels stay unchanged. All artifacts remain
PNG paths served as `image/png`.

## Verification

The source regression suite retains a complete pre-change overlay PNG stream and
asserts full decoded RGBA and dimensions for every supported style. It covers
hidden RGB under transparency, alpha comparison on/off, color tolerance,
anti-aliasing highlights, unequal dimensions, and transparent padding. Invalid
formats are checked before input file access, before first-baseline writes, and
after construction against an existing identical baseline. The original
comparator fails 11 of the 13 new controls.

The real Chromium contract creates first/matching/one-pixel-different captures,
checks their complete result objects and full diff PNG, downloads the actual
hosted artifact URL with exact MIME/bytes, parses the generated JSON, and loads
the physical HTML report's image to verify its path, label, and natural geometry.
The browser Actions lane retains PNGs, reports, and
`artifacts/musea-vrt-styles/observations.json`. Source qualification, protected
merge, publication, and fresh installed-package acceptance are separate gates.

This change does not add an overlay allocation to side-by-side comparison:
its statistics pass renders the paired image only when a difference exists.
The identical-image fast path remains allocation-free for all supported styles.
