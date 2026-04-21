/** Main .vue SFC loader. Compiles SFC → JS; must be used with `oneOf` (mutual exclusion with style-loader). */

import type { LoaderContext } from "@rspack/core";
import fs from "node:fs";
import path from "node:path";
import { compileFile, generateOutput } from "../shared/compiler.ts";
import {
  matchesPattern,
  extractSrcInfo,
  inlineSrcBlocks,
  extractCustomBlocks,
} from "../shared/utils.ts";
import type { VizeLoaderOptions } from "../types/index.ts";

/** .ce.vue → custom element */
const DEFAULT_CE_PATTERN = /\.ce\.vue$/;

export default function vizeLoader(this: LoaderContext<VizeLoaderOptions>, source: string): void {
  const callback = this.async();
  const options = this.getOptions();
  const resourcePath = this.resourcePath;
  const resourceQuery = this.resourceQuery;
  const requestPath = normalizeRequestPath(this, resourcePath);

  const isProduction = this.mode === "production" || process.env.NODE_ENV === "production";
  const isSsr = options.ssr ?? false;
  const needsHotReload = !isSsr && !isProduction && options.hotReload !== false;

  this.addDependency(resourcePath);

  if (resourceQuery?.includes("type=style")) {
    callback(
      new Error(
        `[vize] Main loader received style sub-request: ${resourcePath}${resourceQuery}. ` +
          `Use module.rules[].oneOf with resourceQuery branches so style requests are handled by @vizejs/rspack-plugin/style-loader.`,
      ),
    );
    return;
  }

  // Custom block sub-requests (e.g. ?vue&type=i18n&index=0)
  if (
    resourceQuery &&
    resourceQuery.includes("vue") &&
    resourceQuery.includes("type=") &&
    !resourceQuery.includes("type=style")
  ) {
    const params = new URLSearchParams(resourceQuery.slice(1));
    const blockType = params.get("type");
    if (blockType && blockType !== "style") {
      const blockIndex = parseInt(params.get("index") || "0", 10);
      const customBlocks = extractCustomBlocks(source);
      const block = customBlocks[blockIndex];
      if (block) {
        if (block.src) {
          const blockPath = path.resolve(path.dirname(resourcePath), block.src);
          this.addDependency(blockPath);
          try {
            const blockContent = fs.readFileSync(blockPath, "utf-8");
            callback(null, blockContent);
          } catch {
            callback(
              new Error(
                `[vize] Custom block <${blockType} src="${block.src}"> not found (resolved: ${blockPath}) in ${resourcePath}`,
              ),
            );
          }
          return;
        }
        callback(null, block.content);
      } else {
        callback(null, "");
      }
      return;
    }
  }

  if (!shouldCompileFile(resourcePath, options)) {
    this.emitWarning(
      new Error(
        `[vize] File is filtered out by loader options include/exclude: ${resourcePath}. ` +
          `Passing through source unchanged.`,
      ),
    );
    callback(null, source);
    return;
  }

  try {
    const isCustomElement = resolveCustomElement(resourcePath, options.customElement);

    // Resolve external src references
    const srcInfo = extractSrcInfo(source);
    let resolvedSource = source;

    if (srcInfo.scriptSrc) {
      const scriptPath = path.resolve(path.dirname(resourcePath), srcInfo.scriptSrc);
      this.addDependency(scriptPath);
      try {
        const scriptContent = fs.readFileSync(scriptPath, "utf-8");
        resolvedSource = inlineSrcBlocks(resolvedSource, scriptContent, null);
      } catch {
        callback(
          new Error(
            `[vize] <script src="${srcInfo.scriptSrc}"> not found (resolved: ${scriptPath}) in ${resourcePath}`,
          ),
        );
        return;
      }
    }

    if (srcInfo.templateSrc) {
      const templatePath = path.resolve(path.dirname(resourcePath), srcInfo.templateSrc);
      this.addDependency(templatePath);
      try {
        const templateContent = fs.readFileSync(templatePath, "utf-8");
        resolvedSource = inlineSrcBlocks(resolvedSource, null, templateContent);
      } catch {
        callback(
          new Error(
            `[vize] <template src="${srcInfo.templateSrc}"> not found (resolved: ${templatePath}) in ${resourcePath}`,
          ),
        );
        return;
      }
    }

    const compiled = compileFile(resourcePath, resolvedSource, {
      sourceMap: options.sourceMap ?? this.sourceMap ?? true,
      ssr: options.ssr ?? false,
      vapor: options.vapor ?? false,
      compilerOptions: options.compilerOptions,
      isCustomElement,
      rootContext: this.rootContext,
      isProduction,
      transformAssetUrls: options.transformAssetUrls,
    });

    for (const warning of compiled.warnings) {
      this.emitWarning(new Error(`[vize] ${warning}`));
    }

    if (compiled.errors.length > 0) {
      for (const error of compiled.errors) {
        this.emitError(new Error(`[vize] ${error}`));
      }
      const errorSummary = compiled.errors.join("\\n");
      callback(new Error(`[vize] Compilation failed for ${resourcePath}:\n${errorSummary}`));
      return;
    }

    const output = generateOutput(compiled, {
      requestPath,
      hmr: needsHotReload,
      filePath: resourcePath,
      isProduction,
      rootContext: this.rootContext,
    });

    // TODO: pass source map when @vizejs/native exposes it
    callback(null, output);
  } catch (error) {
    callback(error as Error);
  }
}

function shouldCompileFile(file: string, options: VizeLoaderOptions): boolean {
  if (!matchesPattern(file, options.include, true)) {
    return false;
  }

  if (matchesPattern(file, options.exclude, false)) {
    return false;
  }

  return true;
}

/** Resolve custom element mode for a file. */
function resolveCustomElement(
  resourcePath: string,
  customElement: boolean | RegExp | undefined,
): boolean {
  if (customElement === true) return true;
  if (customElement === false || customElement === undefined) {
    return DEFAULT_CE_PATTERN.test(resourcePath);
  }
  return customElement.test(resourcePath);
}

/** Returns `./basename.vue` for style sub-import paths. */
function normalizeRequestPath(
  _context: LoaderContext<VizeLoaderOptions>,
  resourcePath: string,
): string {
  const basename = path.basename(resourcePath);
  return `./${basename}`;
}
