/**
 * TSRX loader. Compiles `.tsrx` modules through the native TSRX entry
 * (`compileTsrx`, #13140), the one `vize build` uses. The output is TypeScript
 * without JSX; stripping it is left to a `builtin:swc-loader` post-rule, as for
 * `.vue` and `.tsx` modules. A refused module fails with every diagnostic.
 */

import type { LoaderContext } from "@rspack/core";
import * as native from "@vizejs/native";
import { compileTsrxModule, type CompileTsrxFn } from "../../../shared/tsrx.ts";
import { matchesPattern } from "../shared/utils.ts";
import type { VizeTsrxLoaderOptions } from "../types/index.ts";

const { compileTsrx } = native as unknown as { compileTsrx: CompileTsrxFn };

/** Compile a TSRX module through the native TSRX entry. */
export default function vizeTsrxLoader(
  this: LoaderContext<VizeTsrxLoaderOptions>,
  source: string,
): void {
  const options = this.getOptions();
  const resourcePath = this.resourcePath;
  if (
    !matchesPattern(resourcePath, options.include, true) ||
    matchesPattern(resourcePath, options.exclude, false)
  ) {
    this.emitWarning(
      new Error(
        `[vize] File is filtered out by loader options include/exclude: ${resourcePath}. ` +
          `Passing through source unchanged.`,
      ),
    );
    this.callback(null, source);
    return;
  }
  try {
    const { code } = compileTsrxModule(compileTsrx, resourcePath, source, {
      ssr: options.ssr ?? false,
      vapor: options.vapor ?? false,
    });
    this.callback(null, code);
  } catch (error) {
    this.callback(error as Error);
  }
}
