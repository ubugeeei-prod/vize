import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

// Python, Lean, MoonBit and `.mjs` tooling were retired on 2026-10-13 (#8840).
// The ledger lists every file still waiting for its port; its test is the ratchet.
// This file is an executable proof specification outside the product tooling.
export const independentGraphTheory = "tests/formal/ir-graph/Reach.lean";
// UTF-16 source map coordinate conversion.
export const sourceMapCoordinates = {
  theorem: "tests/formal/source-map/Coordinates.lean",
  kernel: "davinci/vize_l4/src/write/source_map/indexed/geometry.rs",
  harness: "davinci/vize_l4/src/write/source_map/indexed/kani.rs",
} as const;
export const styleHeaderValidation = {
  theorem: "tests/formal/style-format/Header.lean",
  kernel: "crates/vize_maestro/src/source_project/style_format/header.rs",
  harness: "crates/vize_maestro/src/source_project/style_format/kani.rs",
} as const;
const positionIndex = {
  theorem: "tests/formal/position-index/Intervals.lean",
  kernel: "davinci/vize_l2/src/query/position/index/intervals.rs",
  harness: "davinci/vize_l2/src/query/position/index/intervals/tests.rs",
} as const;
export const rustProofReferences = [
  sourceMapCoordinates,
  styleHeaderValidation,
  positionIndex,
] as const;

export function checkedRustTheorems(paths: readonly string[]): Set<string> {
  return new Set(
    rustProofReferences
      .filter(({ kernel, harness }) => paths.includes(kernel) && paths.includes(harness))
      .map(({ theorem }) => theorem),
  );
}

export const retiredToolingLedger = "tools/config/retired-tooling-languages.json";
export const retiredToolingLedgerTest = "tests/tooling/retired-tooling-languages.test.ts";
export const retiredToolingLanguages = ["py", "lean", "mbt", "mjs"] as const;
export type RetiredToolingLanguage = (typeof retiredToolingLanguages)[number];

export function retiredToolingLanguage(path: string): RetiredToolingLanguage | undefined {
  const name = path.slice(path.lastIndexOf("/") + 1);
  if (name === "lean-toolchain") return "lean";
  return retiredToolingLanguages.find((language) => name.endsWith(`.${language}`));
}

function record(value: unknown): Readonly<Record<string, unknown>> | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as Readonly<Record<string, unknown>>)
    : undefined;
}

// Whether a changed path can flip the ledger test: a retired-language file
// outside the exempt prefixes that is present but unlisted, or listed but gone.
// An unreadable ledger keeps the test selected so it reports the damage.
export function retiredToolingDrift(paths: readonly string[], cwd: string): boolean {
  const changed = paths.flatMap((path) => {
    if (path === independentGraphTheory) return [];
    const language = retiredToolingLanguage(path);
    return language ? [{ path, language }] : [];
  });
  if (
    rustProofReferences.some(
      ({ kernel, harness }) => paths.includes(kernel) || paths.includes(harness),
    )
  ) {
    return true;
  }
  if (changed.length === 0) return false;
  let ledger: Readonly<Record<string, unknown>> | undefined;
  try {
    ledger = record(JSON.parse(readFileSync(join(cwd, retiredToolingLedger), "utf8")));
  } catch {
    return true;
  }
  const permanent = ledger?.permanent;
  const retiring = record(ledger?.retiring);
  if (!Array.isArray(permanent) || !retiring) return true;
  return changed.some(({ path, language }) => {
    const exempt = permanent.some((entry: unknown) => {
      const { language: owner, prefix } = record(entry) ?? {};
      return owner === language && typeof prefix === "string" && path.startsWith(prefix);
    });
    if (exempt) return false;
    const listed = retiring[language];
    return !Array.isArray(listed) || listed.includes(path) !== existsSync(join(cwd, path));
  });
}
