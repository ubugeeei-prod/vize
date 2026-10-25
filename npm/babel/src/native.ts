import { createRequire } from "node:module";
import type { compileJsx } from "@vizejs/native";

const require = createRequire(import.meta.url);
let compiler: typeof compileJsx | undefined;

/** Keep addon initialization outside the common no-JSX path. */
export function getCompiler(): typeof compileJsx {
  if (compiler) return compiler;
  const binding: unknown = require("@vizejs/native");
  if (
    typeof binding !== "object" ||
    binding === null ||
    !("compileJsx" in binding) ||
    typeof binding.compileJsx !== "function"
  ) {
    throw new Error(
      "The installed Vize native binding does not expose compileJsx; use matching Vize package versions",
    );
  }
  compiler = binding.compileJsx as typeof compileJsx;
  return compiler;
}
