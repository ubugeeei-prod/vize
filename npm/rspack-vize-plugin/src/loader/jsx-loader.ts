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
    const { code, warnings } = compileJsxModule(resourcePath, source, {
      jsxMode: options.jsxMode,
      vapor: options.vapor ?? false,
    });

    for (const warning of warnings) {
      this.emitWarning(new Error(`[vize] ${warning}`));
    }

    callback(null, code);
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
