/** Complete public-import examples using actual clipboard and element geometry. */
export const observedBrowserExamples = [
  {
    name: "use-clipboard",
    title: "Copy and read an editable install command",
    purpose:
      "Report the outcome of an explicit clipboard operation while keeping the last successful text and time-limited copy feedback separate.",
    observe:
      "Edit Install command, then activate Copy install command. The readout reports the actual mechanism and recent-copy feedback expires after 1.5 seconds. Read clipboard text requests a read only when activated. If your browser refuses permission, the failure is displayed while the last successful text remains.",
    context:
      "This example uses the real default browser host; it never supplies a fake clipboard. Copy writes to your clipboard. The async Clipboard API normally requires a secure context and browser permission; legacy copying may still be available where reading is not. supported means at least one copying mechanism exists, not that reading is permitted. Every action resolves to a discriminated result instead of rejecting. A later success clears the last error; reading does not restart the copied timer. During SSR no clipboard or permission is read, text is empty, and support is false. The mounted host resolves after the initial hydration render. Scope disposal owns the copied reset timer.",
  },
  {
    name: "scroll",
    title: "Navigate a scrollable preparation guide",
    purpose:
      "Connect native scrolling, writable vertical offset, edge state, and keyboard controls to one bounded guide preview.",
    observe:
      "Scroll the read-only preparation preview, or activate Next guide section. Its actual vertical offset changes and Previous becomes available. Continue to the end: Next disables when the element reaches its bottom. Return to the beginning resets the offset and disables Previous. The surrounding document is not the observed target.",
    context:
      "useScroll receives the real HTMLTextAreaElement template ref. Assigning y and calling scrollTo move that element; its real client and scroll dimensions determine arrivedState. Native scroll events update activity, which settles on scrollend or the 200 ms idle fallback. Read-only text still supports ordinary selection and keyboard scrolling. SSR starts at zero with the top/left edges arrived, no bottom/right arrival, and no browser listeners. Client attachment follows mounted state; scope disposal removes listeners and the idle timer. These controls navigate local content and do not request or save data.",
  },
  {
    name: "resize-observer",
    title: "Observe the size of a selected guide panel",
    purpose:
      "Report the native content-box measurement of one panel and follow a changing target without observing the other panel.",
    observe:
      "Toggle Wide first panel to change its real CSS size and delivered measurement. Resizing the second panel leaves the first panel's readout alone. Choose Second guide panel to retarget the observer. Stop size observation freezes the last delivered measurement while both native checkboxes keep resizing their panels.",
    context:
      "A computed template target selects one actual section. The native callback reads contentRect and labels its target; its notification count counts deliveries, not every layout change. Border and padding are excluded from the reported content box. Retargeting disconnects the previous observer, and stop permanently stops its watcher and observer. A callback entry from an earlier target is ignored by this example. During SSR there is no host, support is false, and no measurement is taken. The browser host resolves after mounting, retaining the initial hydration markup. Scope disposal also disconnects observation. Panel classes and responsive maximum width remain ordinary CSS.",
  },
  {
    name: "intersection-observer",
    title: "Observe a featured guide inside its own viewport",
    purpose:
      "Measure whether a featured card crosses a real 60 percent visibility threshold and expose pause, resume, and permanent stop.",
    observe:
      "Show featured guide scrolls the actual bounded viewport to the card. Once at least 60 percent is visible, the readout and card highlight change. Pause, return to preparation, then Resume: the resumed observer measures the current hidden position. Stop retains the last recorded visibility while the native scrolling controls continue working.",
    context:
      "The template refs supply a real ancestor root and article target to the browser's IntersectionObserver. Native entries supply intersectionRatio and isIntersecting; the example applies the 0.6 threshold to its highlight. isActive describes requested observation, not a guarantee that an observer is connected. The callback explicitly ignores entries while inactive so queued delivery does not overwrite this example's paused readout. Pause disconnects and permits resume; stop permanently disables reconnecting. During SSR support is false and no geometry is read. Mounted targets resolve on the client without replacing the initial SSR markup or elements. Scope disposal disconnects the observer. No synthetic observer host is used.",
  },
] as const;
