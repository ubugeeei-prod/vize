/**
 * Vize Loader - Main loader for compiling .vue files
 *
 * Responsibilities:
 * 1. Compile SFC to JavaScript using @vizejs/native
 * 2. Inject style import statements (with query parameters)
 * 3. Output JS module code
 *
 * Note: Must be used with `oneOf` in Rspack config to ensure mutual exclusion
 * with the style-loader rule.
 */

import type { LoaderContext } from "@rspack/core";
import path from "node:path";
import { compileFile, generateOutput } from "../shared/compiler.js";
import { matchesPattern } from "../shared/utils.js";
import type { VizeLoaderOptions } from "../types/index.js";

export default function vizeLoader(this: LoaderContext<VizeLoaderOptions>, source: string): void {
  const callback = this.async();
  const options = this.getOptions();
  const resourcePath = this.resourcePath;
  const resourceQuery = this.resourceQuery;
  const requestPath = normalizeRequestPath(this, resourcePath);

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
    // 1. Compile SFC
    const compiled = compileFile(resourcePath, source, {
      sourceMap: options.sourceMap ?? this.sourceMap ?? true,
      ssr: options.ssr ?? false,
      compilerOptions: options.compilerOptions,
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
      callback(
        new Error(
          `[vize] Compilation failed for ${resourcePath}:\n${errorSummary}`,
        ),
      );
      return;
    }

    // 2. Generate output code (with style imports)
    const output = generateOutput(compiled, {
      requestPath,
    });

    // 3. Return the compiled JavaScript
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
