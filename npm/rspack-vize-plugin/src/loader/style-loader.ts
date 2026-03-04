/**
 * Vize Style Loader - Extract style blocks from .vue files
 *
 * Responsibilities:
 * 1. Extract the style block at the specified index from .vue source
 * 2. Process scoped CSS (using fallback transformer)
 * 3. Output pure CSS string
 *
 * Loader chain order (executed right to left):
 *   - Native CSS:   [vizeStyleLoader] → Rspack experiments.css
 *   - CssExtract:   CssExtract.loader → css-loader → [vizeStyleLoader]
 *   - SCSS:         sass-loader → [vizeStyleLoader]
 *
 * Note: vizeStyleLoader should be placed at the rightmost position (executed first)
 */

import type { LoaderContext } from "@rspack/core";
import fs from "node:fs/promises";
import path from "node:path";
import { extractStyleBlocks, addScopeToCssFallback } from "../shared/utils.js";
import type { VizeStyleLoaderOptions } from "../types/index.js";

/**
 * Track files for which the scoped CSS fallback warning has already been
 * emitted, to avoid flooding console output in watch mode.
 */
const scopedWarningEmitted = new Set<string>();

export default function vizeStyleLoader(
  this: LoaderContext<VizeStyleLoaderOptions>,
  source: string,
): void {
  const callback = this.async();
  const { resourceQuery, resourcePath } = this;

  // No query parameter - should not happen, but handle gracefully
  if (!resourceQuery) {
    callback(null, source);
    return;
  }

  const params = new URLSearchParams(resourceQuery.slice(1));
  const type = params.get("type");

  // Only handle style type
  if (type !== "style") {
    callback(null, source);
    return;
  }

  const index = parseInt(params.get("index") || "0", 10);
  const scoped = params.get("scoped");

  // Add dependency for file watching
  this.addDependency(resourcePath);

  // 1. Extract style block at the specified index
  const styles = extractStyleBlocks(source);
  const style = styles[index];

  if (!style) {
    this.emitError(new Error(`[vize] Style block at index ${index} not found in ${resourcePath}`));
    callback(null, "");
    return;
  }

  // 2. Handle <style src="..."> external style files
  if (style.src) {
    const resolvedStylePath = path.resolve(path.dirname(resourcePath), style.src);
    this.addDependency(resolvedStylePath);

    // Use async fs to avoid blocking the loader runner thread pool
    fs.readFile(resolvedStylePath, "utf-8")
      .then((externalCss) => {
        const result = processScopedCss(this, externalCss, style.scoped, scoped, resourcePath);
        callback(null, result);
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

  // 3. Process inline style content
  const result = processScopedCss(this, style.content, style.scoped, scoped, resourcePath);
  callback(null, result);
}

/**
 * Apply scoped CSS transformation if needed.
 *
 * Note: @vizejs/native compileSfc returns merged scoped CSS (result.css), but
 * only as a single combined string — not per-block. Until the native API exposes
 * per-block scoped CSS, this fallback is used for individual style blocks.
 */
function processScopedCss(
  context: LoaderContext<VizeStyleLoaderOptions>,
  css: string,
  isScoped: boolean,
  scopeId: string | null,
  resourcePath: string,
): string {
  if (isScoped && scopeId) {
    // Only warn once per file to avoid noise in watch mode
    if (!scopedWarningEmitted.has(resourcePath)) {
      scopedWarningEmitted.add(resourcePath);
      context.emitWarning(
        new Error(
          `[vize] scoped CSS uses fallback transformer with limitations: ` +
            `no :deep()/:global()/:slotted(), no nested @media. ` +
            `File: ${resourcePath}`,
        ),
      );
    }
    return addScopeToCssFallback(css, scopeId);
  }
  return css;
}
