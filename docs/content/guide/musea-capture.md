# Stable screenshots and accessibility details

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

Run `musea-vrt --a11y --json` to save accessibility reports. Each violation
retains its `nodes` count and adds `targets` containing the selector arrays,
HTML snippet, failure summary, and `any`/`all`/`none` checks. Check `data`
includes measured contrast colors and ratios when axe provides them.

`incompleteResults` carries the same details for findings needing manual review;
`incomplete` remains a count. Expand a node in the HTML report to inspect its
markup, failure reason, and measured check data.
