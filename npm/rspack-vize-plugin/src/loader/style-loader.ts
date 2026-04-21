/** Style Loader — extracts a style block by index from .vue source. Chain: [scope-loader] ← (preprocessor) ← [vizeStyleLoader] */

import type { LoaderContext } from "@rspack/core";
import fs from "node:fs/promises";
import path from "node:path";
import { extractStyleBlocks } from "../shared/utils.ts";
import type { VizeStyleLoaderOptions } from "../types/index.ts";

export default function vizeStyleLoader(
  this: LoaderContext<VizeStyleLoaderOptions>,
  source: string,
): void {
  const callback = this.async();
  const { resourceQuery, resourcePath } = this;

  if (!resourceQuery) {
    callback(null, source);
    return;
  }

  const params = new URLSearchParams(resourceQuery.slice(1));
  const type = params.get("type");

  if (type !== "style") {
    callback(null, source);
    return;
  }

  const index = parseInt(params.get("index") || "0", 10);

  this.addDependency(resourcePath);

  const styles = extractStyleBlocks(source);
  const style = styles[index];

  if (!style) {
    this.emitError(new Error(`[vize] Style block at index ${index} not found in ${resourcePath}`));
    callback(null, "");
    return;
  }

  // Handle <style src="..."> external files
  if (style.src) {
    const resolvedStylePath = path.resolve(path.dirname(resourcePath), style.src);
    this.addDependency(resolvedStylePath);

    fs.readFile(resolvedStylePath, "utf-8")
      .then((externalCss) => {
        callback(null, externalCss);
      })
      .catch(() => {
        this.emitWarning(
          new Error(
            `[vize] <style src> target not found: ${style.src} (resolved: ${resolvedStylePath}) in ${resourcePath}`,
          ),
        );
        callback(null, "");
      });
    return;
  }

  callback(null, style.content);
}
