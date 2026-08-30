import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { lintPatinaSfc } from "@vizejs/native";
import {
  formatSfcLintResults,
  lintSfcFiles as lintFiles,
  runSfcLintCli,
  type SfcLintFunction,
} from "@vizejs/ui-tooling/lint-sfc";

export { formatSfcLintResults };

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const corsaExecutable = process.platform === "win32" ? "tsc.exe" : "tsc";
const corsaPackageName = `typescript-${process.platform}-${process.arch}`;
const workspaceCorsaPath = resolveWorkspaceCorsaPath();

const lintCoreSfc: SfcLintFunction = (source, options) =>
  lintPatinaSfc(source, {
    ...options,
    ...(workspaceCorsaPath == null ? {} : { corsaPath: workspaceCorsaPath }),
  });

/**
 * Lint the package's SFC sources with Vize's opinionated preset.
 *
 * @param sourceRoots Directories containing SFC sources.
 * @default "src"
 */
export async function lintSfcFiles(sourceRoots: string | readonly string[] = "src") {
  return lintFiles(lintCoreSfc, sourceRoots);
}

function resolveWorkspaceCorsaPath(): string | undefined {
  const candidates = [
    process.env.VIZE_UI_CORSA_PATH,
    path.resolve(
      scriptDirectory,
      "../../../..",
      "node_modules",
      "@typescript",
      corsaPackageName,
      "lib",
      corsaExecutable,
    ),
    path.resolve(
      process.cwd(),
      "../../../node_modules/@typescript",
      corsaPackageName,
      "lib",
      corsaExecutable,
    ),
  ];
  return candidates.find(
    (candidate): candidate is string => candidate != null && existsSync(candidate),
  );
}

const invokedAsScript =
  process.argv[1] != null && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url;

if (invokedAsScript) {
  const sourceRoots = process.argv.slice(2);
  await runSfcLintCli(lintCoreSfc, sourceRoots.length > 0 ? sourceRoots : "src");
}
