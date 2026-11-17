import * as native from "@vizejs/native";
import {
  compileTsrxModule,
  isTsrxFile,
  TSRX_INCLUDE_PATTERN,
  type CompileTsrxFn,
} from "../../shared/tsrx.ts";
import { createFilter } from "./filter.ts";
import { stripTypeScript } from "./strip-types.ts";
import type { NormalizedVizeUnpluginOptions } from "./types.ts";

const { compileTsrx } = native as unknown as { compileTsrx: CompileTsrxFn };

export { isTsrxFile };

/**
 * `.tsrx` modules compile unless `exclude` matches them: no other plugin reads
 * them, so `include` (which selects `.vue` and JSX modules) does not apply.
 */
export function createTsrxFilter(
  exclude: NormalizedVizeUnpluginOptions["exclude"],
): (id: string) => boolean {
  return createFilter(TSRX_INCLUDE_PATTERN, exclude);
}

/**
 * Compile one `.tsrx` module through the native TSRX entry (#13140), then strip
 * its TypeScript as the `.vue` path strips compiled SFCs. A refused module
 * throws with every diagnostic.
 */
export async function compileTsrxRequest(
  id: string,
  source: string,
  options: NormalizedVizeUnpluginOptions,
): Promise<{ code: string; map: null }> {
  const compiled = compileTsrxModule(compileTsrx, id, source, {
    ssr: options.ssr,
    vapor: options.vapor,
  });
  // No map: the entry maps no span yet (#13131), and a map of the strip alone
  // would point at the compiled module instead of the authored source.
  const stripped = await stripTypeScript(id, compiled.code, false);
  return { code: stripped.code, map: null };
}
