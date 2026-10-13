/** Public-import examples for the actual preview viewport and media queries. */
export const environmentExamples = [
  {
    name: "breakpoints",
    title: "Adapt guide detail to actual viewport breakpoints",
    purpose:
      "Choose automatic guide detail from native media queries while preserving explicit user choices and a local review draft.",
    observe:
      "Resize the preview across 480 and 768 CSS pixels. Matched lists every reached named breakpoint in ascending order; Active reports the largest. At exactly 768, At least and At most both match while Above and Below do not. The middle range includes 480 and excludes 768. Choose Full detail or Summary to override automatic detail without changing any readonly query result. The draft survives resizing.",
    context:
      "useBreakpoints receives numeric CSS-pixel thresholds and uses the default real matchMedia capability. Named refs are mobile-first greater-or-equal results; current and active are readonly computed views. Comparison helpers cache their underlying queries. Strict greater/smaller use 0.02 px offsets, demonstrated here at integer viewport boundaries. Without ssrWidth, every query is false on the server, including smaller and smallerOrEqual; current is empty and active is null. The first browser render retains that fallback, then actual media results apply. An explicit ssrWidth would be an application-supplied assumption, not a measured client width, and must be consistent between server and client. This iframe's viewport can differ from the surrounding documentation page. Native select and draft state belong to the example. Scope disposal removes the media subscriptions; no host or media result is simulated.",
  },
  {
    name: "window-size",
    title: "Lay out review cards using this viewport's real size",
    purpose:
      "Read actual window dimensions and support without letting responsive changes overwrite the user's layout choice or draft.",
    observe:
      "Resize the preview and compare the readonly width and height with its viewport. Automatic uses two columns from 640 CSS pixels; Single column keeps one at every width. Toggle Show layout measurements and edit the draft, then resize again: both choices remain. The measurement label always reports actual dimensions independently of the layout override.",
    context:
      "useWindowSize() exposes readonly width, height, and isSupported refs. Default inner dimensions include scrollbars; includeScrollbar: false would use the document root's client dimensions, and type: outer would measure the outer browser window. This example uses the default real iframe window and no injected host. During SSR and the first hydrating render, dimensions are 0 by default and support is false; mounting attaches the actual window and reads its size. Resize and orientationchange then refresh it. initialWidth and initialHeight can supply a consistent application-chosen SSR fallback, not a detected browser size. The native select, checkbox, and draft own separate local state. Scope disposal removes passive listeners and the host watcher. The surrounding page and this iframe need not have the same dimensions.",
  },
] as const;
