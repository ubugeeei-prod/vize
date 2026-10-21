const historicalRequireScopedStyleMessage =
  "[vize:vue/require-scoped-style] Style block should use `scoped` or `module` attribute to prevent CSS leaking";
const localizedRequireScopedStyleMessage =
  "[vize:vue/require-scoped-style] Style blocks should use `scoped` or `module` attribute to prevent CSS leaking";
const historicalRequireScopedStyleHelp = "Add scoped attribute: <style scoped>";
const localizedRequireScopedStyleHelp =
  "Add `scoped` or `module` to the style block: `<style scoped>` or `<style module>`";

export function reviewLocalizedRequireScopedStyle(raw: string): {
  raw: string;
  reviewedMessages: number;
} {
  let reviewedMessages = 0;
  return {
    raw: raw
      .replaceAll(historicalRequireScopedStyleMessage, () => {
        reviewedMessages++;
        return localizedRequireScopedStyleMessage;
      })
      .replaceAll(historicalRequireScopedStyleHelp, localizedRequireScopedStyleHelp),
    reviewedMessages,
  };
}
