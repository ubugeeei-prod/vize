import { compileCss } from "@vizejs/native";
import { SourceMapSource } from "webpack-sources";
import { parseSourceMap } from "@vizejs/builder-shared/mapped-module";
import type { LoaderContext } from "./types.ts";

export default function vizeScopeLoader(
  this: LoaderContext<{ sourceMap?: boolean }>, source: string, inputMap?: object | string,
): void {
  const callback = this.async();
  try {
    const scopeId = new URLSearchParams(this.resourceQuery.slice(1)).get("scoped");
    if (!scopeId) { callback(null, source, inputMap); return; }
    if (!/^data-v-[a-f0-9]+$/.test(scopeId)) throw new Error("[vize] Invalid style scope identity.");
    const result = compileCss(source, { filename: this.resourcePath, scoped: true, scopeId, sourceMap: this.getOptions().sourceMap ?? true });
    if (result.errors.length) throw new Error(`[vize] ${this.resourcePath}:\n${result.errors.join("\n")}`);
    for (const warning of result.warnings) this.emitWarning(new Error(`[vize] ${warning}`));
    const map = parseSourceMap(result.map);
    const composed = map && inputMap ? new SourceMapSource(result.code, this.resourcePath, JSON.stringify(map), source, typeof inputMap === "string" ? inputMap : JSON.stringify(inputMap), true).map({ columns: true }) : map;
    callback(null, result.code, composed);
  } catch (error) { callback(error instanceof Error ? error : new Error(String(error))); }
}
