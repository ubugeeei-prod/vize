/** Browser capability examples use real storage, media-query, and document hosts. */
export const browserEffectExamples = [
  {
    name: "use-storage",
    title: "Remember reading progress",
    purpose:
      "Persist a selected guide section in the current browser and provide an explicit way to remove the saved choice.",
    observe:
      "Choose Components, then reload this page to keep that choice. Reset reading progress removes the saved key and restores Getting started, including after another reload.",
    context:
      "`state` is writable; `supported` and `error` describe the real localStorage capability. A validator accepts only the three guide sections. `writeDefaults: false` leaves an absent or removed key absent. SSR uses Getting started without accessing storage; the default post-flush initial read preserves hydration before restoring a saved choice. When storage is unavailable, choices still update in this tab but reload returns to the default. Storage can fail in restricted browsers. The key belongs to this example's origin; an embedded frame may have a different origin or storage policy from its parent. Watches and storage listeners follow the component scope.",
  },
  {
    name: "media-query",
    title: "Adapt recommended guides to available space",
    purpose:
      "Use an actual media-query result to switch a guide list between one and two columns, with a reactive breakpoint selector.",
    observe:
      "Resize the window across 600 pixels: the layout and visible match state change together. At 760 pixels, selecting 800 pixels switches to Compact; selecting 600 pixels restores Roomy. An iframe evaluates its own width.",
    context:
      "The query getter follows the writable breakpoint ref. `useMediaQuery` returns a readonly boolean ref and uses the browser's matchMedia change events. `ssrValue: false` renders Compact before the browser's first post-render evaluation, so hydration starts from the same markup. Query changes replace the previous subscription, and component disposal removes the final listener. This example changes layout without animation or a simulated host.",
  },
  {
    name: "active-element",
    title: "Show guidance for the focused profile field",
    purpose:
      "Follow actual document focus to show contextual instructions for a small workshop profile form.",
    observe:
      "Press Tab from Display name through Email address and Workshop note. The visible focused-field label and guidance follow each field. Focus display name moves focus back; Clear focus returns to the outside-field state.",
    context:
      "`useActiveElement` returns a readonly shallow ref of the real focused Element, with null during SSR. It reads the document only after mounting and listens for capturing focus/blur events. The example recognizes only its own fields through useId-based owner markers. Other controls or a blurred field show the outside-field state. The default follows open shadow roots; this example uses native inputs. Document listeners are released with the component scope.",
  },
] as const;
