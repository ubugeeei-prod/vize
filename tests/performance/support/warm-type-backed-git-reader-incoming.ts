import { createHash } from "node:crypto";
import fs from "node:fs";

type Pair = readonly [string, string | null, string | null];
const manifests: ReadonlyArray<readonly [string, string]> = [
  [
    "warm-type-backed-git-reader-incoming.json",
    "dd570005abfeb4627bd4a80ec8c8b708ea57add07a068722100b237ef7b92b80",
  ],
  [
    "warm-type-backed-git-reader-family-1.json",
    "9387f68fbe7dfa6939e0cbea9a479158e19e072a275a568de62c3475d6f5e193",
  ],
  [
    "warm-type-backed-git-reader-family-2.json",
    "c999edc4c12145306972c056a2d4ae7da06f960658a0935f3a918da1015aa326",
  ],
];
const prefix = "tests/performance/support/";
const loaded = manifests.map(([name, expected]) => {
  const body = fs.readFileSync(new URL(name, import.meta.url));
  if (createHash("sha256").update(body).digest("hex") !== expected)
    throw new Error(`complete incoming reader manifest changed: ${name}`);
  return JSON.parse(body.toString()) as { files: Pair[] };
});
export const gitReaderIncomingSource = "dd188515f8ec3c51662b4f4d69adcd39c86a0907";
export const gitReaderIncomingGuard = `${prefix}warm-type-backed-git-reader-incoming.ts`;
export const gitReaderIncomingLaw = "tests/tooling/warm-type-backed-git-reader-incoming.test.ts";
export const gitReaderIncomingPairs: readonly Pair[] = [
  ...loaded.flatMap(({ files }) => files),
  ...manifests.map(([name, after]) => [prefix + name, null, after] as const),
];
export const gitReaderIncomingChanges = [
  ...loaded[0].files.slice(0, 3).map(([file]) => file),
  gitReaderIncomingLaw,
  gitReaderIncomingGuard,
  ...manifests.map(([name]) => prefix + name),
];

/** The preset successor is a separate complete source tuple, never the old measured host. */
export function qualifyGitReaderIncomingHarness(
  changed: readonly string[],
  alreadyQualified: ReadonlySet<string>,
  digest: (side: "before" | "after", file: string) => string | null,
) {
  const finite = new Set(gitReaderIncomingChanges);
  if (
    new Set(changed).size !== changed.length ||
    gitReaderIncomingChanges.some((file) => !changed.includes(file)) ||
    changed.some((file) => !alreadyQualified.has(file) && !finite.has(file)) ||
    new Set(gitReaderIncomingPairs.map(([file]) => file)).size !== gitReaderIncomingPairs.length
  )
    return null;
  for (const [file, before, after] of gitReaderIncomingPairs)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    authority: "complete-incoming-raw-git-reader-harness",
    incomingSource: gitReaderIncomingSource,
    completeBodyPairs: gitReaderIncomingPairs.length,
    namedFamilyBodies: 70,
    acceptanceCredit: "none",
    files: gitReaderIncomingChanges,
  };
}
