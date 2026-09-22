// Team convention #3 (every vue-i18n team): no hard-coded copy in templates,
// visible text goes through $t(). Code samples are exempt. The read-heavy
// shape the spike measured: two reads per text node plus an ancestor walk.
import { definePlugin } from "./sdk.mjs";

export default definePlugin({
  name: "i18n",
  version: "1.0.0",
  visit: ["ui.text", "ui.element"],
  rules: {
    "no-raw-text"(ctx) {
      for (const text of ctx.nodes) {
        if (text.kind !== "ui.text" || !/\p{L}/u.test(text.value)) continue;
        const inCode = (el) => el.name === "code" || el.name === "pre";
        if ([...ctx.ancestors(text, "ui.element")].some(inCode)) continue;
        ctx.report(
          text,
          `Hard-coded text "${text.value.trim()}": use $t() so it can be translated.`,
        );
      }
    },
  },
});
