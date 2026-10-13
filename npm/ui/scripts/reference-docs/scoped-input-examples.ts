/** Complete public-import examples for native focus and input guidance. */
export const scopedInputExamples = [
  {
    name: "element-bounding",
    title: "Measure a guide panel's actual border rectangle",
    purpose:
      "Observe actual viewport-relative geometry, explicitly refresh a transformed position, and distinguish missing or stopped targets from current measurements.",
    observe:
      "Toggle Wide guide panel and compare its actual border-box size. Shift guide panel changes CSS position without resizing; Measure current rectangle explicitly refreshes that position. Hide the panel to reset all values to zero, then show it again. Stop rectangle observation freezes the labelled last-recorded rectangle while all panel checkboxes keep working.",
    context:
      "useElementBounding exposes readonly refs from the actual getBoundingClientRect, including border and padding, relative to this preview's viewport. Template resolution, native ResizeObserver delivery, window resize, and captured ancestor scrolling refresh the geometry. A transform does not resize the target, so the example offers the public update action explicitly. Default reset clears all values when the target disappears. stop removes automatic observation; update remains a callable API, but this example disables its button to retain an explicitly last-recorded snapshot. SSR values are zero and no target geometry is read. Scope disposal removes its observer, target watcher, and listeners. No geometry host is simulated.",
  },
  {
    name: "element-hover",
    title: "Show delayed guidance for a selected card",
    purpose:
      "Delay pointer guidance for a real reactive element while keeping the advice independently available to keyboard and tap users.",
    observe:
      "Move over the selected guide card for 300 ms to reveal the advice. Departure retains it for 200 ms; returning cancels the pending departure. Choose the other card to remove the previous target's listeners and reset the delayed flag. Show persistent guidance keeps the same advice visible without hovering.",
    context:
      "useElementHover receives a computed native section target, with explicit delayEnter and delayLeave. pointerenter/pointerleave ignore movement among descendants. Opposite events cancel pending delay, and reactive target replacement removes old listeners, cancels its timer, and resets the hover flag. The output deliberately says Delayed pointer hover rather than instantaneous pointer position. The persistent guidance button owns separate state and works with Enter, Space, or ordinary tapping. SSR starts false and no target exists; the template refs resolve on the client. Scope disposal removes listeners and pending timers. No advice depends exclusively on hover.",
  },
  {
    name: "on-long-press",
    title: "Reveal a review note before releasing a held button",
    purpose:
      "Deliver a bounded long-press action through real pointer events while retaining ordinary native button activation for the same note.",
    observe:
      "Hold Reveal review note for 500 ms: it opens before release and the long-press counter increments once. An early release or movement beyond 10 pixels cancels that delivery. An ordinary click, tap, or Enter still reveals the note independently. Stop long-press observation removes held actions; the native reveal and hide buttons continue working.",
    context:
      "onLongPress watches the real native button ref and receives the initiating PointerEvent. The example records its pointerType and native release summary separately from the ordinary click handler. Only the timed handler increments the long-press count. Its release duration comes from the real default Date.now clock; distance is measured from the initiating coordinates. Moving beyond the threshold, leaving, cancelling, or releasing clears pending delivery, and stop removes listeners and the timer. The last release summary stays explicitly last-observed after stop. During SSR there is no target and no timer starts. The scope also disposes the registration. No custom pointer host or scheduler is supplied.",
  },
  {
    name: "on-start-typing",
    title: "Focus local guide search when typing begins",
    purpose:
      "Redirect a printable key from a non-editable reading control into search while respecting native field editing and existing modifier shortcuts.",
    observe:
      "Focus Reading control and press a letter: search receives actual focus and the shortcut delivery count increments. Then type to filter the local guides. Space activates the reading button normally, navigation keys do not redirect, and Ctrl/Meta/Alt shortcuts are ignored. Editing Workshop note does not focus search. Stop typing shortcut removes the redirect while both fields remain usable.",
    context:
      "The default host is this preview's document. onStartTyping matches single printable characters other than space, with no Ctrl/Meta/Alt modifier and no editable element focused. The handler only counts and focuses the actual search input; v-model and the local computed filter own ordinary text editing. The document listener is passive, so the handler does not call preventDefault. The returned stop function permanently removes its watcher/listener; component scope disposal does the same. SSR has no document host and installs nothing. Search and note values are local and no request or save is sent. This is not a global shortcut on the parent documentation page.",
  },
] as const;
