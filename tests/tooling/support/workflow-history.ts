import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { workflowPredecessor } from "../../../tools/support/ci/workflow-program-history.ts";
const root = fileURLToPath(new URL("../../../", import.meta.url));

/** Byte-for-byte predecessor used by historical inverse and release-identity contracts. */
export function archivedWorkflow(...parts: (string | URL)[]): string {
  if (parts.at(-1) === "utf8") parts.pop();
  const file =
    parts[0] instanceof URL ? fileURLToPath(parts[0]) : path.resolve(root, ...(parts as string[]));
  assert.equal(path.dirname(file), path.join(root, ".github/workflows"));
  return workflowPredecessor(path.basename(file, ".yml"), readFileSync(file), (relative) =>
    readFileSync(path.join(root, relative)),
  ).toString("utf8");
}
