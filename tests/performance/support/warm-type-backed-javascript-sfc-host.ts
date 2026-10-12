import {
  javascriptSfcBodyPairs,
  javascriptSfcProductionPaths,
} from "../../../tools/support/compat/github/javascript-sfc-source-context.mjs";

type Digest = (side: "before" | "after", file: string) => string | null;

/** Exact actual JS source pairs; original400 must still execute every original vector. */
export function qualifyJavaScriptSfcHost(
  production: string[],
  qualified: ReadonlySet<string>,
  digest: Digest,
) {
  if (new Set(production).size !== production.length) return null;
  const files = new Set(javascriptSfcProductionPaths);
  if (production.every((file) => qualified.has(file))) return null;
  if (production.some((file) => !qualified.has(file) && !files.has(file))) return null;
  if (
    JSON.stringify(
      production.filter((file) => files.has(file)).toSorted((a, b) => a.localeCompare(b)),
    ) !== JSON.stringify([...files].toSorted((a, b) => a.localeCompare(b)))
  )
    return null;
  for (const [file, before, after] of javascriptSfcBodyPairs)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    authority: "complete-javascript-sfc-source-pairs",
    completeBodyPairs: javascriptSfcBodyPairs.length,
    runtimeCredit: false,
    original400Credit: false,
    performanceCredit: false,
    files: [...files],
  };
}
