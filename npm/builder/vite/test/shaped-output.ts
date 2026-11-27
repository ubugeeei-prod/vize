// Output assembly for hand-written compiled modules. The native compiler
// returns every module with its shape, so these attach the shape it reports.
import type { CompiledModule } from "../src/types.ts";
import {
  generateOutput as assemble,
  generateOutputWithMap as assembleWithMap,
} from "../src/utils/index.ts";
import { withModuleShape } from "./oxc-module-output.ts";

export const generateOutput: typeof assemble = (compiled, options) =>
  assemble(withModuleShape(compiled), options);

export const generateOutputWithMap: typeof assembleWithMap = (compiled, options) =>
  assembleWithMap(withModuleShape(compiled), options);

/** A compile cache of hand-written modules, each with its native shape. */
export function shapedCache(entries: [string, CompiledModule][]): Map<string, CompiledModule> {
  return new Map(entries.map(([file, compiled]) => [file, withModuleShape(compiled)]));
}
