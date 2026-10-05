import path from "node:path";

import {
  createScopedMirror,
  readScopedConfig,
  validateScopedConfig,
  validateSelectionPaths,
} from "./scoped-config.ts";
import { withoutLintTargets } from "./args.ts";
import type { PreparedWorkaroundFiles } from "./workaround-files.js";

export interface OxlintProcessResult {
  status: number | null;
  stderr: string;
  stdout: string;
}

type Run = (args: readonly string[]) => Promise<OxlintProcessResult>;

export async function prepareScopedSelection(
  cwd: string,
  originalArgs: readonly string[],
  vueFiles: readonly string[],
  run: Run,
  candidates: ReadonlySet<string>,
): Promise<
  | { result: OxlintProcessResult }
  | { args: string[]; prepared: PreparedWorkaroundFiles }
  | undefined
> {
  if (vueFiles.length === 0) return undefined;
  const config = readScopedConfig(cwd, originalArgs);
  if (config == null) return undefined;
  const options = withoutLintTargets(originalArgs);
  if (
    options.some(
      (arg) =>
        [
          "--debug",
          "--print-config",
          "--rules",
          "--lsp",
          "--init",
          "-h",
          "--help",
          "-V",
          "--version",
          "--suppress-all",
          "--prune-suppressions",
        ].includes(arg) || arg.startsWith("--debug="),
    )
  )
    throw new Error(
      "Scoped Vue transport cannot combine linting with other Oxlint inspection modes.",
    );
  validateSelectionPaths(cwd, candidates);

  // Stock --debug files runs after original CLI/VCS/config ignore selection and
  // config validation, before linting or fixing. The engine owns this predicate.
  const selection = await run(["--debug", "files", ...originalArgs]);
  if (selection.status !== 0) return { result: selection };
  validateScopedConfig(config, originalArgs);
  const selected = parseFiles(cwd, selection);
  if (selected.some((file) => !candidates.has(file)))
    throw new Error(
      "Scoped Vue transport cannot bind every selected file to an original candidate.",
    );
  const selectedVue = vueFiles.filter((file) => selected.includes(file));
  if (selectedVue.length === 0) return { result: await run(originalArgs) };
  const mirror = createScopedMirror(cwd, config, selectedVue);
  try {
    const args = [...options];
    // Last config wins in the original parser; replace its explicit occurrence
    // rather than adding a duplicate option with implementation-specific rules.
    for (let index = 0; index < args.length && args[index] !== "--"; index += 1) {
      if (args[index] === "-c" || args[index] === "--config") args[++index] = mirror.sibling;
      else if (args[index].startsWith("--config=")) args[index] = `--config=${mirror.sibling}`;
    }
    const projected = selected.map((file) => mirror.originalsToCopies.get(file) ?? file).sort();
    args.push(...projected);
    const transported = await run(["--debug", "files", ...args]);
    if (
      transported.status !== 0 ||
      transported.stderr !== "" ||
      !equalFiles(projected, parseFiles(cwd, transported))
    )
      throw new Error(
        "Scoped Vue transport changed Oxlint's selected files. " +
          "The current ignore/config envelope cannot preserve original path ownership.\n" +
          transported.stdout +
          transported.stderr,
      );
    return {
      args,
      prepared: {
        appendedArgs: [],
        pathReplacements: mirror.pathReplacements,
        usedScriptlessWorkaround: true,
        cleanup: mirror.cleanup,
      },
    };
  } catch (error) {
    mirror.cleanup();
    throw error;
  }
}

function parseFiles(cwd: string, result: OxlintProcessResult): string[] {
  if (result.stderr !== "")
    throw new Error(`Oxlint file selection emitted unexpected stderr: ${result.stderr}`);
  const lines = result.stdout.split("\n");
  if (lines.at(-1) === "") lines.pop();
  if (lines.some((line) => line === "" || line.includes("\r")))
    throw new Error("Oxlint file selection did not return a complete path per line.");
  const files = lines.map((file) => path.resolve(cwd, file));
  if (new Set(files).size !== files.length)
    throw new Error("Oxlint file selection returned duplicate paths.");
  return files.sort();
}

function equalFiles(expected: readonly string[], actual: readonly string[]): boolean {
  return (
    expected.length === actual.length && expected.every((file, index) => file === actual[index])
  );
}
