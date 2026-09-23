// A team convention every Vue team has argued about, as a 20-line JS rule:
// never key a v-for by its index — reorder or delete a row and Vue reuses
// the wrong component state. Scope-correct: the key is resolved through the
// enclosing v-for scopes innermost-first, so `(cell, j)` inside `(row, i)`
// still catches `:key="i"`, and a shadowing loop variable named `index` is
// not an index.
import { definePlugin } from "./sdk.mjs";

export default definePlugin({
  name: "team-conventions",
  version: "1.0.0",
  visit: ["ui.for", "ui.bind"],
  demands: ["templateScopes"],
  cacheInputs: [],
  rules: {
    "no-index-key"(ctx) {
      const scopes = ctx.facts("templateScopes");
      for (const bind of ctx.nodes) {
        if (bind.kind !== "ui.bind" || bind.name !== "key") continue;
        const name = bind.value?.trim();
        const loops = [...ctx.ancestors(bind, "ui.for")];
        const bound = (loop) => scopes.get(loop.id)?.find((entry) => entry.name === name);
        const owner = loops.find(bound);
        // `(item, i)`: S2 names the second alias `key` (Vue's grammar); on
        // an array it is the index. Only the item itself is stable.
        if (owner && bound(owner).position !== "value") {
          const item = loops[0].alias.value;
          ctx.report(
            bind,
            `Don't key a v-for by its index \`${name}\`; use a stable id such as \`${item}.id\`.`,
          );
        }
      }
    },
  },
});
