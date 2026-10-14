/** Complete public-import examples for countdowns and relative dates. */
export const timeFormatExamples = [
  {
    name: "use-countdown",
    title: "Wait before resending a sign-in code",
    purpose:
      "Count down a cooldown after an action, keep the button disabled until it ends, and allow another request afterwards.",
    observe:
      "Send code starts a 30-second cooldown and disables the button while the remaining seconds count down. When it reaches zero the button reads Resend code again and the message counts the codes sent.",
    context:
      "`start()` resets to the initial count and begins ticking; `isActive` is true while the timer runs. Call `useCountdown` inside setup so its interval is cleared with the component. Pass `intervalMs` for a different tick length.",
  },
  {
    name: "use-relative-time-format",
    title: "Say when a meetup happens",
    purpose:
      "Describe a date relative to today in the reader's language, using words such as tomorrow instead of in 1 day.",
    observe:
      "The meetup is in 3 days. Change the days to 1, 0 and -1 for tomorrow, today and yesterday. Choose German (Germany) for in 3 Tagen or Japanese (Japan) for 3 日後.",
    context:
      '`numeric: "auto"` lets the formatter use words for nearby values. The value and unit can be refs; pass `locale` explicitly when rendering on the server. `selectRelativeTimeUnit()` picks a unit from a millisecond difference.',
  },
] as const;
