import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

// Python, Lean, MoonBit and `.mjs` tooling were retired on 2026-10-13 (#8840).
// The ledger lists every file still waiting for its port; its test is the ratchet.
export const retiredToolingLedger = "tools/config/retired-tooling-languages.json";
// A mathematical reference paired with the Rust coordinate kernel and Kani harness.
export const coordinateTheory = "tests/formal/source-map/Coordinates.lean";
export const coordinateKernel = "davinci/vize_l4/src/write/source_map/indexed/geometry.rs";
export const coordinateProof = "davinci/vize_l4/src/write/source_map/indexed/kani.rs";
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
    if (path === coordinateTheory) return [];
    const language = retiredToolingLanguage(path);
    return language ? [{ path, language }] : [];
  });
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
