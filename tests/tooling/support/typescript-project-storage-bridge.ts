import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { scanStorage } from "../level-storage-scan.ts";

export const projectStorageBridgePaths = [
  "levels/vize_l0/src/path.rs",
  "levels/vize_l0/src/typescript_project.rs",
  "levels/vize_l0/src/typescript_project/rebase.rs",
] as const;
const bytes = readFileSync(
  new URL("../../_fixtures/tooling/typescript-project-storage-bridge-11794.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(bytes).digest("hex"),
  "b14ba21ecf3c523589fc6425909fb446ab4b17258287217bc415020362251ebd",
);
const record = JSON.parse(bytes.toString("utf8")) as {
  schema: string;
  files: {
    from: string;
    to: string;
    referenceBlob: string;
    currentBlob: string;
    transformation: string;
    referenceSource: string;
    issues: string[];
  }[];
};
assert.equal(record.schema, "vize.typescript-project-storage-bridge.v1");
assert.deepEqual(
  record.files.map((file) => file.to),
  projectStorageBridgePaths,
);
const blob = (source: string) =>
  createHash("sha1")
    .update(`blob ${Buffer.byteLength(source)}\0`)
    .update(source)
    .digest("hex");

/** Account for the same Carton std bridges only after authenticating every moved body. */
export function movedProjectStorageIssues(sources: Record<string, string | null>): string[] {
  assert.deepEqual(Object.keys(sources).sort(), [...projectStorageBridgePaths].sort());
  // With no moved files the original stage policy remains exactly unchanged.
  if (Object.values(sources).every((source) => source === null)) return [];
  return record.files.flatMap((file) => {
    const current = sources[file.to];
    assert.ok(typeof current === "string", "incomplete project loader storage move");
    assert.equal(blob(current), file.currentBlob, `complete moved storage source: ${file.to}`);
    const restored =
      file.transformation === "counter-crate-qualifier"
        ? current.replaceAll("crate::micro_count!", "vize_l0::micro_count!")
        : file.transformation === "retain-carton-tests"
          ? `${current}\n#[cfg(test)]\nmod tests;\n`
          : current;
    assert.equal(restored, file.referenceSource, `complete original Carton source: ${file.from}`);
    assert.equal(blob(restored), file.referenceBlob);
    const originalIssues = scanStorage(restored).issues;
    assert.deepEqual(scanStorage(current).issues, originalIssues);
    assert.deepEqual(
      file.issues,
      originalIssues.map((issue) => `${file.to}: ${issue}`),
    );
    return file.issues;
  });
}
