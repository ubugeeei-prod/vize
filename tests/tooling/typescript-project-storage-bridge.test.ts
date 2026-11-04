import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  movedProjectStorageIssues,
  projectStorageBridgePaths,
} from "./support/typescript-project-storage-bridge.ts";

const current = () =>
  Object.fromEntries(
    projectStorageBridgePaths.map((file) => [
      file,
      fs.readFileSync(new URL(`../../${file}`, import.meta.url), "utf8"),
    ]),
  );

test("the complete loader move retains exactly the eight original Carton std bridges", () => {
  const issues = movedProjectStorageIssues(current());
  assert.equal(issues.length, 8);
  assert.deepEqual(
    projectStorageBridgePaths.map(
      (file) => issues.filter((issue) => issue.startsWith(`${file}: `)).length,
    ),
    [3, 3, 2],
  );
});
test("absence of the move leaves the original stage policy unchanged", () => {
  assert.deepEqual(
    movedProjectStorageIssues(
      Object.fromEntries(projectStorageBridgePaths.map((file) => [file, null])),
    ),
    [],
  );
});
test("partial, changed, swapped and unknown storage sources retain no compatibility admission", () => {
  for (const file of projectStorageBridgePaths) {
    const missing: Record<string, string | null> = current();
    missing[file] = null;
    assert.throws(() => movedProjectStorageIssues(missing));
    const changed = current();
    changed[file] += "\n";
    assert.throws(() => movedProjectStorageIssues(changed));
    const extra = current();
    extra[file] += "\ntype Unknown = std::vec::Vec<u8>;\n";
    assert.throws(() => movedProjectStorageIssues(extra));
  }
  const swapped = current();
  swapped[projectStorageBridgePaths[0]] = swapped[projectStorageBridgePaths[1]];
  assert.throws(() => movedProjectStorageIssues(swapped));
  assert.throws(() => movedProjectStorageIssues({ ...current(), "unknown.rs": "" }));
});
