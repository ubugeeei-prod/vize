/** Complete public-import examples for explicit workflow states and value history. */
export const workflowStateExamples = [
  {
    name: "use-machine",
    title: "Walk through a guarded checkout",
    purpose:
      "Model a checkout as named steps so each screen only offers the actions that are valid there.",
    observe:
      "The cart starts empty and only offers Add item. Add one item to unlock Go to shipping, then continue to Review order and Place order. Back returns one step; Start a new order empties the cart.",
    context:
      "`nextEvents` lists the events the current state accepts and whose guards pass, so the buttons follow the machine. `add` is a self-transition whose `action` returns the next context. `send()` returns false for an event the state does not accept.",
  },
  {
    name: "use-previous",
    title: "Show how a ticket price moved",
    purpose:
      "Compare a value with the one it replaced to describe a change, without keeping your own history.",
    observe:
      "Before any edit there is no earlier price. Change 120 to 150 to see Up 30 from 120, then to 90 to see Down 60 from 150. Entering the same price again reports it as unchanged.",
    context:
      "`usePrevious` returns a readonly ref that holds the value the source had before its last change, starting as `undefined` unless you pass an initial value. It updates synchronously, so the comparison is ready in the same render.",
  },
] as const;
