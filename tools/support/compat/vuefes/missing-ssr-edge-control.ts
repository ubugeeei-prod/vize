import type {
  BabelParse,
  BabelStatement,
  FixturePlugin,
  ModuleTransformOptions,
} from "./oracle-support.ts";

export interface MissingEdge {
  readonly id: string;
  readonly original: string;
  readonly altered: string;
  readonly removed: readonly string[];
}

const isStyleEdge = (
  node: BabelStatement,
): node is BabelStatement & { source: { value: string } } =>
  node.type === "ImportDeclaration" &&
  node.specifiers?.length === 0 &&
  node.source !== undefined &&
  node.source.value.includes(".__vize_style_");

export function createMissingSsrEdgeControl({
  producer,
  missingEdges,
  parse,
}: {
  producer: string;
  missingEdges: MissingEdge[];
  parse: BabelParse;
}): FixturePlugin {
  return {
    name: "fixture:missing-ssr-style-edge",
    enforce: "post",
    transform(code: string, id: string, options?: ModuleTransformOptions) {
      if (
        producer !== "native-missing-ssr-edge" ||
        !options?.ssr ||
        !id.includes("VFHeading.vue") ||
        !code.includes(".__vize_style_")
      )
        return undefined;
      const imports = parse(code, {
        sourceType: "module",
        plugins: ["typescript"],
      }).program.body.filter(isStyleEdge);
      if (!imports.length) return undefined;
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
