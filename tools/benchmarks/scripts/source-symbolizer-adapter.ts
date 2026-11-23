import { readFileSync } from "node:fs";

/** Perf installs a self-contained adapter alongside the evidence for its selected decoder. */
export function sentinelAdapterScript(decoder: string): string {
  const program = readFileSync(new URL("./source-symbolizer-adapter.cjs", import.meta.url), "utf8");
  return program.replace('"@@SYMBOLIZER_DECODER@@"', () => JSON.stringify(decoder));
}
