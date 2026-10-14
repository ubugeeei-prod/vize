// Webpack style blocks become virtual files beside the SFC, e.g.
// `App.vue.__vize_style_0.data-v-1a2b3c4d.scss`, so the host's own `.scss`
// rule (sass-loader, css-loader, ...) matches them and relative `@use` and
// `@import` resolve from the SFC's directory.
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Compiler as WebpackCompiler } from "webpack";
import type { StyleBlockInfo } from "./types.ts";

const SCOPED_STYLE_FILE = /\.__vize_style_\d+\.(data-v-[0-9a-f]+)(?:\.module)?\.\w+$/;

interface VirtualModules {
  writeModule(filePath: string, contents: string): void;
}

export function webpackStylePath(
  filePath: string,
  block: StyleBlockInfo,
  scoped: string | null,
): string {
  const scope = scoped ? `.${scoped}` : "";
  const module = block.module === false ? "" : ".module";
  return `${filePath}.__vize_style_${block.index}${scope}${module}.${block.lang ?? "css"}`;
}

export function webpackStyleScope(resourcePath: string): string | null {
  return SCOPED_STYLE_FILE.exec(resourcePath)?.[1] ?? null;
}

export function findVirtualModules(compiler: WebpackCompiler): VirtualModules | undefined {
  return compiler.options.plugins.find(
    (plugin): plugin is VirtualModules & typeof plugin =>
      typeof (plugin as Partial<VirtualModules> | null)?.writeModule === "function",
  );
}

/** Scope styles after the host's preprocessor loader; see webpack-style-loader.ts. */
export function registerWebpackStyleRule(compiler: WebpackCompiler): void {
  const self = fileURLToPath(import.meta.url);
  compiler.options.module?.rules?.unshift({
    test: SCOPED_STYLE_FILE,
    loader: path.join(path.dirname(self), `webpack-style-loader${path.extname(self)}`),
  });
}
