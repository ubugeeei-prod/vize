import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

interface SfcDiagnostic {
  readonly rule?: unknown;
  readonly severity?: unknown;
  readonly message?: unknown;
  readonly location?: {
    readonly start?: {
      readonly line?: unknown;
      readonly column?: unknown;
    };
  };
}

interface SfcLintResult {
  readonly diagnostics?: unknown;
}

/** One SFC and the diagnostics produced for it. */
export interface LintedFile {
  readonly filename: string;
  readonly diagnostics: readonly SfcDiagnostic[];
}

/** Native SFC lint entry point supplied by the consuming workspace package. */
export interface SfcLintFunction {
  (
    source: string,
    options: {
      readonly filename: string;
      readonly preset: "opinionated";
      readonly typeAware: true;
      readonly helpLevel: "short";
    },
  ): unknown;
}

/**
 * Lint SFC sources with Vize's strict opinionated preset.
 *
 * Diagnostics are reported in stable path order so CI logs and automated
 * tooling can compare results without depending on filesystem traversal order.
 *
 * @param lint Native Vize SFC lint function.
 * @param sourceRoots Directories containing SFC sources.
 * @default "src"
 */
export async function lintSfcFiles(
  lint: SfcLintFunction,
  sourceRoots: string | readonly string[] = "src",
): Promise<readonly LintedFile[]> {
  const roots = typeof sourceRoots === "string" ? [sourceRoots] : sourceRoots;
  const discovered = await Promise.all(roots.map((root) => collectVueFiles(path.resolve(root))));
  const files = [...new Set(discovered.flat())].sort();
  const results: LintedFile[] = [];

  for (const filename of files) {
    const source = await readFile(filename, "utf8");
    const raw = lint(source, {
      filename,
      preset: "opinionated",
      typeAware: true,
      helpLevel: "short",
    }) as SfcLintResult;
    const diagnostics = Array.isArray(raw.diagnostics)
      ? raw.diagnostics.filter(isSfcDiagnostic)
      : [];
    results.push({
      filename: path.relative(process.cwd(), filename),
      diagnostics,
    });
  }

  return results;
}

/**
 * Render Vize diagnostics for terminals and continuous integration.
 *
 * @param results Results returned by {@link lintSfcFiles}.
 * @returns A newline-delimited report; an empty string means a clean run.
 */
export function formatSfcLintResults(results: readonly LintedFile[]): string {
  return results
    .flatMap(({ filename, diagnostics }) =>
      diagnostics.map((diagnostic) => {
        const line = numberOrFallback(diagnostic.location?.start?.line, 1);
        const column = numberOrFallback(diagnostic.location?.start?.column, 1);
        const severity = diagnostic.severity === "warning" ? "warning" : "error";
        const rule = stringOrFallback(diagnostic.rule, "vize/sfc");
        const message = stringOrFallback(diagnostic.message, "Vize SFC lint diagnostic");
        return `${filename}:${line}:${column} ${severity} ${rule} ${message}`;
      }),
    )
    .join("\n");
}

/**
 * Run the shared SFC gate and set a failing process status for any diagnostic.
 *
 * @param lint Native Vize SFC lint function.
 * @param sourceRoots Directories containing SFC sources.
 * @default "src"
 */
export async function runSfcLintCli(
  lint: SfcLintFunction,
  sourceRoots: string | readonly string[] = "src",
): Promise<void> {
  const results = await lintSfcFiles(lint, sourceRoots);
  const report = formatSfcLintResults(results);
  if (report) process.stderr.write(`${report}\n`);
  if (results.some((result) => result.diagnostics.length > 0)) process.exitCode = 1;
}

async function collectVueFiles(directory: string): Promise<string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const files: string[] = [];
  for (const entry of entries) {
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await collectVueFiles(filename)));
    } else if (entry.isFile() && entry.name.endsWith(".vue")) {
      files.push(filename);
    }
  }
  return files;
}

function isSfcDiagnostic(value: unknown): value is SfcDiagnostic {
  return typeof value === "object" && value !== null;
}

function numberOrFallback(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function stringOrFallback(value: unknown, fallback: string): string {
  return typeof value === "string" && value.length > 0 ? value : fallback;
}
