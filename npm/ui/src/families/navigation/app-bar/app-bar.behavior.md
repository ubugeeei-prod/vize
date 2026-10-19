# AppBar behavior contract

Normative state x input -> outcome table for `app-bar.vue` (`@vizejs/ui/app-bar`). An app bar
is the page's top bar: a `header` landmark with leading, main, and trailing content. It only
reports scroll state (`data-elevated`, `data-hidden`, `data-state`); positioning, elevation, and
the hide transition stay consumer-owned CSS. Every row is proven by the named test in
`app-bar.test.ts` or `app-bar-ssr.test.ts`.

| #   | State            | Input                          | Outcome                                                                                                             | Proven by                                                                             |
| --- | ---------------- | ------------------------------ | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| AB1 | render           | slots / `as` / labels          | `<header>` (or `<div>`) with `start`, `content`, `end` parts; empty `start`/`end` render nothing; ARIA labels apply | `renders a header landmark with start, content, and end parts`                        |
| AB2 | window scroll    | offset past `elevateThreshold` | `data-elevated="true"` strictly past the threshold, removed at or below it; never hides without `hideOnScroll`      | `reports elevation once the window scrolls past the threshold`                        |
| AB3 | `hideOnScroll`   | scroll down / up               | hides after `hideThreshold` while scrolling down, reveals on any upward scroll, emits `visibilityChange` each time  | `hideOnScroll hides while scrolling down past the threshold and reveals on scroll up` |
| AB4 | hidden           | focus inside / `show()`        | focus reveals the bar and keeps it visible until focus leaves; `show()` reveals it                                  | `focus inside a hidden bar reveals it and keeps it visible`                           |
| AB5 | element `target` | selector container scroll      | follows the selected container and ignores window scrolling; `refresh()` re-reads the offset                        | `follows an element scroll container given as a selector`                             |
| AB6 | option changes   | `hideOnScroll` off / NaN       | turning `hideOnScroll` off reveals the bar; non-finite thresholds fall back to their defaults                       | `turning hideOnScroll off reveals a hidden bar, and invalid thresholds fall back`     |
| AB7 | SSR / hydration  | isolated requests              | byte-identical visible markup without scroll state, hydrating without diagnostics                                   | `renders byte-identical app bar markup and hydrates without mismatches`               |

Hiding only publishes state: the bar keeps its content in the DOM and in the tab order, and
keyboard focus brings it back, so a consumer's off-screen transform never traps focus out of
view. Use `as="div"` when the bar sits inside a landmark that already has a header.
