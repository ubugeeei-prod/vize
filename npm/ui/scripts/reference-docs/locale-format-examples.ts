/** Complete public-import examples for locale-aware number, list, and plural formatting. */
export const localeFormatExamples = [
  {
    name: "use-number-format",
    title: "Show an order total in the reader's format",
    purpose:
      "Format a currency amount with the separators, symbol position, and decimals a reader expects in their locale.",
    observe:
      "The total starts as €1,234.50. Choose German (Germany) to see 1.234,50 €, then change the amount to 99 to see 99,00 €. Japanese (Japan) keeps the euro symbol in front.",
    context:
      "`options` is a computed ref, so changing the locale rebuilds the cached `Intl.NumberFormat`. Pass `locale` explicitly when rendering on the server; without it the browser language is detected and hydration can differ.",
  },
  {
    name: "use-list-format",
    title: "Announce a list of speakers",
    purpose:
      "Join a changing list of names into one sentence with the conjunction and punctuation of the reader's language.",
    observe:
      "The speakers read Ada, Grace, and Linus. Choose German (Germany) for Ada, Grace und Linus, and Japanese (Japan) for Ada、Grace、Linus. Add and remove speakers to see one, two, and many names.",
    context:
      'The list is a ref that is reassigned, not mutated in place, so the formatted text updates. `type: "conjunction"` gives “and” lists; use `"disjunction"` for “or”. The empty state is handled in the template.',
  },
  {
    name: "use-plural-rules",
    title: "Word an unread-message count",
    purpose:
      "Choose the right message for zero, one, and many items, using the locale's plural rules rather than `count === 1`.",
    observe:
      "Zero shows No unread messages from the exact `=0` message. Enter 1 for 1 unread message and 2 for 2 unread messages. The plural category below the message shows which rule matched.",
    context:
      "Messages are keyed by CLDR category, with optional exact `=N` overrides; `other` is required. A function message receives the count and its locale-formatted text. Languages with more categories, such as `few` or `many`, need those keys.",
  },
] as const;
