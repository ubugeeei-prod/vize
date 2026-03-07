/**
 * Vize Loader - Main loader for compiling .vue files
 *
 * Responsibilities:
 * 1. Compile SFC to JavaScript using @vizejs/native
 * 2. Inject style import statements (with query parameters)
 * 3. Inject custom block import statements
 * 4. Handle <script src="..."> and <template src="..."> external references
 * 5. Detect custom element files (.ce.vue) and adjust output accordingly
 * 6. Output JS module code
 *
 * Note: Must be used with `oneOf` in Rspack config to ensure mutual exclusion
 * with the style-loader rule.
 */

import type { LoaderContext } from "@rspack/core";
import fs from "node:fs";
import path from "node:path";
import { compileFile, generateOutput } from "../shared/compiler.js";
import {
  matchesPattern,
  extractSrcInfo,
  inlineSrcBlocks,
  extractCustomBlocks,
} from "../shared/utils.js";
import type { VizeLoaderOptions } from "../types/index.js";

/** Default pattern: files ending with .ce.vue are custom elements */
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

  // Add dependency to trigger recompilation on file change
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

  // Handle custom block sub-requests (e.g. ?vue&type=i18n&index=0).
  // Without this guard, custom block imports would re-enter the main loader
  // and trigger a full SFC compilation, leading to recursive imports.
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
        // If the block has an external src, read the external file
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
    // 0. Determine custom element mode
    const isCustomElement = resolveCustomElement(resourcePath, options.customElement);

    // 1. Resolve <script src="..."> and <template src="..."> external files
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

    // 2. Compile SFC
    const compiled = compileFile(resourcePath, resolvedSource, {
      sourceMap: options.sourceMap ?? this.sourceMap ?? true,
      ssr: options.ssr ?? false,
      compilerOptions: options.compilerOptions,
      isCustomElement,
      rootContext: this.rootContext,
      isProduction,
    });

    for (const warning of compiled.warnings) {
      this.emitWarning(new Error(`[vize] ${warning}`));
    }

    // Fail fast on compilation errors — returning broken code leads to
    // confusing runtime errors that are harder to diagnose.
    if (compiled.errors.length > 0) {
      for (const error of compiled.errors) {
        this.emitError(new Error(`[vize] ${error}`));
      }
      const errorSummary = compiled.errors.join("\\n");
      callback(new Error(`[vize] Compilation failed for ${resourcePath}:\n${errorSummary}`));
      return;
    }

    // 3. Generate output code (with style imports + custom block imports + HMR)
    const output = generateOutput(compiled, {
      requestPath,
      hmr: needsHotReload,
      filePath: resourcePath,
      isProduction,
      rootContext: this.rootContext,
    });

    // 4. Return the compiled JavaScript
    // TODO: @vizejs/native compileSfc does not yet return a `map` field in
    // SfcCompileResultNapi. Once the Rust side adds source map output, pass
    // it here as: callback(null, output, map)
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

/**
 * Determine if the file should be compiled in custom element mode.
 */
function resolveCustomElement(
  resourcePath: string,
  customElement: boolean | RegExp | undefined,
): boolean {
  if (customElement === true) return true;
  if (customElement === false || customElement === undefined) {
    // Default: detect .ce.vue pattern
    return DEFAULT_CE_PATTERN.test(resourcePath);
  }
  // RegExp provided
  return customElement.test(resourcePath);
}

/**
 * Generate the request path for style sub-imports.
 *
 * Style imports are resolved relative to the issuing .vue file's directory,
 * so we use `./basename.vue` (self-reference) instead of a root-relative path.
 */
function normalizeRequestPath(
  _context: LoaderContext<VizeLoaderOptions>,
  resourcePath: string,
): string {
  const basename = path.basename(resourcePath);
  return `./${basename}`;
}
