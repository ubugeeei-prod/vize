/** Complete public-import examples for scoped browser interactions. */
export const domInteractionExamples = [
  {
    name: "focus",
    title: "Track and control workshop profile focus",
    purpose:
      "Distinguish one field's focus from focus anywhere in its group, and control that field through a writable focus ref.",
    observe:
      "Focus badge name moves actual browser focus into the name field. Escape in that field assigns false to blur it. Tab to Workshop contact: name focus becomes no while profile focus stays yes. Tab outside the group clears both flags. Stop focus tracking freezes the last observed flags and disables the focus controls; the fields remain editable.",
    context:
      "`useFocus` returns a writable computed ref: assigning true calls the target element's focus(), and false calls blur(). `useFocusWithin` observes focusin/focusout across descendants and retains true when focus moves inside the group. Template refs resolve only on the client; both flags start false during SSR. No initial autofocus is requested. Both stop functions remove their listeners and freeze their recorded flags, so the stopped output is explicitly labelled Last observed. The component scope also disposes both watchers. This does not trap focus or validate the profile fields.",
  },
  {
    name: "on-click-outside",
    title: "Dismiss inline guide preferences",
    purpose:
      "Keep an expandable preference region open during inside and ignored clicks, then dismiss it when an actual outside click occurs.",
    observe:
      "Open the preferences and edit Review note. Apply local preference leaves the region open and reports the note. Keep preferences open is outside but ignored. Continue reading outside dismisses the region and increments the counter. Reopen it and press Escape inside: the example closes it, returns focus to its trigger, and leaves the outside-dismissal count unchanged.",
    context:
      "The target is a template ref recreated by v-if; the trigger and separate keep-open button are ignored element targets. The composable tracks pointerdown as well as click, so a drag starting inside and ending outside does not dismiss it. Native keyboard activation of an outside button also emits a click. This is an inline non-modal region with ordinary Tab order; Escape and trigger focus are explicit example behavior, not features supplied by onClickOutside. The default window host is absent during SSR and scope disposal removes the client listeners. Applying a note changes only local display state.",
  },
  {
    name: "on-key-stroke",
    title: "Scope review shortcuts to a chosen control",
    purpose:
      "Navigate a bounded review checklist with element-scoped shortcuts while retaining native controls for touch and other keyboard users.",
    observe:
      "Focus Primary review control and press J/K to move through the tips; ? toggles help. Holding J performs only the first action. Choose Secondary review control and verify that the shortcuts move there while the primary control no longer responds. Stop review shortcuts removes the listener; the native Previous/Next buttons still work.",
    context:
      "`target` is a computed native-button ref, so the listener follows the selected element and detaches from the previous one. Key names are normalized; the handler interprets J/K and cancels only matched shortcuts. `dedupe: true` ignores repeated keydown events. The returned stop function permanently stops this registration, which the scope also owns. A null template target attaches nothing during SSR. Bounded tip state and native Previous/Next buttons belong to this example; the composable supplies only matching event delivery. Shortcuts are not registered on the document or window.",
  },
  {
    name: "event-listener",
    title: "Observe input from a selected workshop draft",
    purpose:
      "Follow native input events from one editable draft, retarget the listener, and pause or resume observation without locking either field.",
    observe:
      "Edit First workshop draft to update the event count and last observed text. Editing the other field does not update the monitor. Choose Second workshop draft to move the listener. Pause, edit either draft, switch the observed draft, then Resume: only new input from the current target is observed.",
    context:
      "`useEventListener` receives a computed HTMLInputElement target and a typed input handler. Retargeting removes the old listener. stop() removes the watcher and listener; start() restores observation of the current target, and repeated active starts are harmless. Each v-model remains independently writable. The monitor reads currentTarget.value from the real event and sends no request or save operation. Inside a component, attachment waits for mounted state, keeping isListening false during SSR and the initial hydration render. The owning scope removes the watcher and listener on disposal.",
  },
] as const;
