import { defineTransformPlugin } from "../../../../../npm/plugin-sdk/index.js";

export default defineTransformPlugin({
  name: "design-system",
  version: "1.0.0",
  cacheInputs: [{ name: "class-migration", value: "legacy-btn -> ds-button" }],
  transform(batch) {
    return batch.nodes
      .filter((node) =>
        node.attrs.some((attr) => attr.name === "class" && attr.value === "legacy-btn"),
      )
      .map((node) => ({
        kind: "replace-static-attribute",
        node: node.id,
        name: "class",
        value: "ds-button",
      }));
  },
});
