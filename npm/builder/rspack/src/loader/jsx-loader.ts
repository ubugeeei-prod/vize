/**
 * JSX/TSX loader. Compiles `.jsx`/`.tsx` Vue components → render code via the
 * native JSX compiler. Mirrors the main `.vue` loader's routing, but the JSX
 * lowering path has no style/custom blocks, so the output is the render module
 * verbatim. TypeScript stripping for `.tsx` is left to a `builtin:swc-loader`
 * post-rule, exactly as `.vue` files rely on one.
 */

import type { LoaderContext } from "@rspack/core";
import { compileJsxModule } from "../shared/compiler.ts";
import { matchesPattern } from "../shared/utils.ts";
import type { VizeLoaderOptions } from "../types/index.ts";

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
    // Honor the loader's source-map setting (rspack toggles `this.sourceMap`
    // from devtool); fall back to the explicit option, then on by default to
    // match the `.vue` loader. Skipped only when the compiler tooling is off.
    const sourceMap = options.sourceMap ?? this.sourceMap ?? true;

    const { code, map, warnings } = compileJsxModule(resourcePath, source, {
      jsxMode: options.jsxMode,
      vapor: options.vapor ?? false,
      sourceMap,
    });

    for (const warning of warnings) {
      this.emitWarning(new Error(`[vize] ${warning}`));
    }

    // Forward the v3 map (parsed to the object rspack expects) so downstream
    // devtools chain it back to the JSX source (#1533).
    callback(null, code, map ? JSON.parse(map) : undefined);
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
