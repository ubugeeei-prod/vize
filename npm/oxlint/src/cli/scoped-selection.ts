import path from "node:path";
import { isStandaloneHtmlFile } from "../file-kinds.ts";

import {
  createScopedMirror,
  loadScopedConfig,
  readScopedConfig,
  validateNestedConfigs,
  validateScopedConfig,
  validateSelectionPaths,
} from "./scoped-config.ts";
import { withoutLintTargets } from "./args.ts";
import type { PreparedWorkaroundFiles } from "./workaround-files.js";
import {
  bridgeProjectArgs,
  replaceConfigArgs,
  splitProjectConfig,
  writeOriginalProjectConfig,
} from "./project-checks.ts";
import { projectOutputFormat, unavailableProjectTransport } from "./project-output.ts";

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
  | { args: string[]; prepared: PreparedWorkaroundFiles; originalResult?: OxlintProcessResult }
  | undefined
> {
  if (vueFiles.length === 0) return undefined;
  // Oxlint does not select standalone HTML. Keep its existing adapter only
  // when exhaustive candidate discovery found no Vue SFC to transport.
  if (vueFiles.every(isStandaloneHtmlFile)) return undefined;
  let config = readScopedConfig(cwd, originalArgs);
  if (config == null) {
    // A safe carrier cannot replace core/custom/parser source ownership. Keep
    // real engine failures and inspection output; refuse unqualified linting
    // instead of falling back to a carrier-only successful report.
    const inspection = originalArgs.some(
      (arg) =>
        ["-h", "--help", "-V", "--version", "--print-config", "--rules", "--debug"].includes(arg) ||
        arg.startsWith("--debug="),
    );
    if (
      originalArgs.some((arg) =>
        [
          "--fix",
          "--fix-suggestions",
          "--fix-dangerously",
          "--init",
          "--suppress-all",
          "--prune-suppressions",
          "--lsp",
        ].includes(arg),
      )
    )
      throw new Error("Script-safe Vue transport cannot preserve writes or LSP execution.");
    if (inspection) return { result: await run(originalArgs) };
    const selection = await run(["--debug", "files", ...originalArgs]);
    if (selection.status !== 0) return { result: selection };
    const selected = parseFiles(cwd, selection);
    const original = await run(originalArgs);
    if (vueFiles.some(isStandaloneHtmlFile))
      return {
        result: unavailableProjectTransport(
          original,
          new Error("Scoped Vue transport cannot combine standalone HTML and Vue targets."),
        ),
      };
    if (!vueFiles.some((file) => selected.includes(file))) return { result: original };
    return {
      result: {
        ...original,
        status: Math.max(original.status ?? 1, 1),
        stderr:
          original.stderr +
          "Script-safe Vue transport requires one regular JSON or TS/MTS object config; " +
          "the original engine report is retained, but Vize transport is unqualified.\n",
      },
    };
  }
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
  config = await loadScopedConfig(config);
  const selected = parseFiles(cwd, selection);
  validateNestedConfigs(config, selected);
  if (selected.some((file) => !candidates.has(file)))
    throw new Error(
      "Scoped Vue transport cannot bind every selected file to an original candidate.",
    );
  const selectedVue = vueFiles.filter((file) => selected.includes(file));
  if (selectedVue.length === 0) {
    const original = await run(originalArgs);
    return {
      result: vueFiles.some(isStandaloneHtmlFile)
        ? unavailableProjectTransport(
            original,
            new Error("Scoped Vue transport cannot combine standalone HTML and Vue targets."),
          )
        : original,
    };
  }
  // The carrier intentionally contains no executable original script. Core,
  // custom and parser checks therefore always need the unchanged originals,
  // even when neither import nor type-aware rules are configured.
  const bridgeOptions = bridgeProjectArgs(options);
  projectOutputFormat(options);
  const phases = splitProjectConfig(config);
  validateScopedConfig(phases.bridge, bridgeOptions);
  let originalResult: OxlintProcessResult | undefined;
  let mirror: ReturnType<typeof createScopedMirror> | undefined;
  try {
    // Preserve the original engine's target traversal and diagnostic order,
    // in addition to its selected set. Expanding a directory into sorted
    // files changes complete native/core packet order even with one thread.
    const source = writeOriginalProjectConfig(phases.original);
    try {
      const sourceArgs = replaceConfigArgs(originalArgs, source.file);
      const sourceSelection = await run(["--debug", "files", ...sourceArgs]);
      if (sourceSelection.status !== 0 || !equalFiles(selected, parseFiles(cwd, sourceSelection)))
        throw new Error("Original-path project checks changed Oxlint's selected files.");
      originalResult = await run(sourceArgs);
    } finally {
      source.cleanup();
    }
    config = phases.bridge;
    if (vueFiles.some(isStandaloneHtmlFile))
      throw new Error("Scoped Vue transport cannot combine standalone HTML and Vue targets.");
    mirror = createScopedMirror(cwd, config, selectedVue);
    const args = [...bridgeOptions];
    if (config.discovered) args.unshift("--config", mirror.sibling);
    // Last config wins in the original parser; replace its explicit occurrence
    // rather than adding a duplicate option with implementation-specific rules.
    for (let index = 0; index < args.length && args[index] !== "--"; index += 1) {
      if (args[index] === "-c" || args[index] === "--config") args[++index] = mirror.sibling;
      else if (args[index].startsWith("--config=")) args[index] = `--config=${mirror.sibling}`;
    }
    const projected = selectedVue.map((file) => mirror.originalsToCopies.get(file) ?? file).sort();
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
      originalResult,
      prepared: {
        appendedArgs: [],
        pathReplacements: mirror.pathReplacements,
        locations: mirror.locations,
        usedScriptlessWorkaround: true,
        cleanup: mirror.cleanup,
      },
    };
  } catch (error) {
    let failure = error;
    try {
      mirror?.cleanup();
    } catch (cleanupError) {
      failure = new AggregateError([error, cleanupError], "Vue transport and cleanup failed.");
    }
    if (originalResult) return { result: unavailableProjectTransport(originalResult, failure) };
    throw failure;
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
