/** Complete public-import examples for form, navigation, selection, and list state. */
export const formStateExamples = [
  {
    name: "use-form",
    title: "Validate a workshop registration preview",
    purpose:
      "Validate two related fields together and create a local preview only after both validators pass.",
    observe:
      "Enter Al and an incomplete email. Errors wait until Preview registration; editing after that attempt revalidates each field. Use Ada and ada@example.test to create the preview. Reset registration clears inputs, errors, the submit count, and the preview.",
    context:
      "Destructure field refs before binding them with v-model. `handleSubmit` prevents native form submission, validates every field, and calls the local preview callback only when valid. `novalidate` lets the composable's recorded errors explain these example rules. `valid` means no errors are currently recorded, so this example does not treat an untouched form as validated. The submit count includes rejected attempts. `reset()` restores initial form values and state; the example separately clears its local preview. No request or registration is sent. Setup and SSR start with unvalidated values; owning-scope disposal aborts pending validation.",
  },
  {
    name: "use-stepper",
    title: "Review a registration in editable steps",
    purpose:
      "Move through attendee, session, and review screens while retaining the values needed when an earlier step is edited.",
    observe:
      "Edit the attendee, choose a session, and continue to Review. Next disables on the last step. Choose Attendee in the step list to revise the name, then return through Session; both values remain and the review reflects the edit.",
    context:
      "`index` is zero-based and readonly; `current` is the typed step name. Navigation changes only the cursor. The name and session are independent writable Vue refs, so conditional screens retain their data. `goBackTo` only moves to an earlier step; the example disables later step buttons. Previous/Next controls expose the first/last boundaries and the step list marks its current item with aria-current. This example navigates and reviews local values without validation or submission. All state is synchronous and SSR-safe; no browser capability or cleanup is required.",
  },
  {
    name: "use-selection",
    title: "Choose workshop sessions with stable identities",
    purpose:
      "Select up to two available sessions with native checkboxes, keeping the selected items accurate when catalogue objects change.",
    observe:
      "Use Space on a checkbox, then choose a second session. The full workshop and further choices are disabled. Clear session choices restores an empty selection. Select available sessions fills only the two-place limit. Revise compiler title replaces all catalogue objects while keeping the selected stable ids and showing the updated title.",
    context:
      "`multiple: true` makes `selected` an array; getKey gives each item a stable id. The composable's `max` and `isSelectable` constrain select/selectAll, and the example mirrors those constraints in native disabled states. `selected` follows catalogue order, while `selectedKeys` preserves selection order. `clear()` removes choices without changing catalogue data. Title revision is an explicit local immutable update, not a simulated server response. This synchronous selection state needs no browser globals or cleanup and renders the same empty state during SSR.",
  },
  {
    name: "use-cycle-list",
    title: "Explore component review tips",
    purpose:
      "Cycle through useful review content with wrapping previous/next controls and a direct-choice selector.",
    observe:
      "Previous tip from the first item wraps to Handle the empty state; Next tip wraps back to Name the props. Choose Label the controls directly, then continue to the next tip. The title, explanation, selector, and visible position stay synchronized.",
    context:
      "A non-empty tuple gives `state` a defined item type. `index` is a writable computed ref; v-model.number moves the cursor through it. `next()` and `prev()` accept optional step counts, so explicit arrows keep native events out of those arguments. Navigation wraps modulo list length. Reactive list changes retain an existing current item or choose the configured fallback; the scope owns that watcher. This fixed checklist starts synchronously with its first tip during SSR and requires no browser host.",
  },
] as const;
