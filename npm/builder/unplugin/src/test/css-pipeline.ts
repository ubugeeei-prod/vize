import type { Plugin } from "rollup";

/**
 * A stand-in host CSS pipeline for Rollup tests, as rollup-plugin-postcss
 * would be: stylesheet modules become one emitted `style.css`.
 */
export function cssPipeline(): Plugin & { css(): string } {
  const styles = new Map<string, string>();
  const css = () => [...styles.values()].join("\n");
  return {
    name: "test-css-pipeline",
    css,
    buildStart() {
      styles.clear();
    },
    transform(code, id) {
      if (!id.endsWith(".css")) return null;
      styles.set(id, code);
      return { code: "", map: null, moduleSideEffects: "no-treeshake" };
    },
    generateBundle() {
      this.emitFile({ type: "asset", fileName: "style.css", source: css() });
    },
  };
}
