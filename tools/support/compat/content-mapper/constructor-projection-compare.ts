/** Compare two complete constructor-projection populations through the shared certifier. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { comparePackets, readAssets, type Row } from "./constructor-projection-parity.ts";

export function compareConstructorProjectionFiles(
  baseFile: string,
  headFile: string,
  baseRoot: string,
  headRoot: string,
) {
  const rows = (file: string): Row[] => JSON.parse(readFileSync(file, "utf8"));
  return comparePackets(rows(baseFile), rows(headFile), readAssets(baseRoot, headRoot));
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [baseFile, headFile, baseRoot, headRoot] = process.argv.slice(2);
  assert.ok(baseFile && headFile && baseRoot && headRoot && process.argv.length === 6);
  console.log(
    JSON.stringify(compareConstructorProjectionFiles(baseFile, headFile, baseRoot, headRoot)),
  );
}
