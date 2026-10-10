/** Real asynchronous and timer effects, described beside their complete SFCs. */
export const asyncEffectExamples = [
  {
    name: "use-interval",
    title: "Time a draft review",
    purpose: "Track a review session with a pausable one-second clock and a reset control.",
    observe:
      "Start the timer, wait for its count to rise, then pause. Paused time does not count. Reset changes the count without changing whether the timer is running.",
    context:
      "With `controls: true`, useInterval returns counter, pause/resume, and reset. `immediate: false` gives SSR and the first client render the same paused zero state. The component scope owns and clears the real interval; this is a tick counter, not an elapsed wall-clock measurement.",
  },
  {
    name: "use-timeout",
    title: "Schedule a review reminder",
    purpose: "Schedule one delayed action, replace its deadline, or cancel it before it runs.",
    observe:
      "Schedule a reminder and cancel it before one second passes. Its delivered count stays unchanged. Schedule again to see the callback run once. Scheduling while pending restarts the one-second deadline.",
    context:
      "This entry exports useTimeoutFn as well as useTimeout. The example uses useTimeoutFn with `immediate: false`; no timer starts during SSR. Pending state belongs to the composable, while the callback owns the message and delivery count. Unmounting clears a pending timer.",
  },
  {
    name: "use-async-state",
    title: "Load and recover a team profile",
    purpose:
      "Connect asynchronous sample data to loading, failure, retry, and successful result states without stale results overwriting a newer request.",
    observe:
      "Try the missing profile for an error, then Load Ada to recover. Start Load Ada and immediately Load Grace: Grace finishes first, and Ada's later completion must not replace it.",
    context:
      "The producer uses local delayed sample data and sends no network request. `immediate: false` starts work only after an explicit action; `resetOnExecute: true` clears the displayed profile while loading. `isReady` means a request has succeeded at least once. useAsyncState ignores superseded completions; the producer remains responsible for canceling external work, so this SFC clears its own simulation timers on disposal.",
  },
] as const;
