const historicalRequireScopedStyleMessage =
  "[vize:vue/require-scoped-style] Style block should use `scoped` or `module` attribute to prevent CSS leaking";
const localizedRequireScopedStyleMessage =
  "[vize:vue/require-scoped-style] Style blocks should use scoped attribute to prevent style leaking";

export function reviewLocalizedRequireScopedStyle(raw: string): {
  raw: string;
  reviewedMessages: number;
} {
  let reviewedMessages = 0;
  return {
    raw: raw.replaceAll(historicalRequireScopedStyleMessage, () => {
      reviewedMessages++;
      return localizedRequireScopedStyleMessage;
    }),
    reviewedMessages,
  };
}
