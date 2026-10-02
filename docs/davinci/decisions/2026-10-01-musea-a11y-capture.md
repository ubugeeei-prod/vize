# Musea accessibility evidence and stable captures

Issues: [#7329](https://github.com/ubugeeei-prod/vize/issues/7329),
[#7330](https://github.com/ubugeeei-prod/vize/issues/7330),
[#7253](https://github.com/ubugeeei-prod/vize/issues/7253).

Preview wrappers use natural document height and scrolling. The low-priority
preview layer keeps its padding and viewport-based variant minimum height,
without a fixed-height `overflow: auto` body. Audit results are not filtered
to conceal wrapper violations. A Chromium/axe negative control restores the
old wrapper and requires the reported body failure to return.

Keep `nodes` and `incomplete` counts for report compatibility. Add per-node
`targets` with the exact selector arrays, HTML, failureSummary, and all `any`,
`all`, and `none` check results including measured data. `incompleteResults`
preserves the same details for manual-review findings. HTML escapes all node
information and presents review findings separately from violations.

Capture defaults freeze animations and hide the caret. Reduced motion defaults
to `no-preference` so components retain their normal visual design; explicit
`reduce` is forwarded to browser contexts. Users can set `animations: "allow"`
or `caret: "initial"` to retain live capture behavior. Finite animation end
states can change existing baselines when adopting the new default.

The real browser contract suite validates a tall document under a common
global reset, measured contrast details from axe, and four identical screenshots
of an authored infinite-animation fixture with reduced motion active. The
dedicated Musea Actions job runs these browser assertions with no skip, while
ordinary package tests retain browser-independent report coverage.
