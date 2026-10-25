# Stable screenshots and accessibility details

Use this page when animation frames or a blinking caret make visual comparisons
vary, or when an accessibility finding needs a closer look at the affected node.
First set up a running gallery using the [Musea guide](./musea.md), then choose
capture settings and inspect the report before changing a baseline.

Musea freezes CSS animations and hides the text caret during screenshots by
default. Finite animations finish and infinite animations return to their initial
state. Existing baselines can change when they previously captured a live frame.

Configure capture behavior in the Musea plugin:

```ts
musea({
  vrt: {
    capture: {
      animations: "disabled", // "allow" keeps animations running
      reducedMotion: "reduce", // default: "no-preference"
      caret: "hide", // "initial" keeps the text caret
    },
  },
});
```

To compare an interactive state, use `capture.beforeCapture`. The callback receives
the isolated Playwright `Page` and a read-only `{ artPath, variantName, viewport }`
context. The CLI reads the callback from your Vite configuration; the same option
works with `MuseaVrtRunner`.

```ts
musea({
  vrt: {
    capture: {
      async beforeCapture(page, { variantName }) {
        if (variantName !== "Submitted") return;
        await page.getByRole("textbox", { name: "Message" }).fill("Hello");
        await page.getByRole("button", { name: "Submit" }).click();
        await page.getByText("Message received", { exact: true }).waitFor();
      },
    },
  },
});
```

The runner awaits the callback after preview readiness and `waitSelector`, then
applies `settleTime`, hidden elements, masks, and screenshot settings. Each capture
and retry gets a fresh page, including when `workers` runs captures concurrently.
A thrown or rejected assertion produces a capture error prefixed with
`beforeCapture failed:`. It closes the page and context without writing a current
image or creating or changing the baseline for that failed capture.

Run `musea-vrt --a11y --json` to save accessibility reports. Each violation
retains its `nodes` count and adds `targets` containing the selector arrays,
HTML snippet, failure summary, and `any`/`all`/`none` checks. Check `data`
includes measured contrast colors and ratios when axe provides them.

`incompleteResults` carries the same details for findings needing manual review;
`incomplete` remains a count. Expand a node in the HTML report to inspect its
markup, failure reason, and measured check data.

For the first run, keep the gallery server running and use the
[visual regression workflow](./musea.md#visual-regression-testing) to compare
the baseline, current image, and diff. Check whether a change comes from your
component or the capture settings before approving a new baseline. Setting
`reducedMotion` to `"reduce"` asks the page to use its reduced-motion behavior;
the default remains `"no-preference"`, independently of animation freezing.

The CLI writes accessibility reports to the output directory, `.vize` by
default. With `--json`, inspect `a11y-report.json`; without it, inspect
`a11y-report.html`. Review `incompleteResults` manually rather than treating
those findings as passes. An audit error means a variant was not audited,
so inspect that error before drawing a conclusion from the findings count.
