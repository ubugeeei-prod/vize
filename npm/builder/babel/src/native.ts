import { createRequire } from "node:module";
import type { compileJsx, compileSfc, generateSfcScopeId } from "@vizejs/native";

/** Native compiler operations required by the Babel adapter. */
export interface NativeCompiler {
  compileJsx: typeof compileJsx;
  compileSfc: typeof compileSfc;
  generateSfcScopeId: typeof generateSfcScopeId;
}

let compiler: NativeCompiler | undefined;

/** Keep addon initialization outside the common no-JSX/non-SFC path. */
export function getCompiler(): NativeCompiler {
  if (compiler) return compiler;
  const require = createRequire(typeof __filename === "string" ? __filename : import.meta.url);
  const binding: unknown = require("@vizejs/native");
  if (
    typeof binding !== "object" ||
    binding === null ||
    !("compileJsx" in binding) ||
    typeof binding.compileJsx !== "function" ||
    !("compileSfc" in binding) ||
    typeof binding.compileSfc !== "function" ||
    !("generateSfcScopeId" in binding) ||
    typeof binding.generateSfcScopeId !== "function"
  ) {
    throw new Error(
      "The installed Vize native binding does not expose the Babel compiler surface; use matching Vize package versions",
    );
  }
  compiler = binding as NativeCompiler;
  return compiler;
}
