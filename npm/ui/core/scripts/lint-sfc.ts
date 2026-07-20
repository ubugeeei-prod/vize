import path from "node:path";
import { pathToFileURL } from "node:url";

import { lintPatinaSfc } from "@vizejs/native";
import {
  formatSfcLintResults,
  lintSfcFiles as lintFiles,
  runSfcLintCli,
} from "@vizeui/tooling/lint-sfc";

export { formatSfcLintResults };

/**
 * Lint the package's SFC sources with Vize's opinionated preset.
 *
 * @param sourceRoots Directories containing SFC sources.
 * @default "src"
 */
export async function lintSfcFiles(sourceRoots: string | readonly string[] = "src") {
  return lintFiles(lintPatinaSfc, sourceRoots);
}

const invokedAsScript =
  process.argv[1] != null && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url;

if (invokedAsScript) {
  const sourceRoots = process.argv.slice(2);
  await runSfcLintCli(lintPatinaSfc, sourceRoots.length > 0 ? sourceRoots : "src");
}
