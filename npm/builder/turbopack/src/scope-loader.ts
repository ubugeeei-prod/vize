import path from "node:path";
import { scopeViteCssForPipeline, transformViteCssVarsForPipeline } from "@vizejs/native";
import sources from "webpack-sources";
import { MappedModule, parseSourceMap } from "@vizejs/builder-shared/mapped-module";
import type { LoaderContext } from "./types.ts";

export default function vizeScopeLoader(
  this: LoaderContext<{ root?: string; sourceMap?: boolean }>,
  source: string,
  inputMap?: object | string,
): void {
  const callback = this.async();
  try {
    const query = new URLSearchParams(this.resourceQuery.slice(1));
    const scopeId = query.get("scoped");
    if (scopeId && !/^data-v-[a-f0-9]+$/.test(scopeId))
      throw new Error("[vize] Invalid style scope identity.");
    const options = this.getOptions();
    const map = inputMap
      ? parseSourceMap(typeof inputMap === "string" ? inputMap : JSON.stringify(inputMap))
      : new sources.OriginalSource(source, this.resourcePath).map({ columns: true });
    const mapped = new MappedModule(source, options.sourceMap === false ? null : map);
    if (scopeId) mapped.edit(scopeViteCssForPipeline(mapped.code, scopeId));
    const id = query.get("id") ?? scopeId?.slice("data-v-".length);
    const owner = query.get("owner");
    if (id) {
      if (!/^[a-f0-9]+$/.test(id)) throw new Error("[vize] Invalid style component identity.");
      const filename = owner
        ? path.resolve(options.root ?? this.rootContext, owner)
        : this.resourcePath;
      mapped.edit(
        transformViteCssVarsForPipeline(mapped.code, id, query.has("dev") ? undefined : filename),
      );
    }
    // Scope and variable edits preserve original spans; Turbopack parses CSS once.
    callback(null, mapped.code, mapped.map);
  } catch (error) {
    callback(error instanceof Error ? error : new Error(String(error)));
  }
}
