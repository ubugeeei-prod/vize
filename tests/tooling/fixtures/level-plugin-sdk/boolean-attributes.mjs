// Constant true native boolean props can use their equivalent literal spelling.
import { definePlugin } from "../../../../npm/plugin-sdk/index.js";

const BOOLEAN = new Set([
  "disabled",
  "required",
  "multiple",
  "checked",
  "selected",
  "readonly",
  "autofocus",
]);
export default definePlugin({
  name: "boolean-attributes",
  version: "1.0.0",
  visit: ["ui.element", "ui.bind"],
  demands: [],
  cacheInputs: [{ name: "boolean-attributes", value: JSON.stringify([...BOOLEAN]) }],
  rules: {
    "prefer-boolean-shorthand"(ctx) {
      for (const bind of ctx.nodes) {
        if (bind.kind !== "ui.bind" || bind.value?.trim() !== "true" || !BOOLEAN.has(bind.name))
          continue;
        const owner = ctx.ancestors(bind, "ui.element").next().value;
        if (!owner || bind.parent !== owner.id) continue;
        ctx.report(bind, `Use the equivalent literal ${bind.name} attribute.`, bind.name);
      }
    },
  },
});
