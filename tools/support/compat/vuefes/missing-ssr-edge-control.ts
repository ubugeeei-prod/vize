export function createMissingSsrEdgeControl({ producer, missingEdges, parse }) {
  return {
    name: "fixture:missing-ssr-style-edge",
    enforce: "post",
    transform(code, id, options) {
      if (
        producer !== "native-missing-ssr-edge" ||
        !options?.ssr ||
        !id.includes("VFHeading.vue") ||
        !code.includes(".__vize_style_")
      )
        return;
      const imports = parse(code, {
        sourceType: "module",
        plugins: ["typescript"],
      }).program.body.filter(
        (node) =>
          node.type === "ImportDeclaration" &&
          node.specifiers.length === 0 &&
          node.source.value.includes(".__vize_style_"),
      );
      if (!imports.length) return;
      let altered = code;
      for (const node of [...imports].reverse())
        altered = altered.slice(0, node.start) + altered.slice(node.end);
      missingEdges.push({
        id,
        original: code,
        altered,
        removed: imports.map((node) => node.source.value),
      });
      return { code: altered, map: null };
    },
  };
}
