# Musea VRT capture masks preserve layout

Issue [#10463](https://github.com/ubugeeei-prod/vize/issues/10463).

Capture masks previously deleted the matching element's children and modified its
background and text color. Auto-sized containers collapsed, unrelated siblings
moved, and images or background images were not reliably covered. In the real
Chromium regression, a 40px-tall container became 0px tall and moved the following
element from y=40 to y=0. The screenshot no longer described the authored layout.

Pass matching locators to the browser's screenshot compositor instead. This
covers each rendered rectangle without changing the component DOM or layout.
Selectors may match multiple elements; unmatched selectors leave the capture
unchanged. Empty or malformed selectors fail the capture. Invalid mask colors
also fail, and error paths create no baseline and release the page/context.
Color validation takes one browser evaluation regardless of selector count;
masking no longer evaluates a DOM mutation for each selector.

`capture.maskColor` accepts a browser-supported CSS color and defaults to the
existing magenta `#ff00ff`. It applies to the configured mask rectangles:

```ts
musea({
  vrt: {
    capture: {
      maskElements: ["[data-vrt-volatile]", ".avatar img"],
      maskColor: "#123456",
    },
  },
});
```

The real Chromium contract checks every screenshot pixel at normal and 2×
device density, default/custom colors, auto-sized containers, an actual image,
unchanged DOM content and geometry before/after capture, and zero differences
when masked content changes. A change outside the masks must retain its exact
diff count, percentage, complete current PNG, and complete diff PNG. Invalid
selector/color controls must return the complete failed result with no baseline
or live context, and a missing selector must produce byte-identical PNG output
to an unmasked capture.

The dedicated browser Actions lane runs the contract and retains PNGs and
`artifacts/musea-vrt-mask/observations.json`. The original source fails the DOM
geometry control; the corrected local source passes every control. Exact-head
Actions, protected merge, publication, and installed-package acceptance remain
separate delivery gates.
