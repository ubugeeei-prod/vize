import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import { compileSfc, type SfcCompileOptionsNapi } from "@vizejs/native";

interface RendererLane {
  /** Stable lane name emitted in CI diagnostics. */
  readonly name: "dom" | "ssr" | "vapor";
  /** Native SFC compiler options that distinguish this lane. */
  readonly options: Readonly<Pick<SfcCompileOptionsNapi, "ssr" | "vapor">>;
}

const rendererLanes: readonly RendererLane[] = [
  { name: "dom", options: { ssr: false, vapor: false } },
  { name: "ssr", options: { ssr: true, vapor: false } },
  { name: "vapor", options: { ssr: false, vapor: true } },
];

/**
 * Recursively collect authored Vue SFCs in deterministic path order.
 *
 * Generated output is deliberately excluded: this gate verifies the canonical,
 * inspectable source that consumers install and edit.
 */
async function collectSfcFiles(sourceRoot: string): Promise<readonly string[]> {
  const entries = await readdir(sourceRoot, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry): Promise<readonly string[]> => {
      const entryPath = path.join(sourceRoot, entry.name);
      if (entry.isDirectory()) return collectSfcFiles(entryPath);
      return entry.isFile() && entry.name.endsWith(".vue") ? [entryPath] : [];
    }),
  );

  return files.flat().sort((left, right) => left.localeCompare(right));
}

/** Format native diagnostics without losing the file and renderer lane. */
function formatDiagnostics(
  file: string,
  lane: RendererLane,
  diagnostics: readonly string[],
): string {
  return `${file} failed ${lane.name} compilation:\n${diagnostics
    .map((diagnostic) => `  - ${diagnostic}`)
    .join("\n")}`;
}

/**
 * Compile one component through a production renderer lane.
 *
 * The Vapor+SSR cross-product is intentionally absent. Vize currently falls
 * back to standard SSR for that combination, and issue #3134 tracks native
 * Vapor SSR as a release-blocking capability rather than treating fallback as
 * conformance.
 */
function verifyRendererLane(file: string, source: string, lane: RendererLane): void {
  const result = compileSfc(source, {
    filename: file,
    isTs: true,
    mode: "module",
    sourceMap: true,
    ...lane.options,
  });

  assert.equal(result.errors.length, 0, formatDiagnostics(file, lane, result.errors));
  assert.equal(result.warnings.length, 0, formatDiagnostics(file, lane, result.warnings));
  assert.ok(result.code.trim().length > 0, `${file} emitted empty ${lane.name} JavaScript`);

  if (lane.name === "vapor") {
    assert.match(
      result.code,
      /defineVaporComponent|__vaporRender|__vapor\s*:\s*true/,
      `${file} did not emit a Vapor component`,
    );
  } else {
    assert.doesNotMatch(
      result.code,
      /defineVaporComponent|__vaporRender|__vapor\s*:\s*true/,
      `${file} leaked Vapor output into the ${lane.name} lane`,
    );
  }

  if (lane.name === "ssr") {
    assert.match(
      result.code,
      /function ssrRender|@vue\/server-renderer/,
      `${file} did not emit an SSR render function`,
    );
  } else {
    assert.doesNotMatch(
      result.code,
      /function ssrRender|@vue\/server-renderer/,
      `${file} leaked SSR output into the ${lane.name} lane`,
    );
  }
}

const sourceRoots = process.argv.slice(2);
const resolvedRoots = sourceRoots.length > 0 ? sourceRoots : ["src"];
const sourceFiles = (
  await Promise.all(resolvedRoots.map((sourceRoot) => collectSfcFiles(path.resolve(sourceRoot))))
).flat();

assert.ok(sourceFiles.length > 0, `No Vue SFCs found below: ${resolvedRoots.join(", ")}`);

for (const file of sourceFiles) {
  const source = await readFile(file, "utf8");
  for (const lane of rendererLanes) verifyRendererLane(file, source, lane);
}

console.log(
  JSON.stringify({
    check: "@vizejs/ui renderer conformance",
    sourceFiles: sourceFiles.length,
    compilations: sourceFiles.length * rendererLanes.length,
    lanes: rendererLanes.map((lane) => lane.name),
  }),
);
