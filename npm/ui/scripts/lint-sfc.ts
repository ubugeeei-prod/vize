import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { lintPatinaSfc } from "@vizejs/native";
import {
  formatSfcLintResults,
  lintSfcFiles as lintFiles,
  runSfcLintCli,
  type SfcLintFunction,
} from "./source-quality/lint-sfc.ts";

export { formatSfcLintResults };

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const corsaExecutable = process.platform === "win32" ? "tsc.exe" : "tsc";
const corsaPackageName = `typescript-${process.platform}-${process.arch}`;
const workspaceCorsaPath = resolveWorkspaceCorsaPath();
const packageCompatibilityRules = new Set([
  // Date/time primitives retain a props object for Vue versions before 3.5.
  "script/define-props-destructuring",
  // This component library intentionally publishes native-element-like names
  // such as Button, Table, and Code as part of its public API.
  "vue/multi-word-component-names",
  // Optional primitive props are intentionally left without runtime defaults;
  // consumers need to be able to distinguish omission from an explicit value.
  "script/require-default-prop",
  // The public component APIs intentionally spell explicit Boolean defaults;
  // removing them is a package migration, not a product-preset exception.
  "script/no-boolean-default",
  // Existing renderless UI components retain their authored void/component
  // syntax until the package can migrate it without snapshot churn.
  "vue/html-self-closing",
]);

const lintCoreSfc: SfcLintFunction = (source, options) => {
  const result = lintPatinaSfc(source, {
    ...options,
    // The published UI package already exposes camelCase event names. Keep
    // its native SFC gate aligned with the package's explicit Vite+ policy.
    customEventNameCasing: "camelCase",
    ...(workspaceCorsaPath == null ? {} : { corsaPath: workspaceCorsaPath }),
  });
  // Keep package-local compatibility exceptions out of the product preset.
  if (Array.isArray(result?.diagnostics)) {
    result.diagnostics = result.diagnostics.filter(
      (diagnostic: { rule?: unknown }) =>
        typeof diagnostic.rule !== "string" || !packageCompatibilityRules.has(diagnostic.rule),
    );
  }
  return result;
};

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
      "../../..",
      "node_modules",
      "@typescript",
      corsaPackageName,
      "lib",
      corsaExecutable,
    ),
    path.resolve(
      process.cwd(),
      "../../node_modules/@typescript",
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
