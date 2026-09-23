/**
 * JSX/TSX loader. Compiles `.jsx`/`.tsx` Vue components → render code via the
 * native JSX compiler. Mirrors the main `.vue` loader's routing, but the JSX
 * lowering path has no style/custom blocks, so the output is the render module
 * verbatim. TypeScript stripping for `.tsx` is left to a `builtin:swc-loader`
 * post-rule, exactly as `.vue` files rely on one.
 */

import type { LoaderContext } from "@rspack/core";
import { compileJsxModule } from "../shared/compiler.ts";
import { parseSourceMap } from "../shared/source-map.ts";
import { matchesPattern } from "../shared/utils.ts";
import type { VizeJsxLoaderOptions as VizeLoaderOptions } from "../types/index.ts";

export default function vizeJsxLoader(
  this: LoaderContext<VizeLoaderOptions>,
  source: string,
): void {
  const callback = this.async();
  const options = this.getOptions();
  const resourcePath = this.resourcePath;

  this.addDependency(resourcePath);

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
    // Explicit loader options take precedence over Rspack's devtool context.
    const isProduction = this.mode === "production" || process.env.NODE_ENV === "production";
    const sourceMap = options.sourceMap ?? this.sourceMap ?? !isProduction;

    const { code, map, warnings } = compileJsxModule(resourcePath, source, {
      jsxMode: options.jsxMode,
      jsxCompat: options.jsxCompat,
      vapor: options.vapor ?? false,
      sourceMap,
    });

    for (const warning of warnings) {
      this.emitWarning(new Error(`[vize] ${warning}`));
    }

    callback(null, code, parseSourceMap(map) ? map! : undefined);
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
