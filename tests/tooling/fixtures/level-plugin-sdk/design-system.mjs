// Team convention #2, the sweep-shaped rule the spike measured with: our
// design system wraps every button, so a raw <button> is a review comment.
// It reads one property of every element — the shape where per-read napi
// crossings (the proxy arm) cost the most.
import { definePlugin } from "./sdk.mjs";

export default definePlugin({
  name: "design-system",
  version: "1.0.0",
  visit: ["ui.element"],
  rules: {
    "use-base-button"(ctx) {
      for (const element of ctx.nodes) {
        if (element.name === "button") {
          ctx.report(element, "Use <BaseButton> from the design system instead of a raw <button>.");
        }
      }
    },
  },
});
