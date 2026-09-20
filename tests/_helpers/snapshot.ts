import * as fs from "node:fs";
import * as path from "node:path";

/**
 * Simple CLI snapshot testing.
 * Compares actual output against a stored snapshot file.
 * UPDATE_SNAPSHOTS=1 explicitly creates or updates a baseline.
 * Missing baselines fail verification, including the first CI run.
 */
export function assertSnapshot(snapshotDir: string, name: string, actual: string): void {
  fs.mkdirSync(snapshotDir, { recursive: true });
  const snapshotPath = path.join(snapshotDir, `${name}.snap`);
  const actualPath = `${snapshotPath}.actual`;

  if (process.env.UPDATE_SNAPSHOTS === "1") {
    fs.writeFileSync(snapshotPath, actual);
    fs.rmSync(actualPath, { force: true });
    console.log(`Snapshot updated: ${snapshotPath}`);
    return;
  }

  if (!fs.existsSync(snapshotPath)) {
    fs.writeFileSync(actualPath, actual);
    throw new Error(
      `Missing snapshot baseline: ${snapshotPath}\nRun with UPDATE_SNAPSHOTS=1 to create it for review.`,
    );
  }

  const expected = fs.readFileSync(snapshotPath, "utf-8");
  if (actual !== expected) {
    // Preserve the complete result for CI review; console diffs are bounded.
    fs.writeFileSync(actualPath, actual);
    const diffLines: string[] = [];
    const actualLines = actual.split("\n");
    const expectedLines = expected.split("\n");
    const maxLines = Math.max(actualLines.length, expectedLines.length);
    for (let i = 0; i < maxLines; i++) {
      if (actualLines[i] !== expectedLines[i]) {
        diffLines.push(`  Line ${i + 1}:`);
        diffLines.push(`    - ${expectedLines[i] ?? "(missing)"}`);
        diffLines.push(`    + ${actualLines[i] ?? "(missing)"}`);
      }
    }
    throw new Error(
      `Snapshot mismatch: ${snapshotPath}\n` +
        `Complete actual result: ${actualPath}\n` +
        `Run with UPDATE_SNAPSHOTS=1 to update.\n\n` +
        diffLines.slice(0, 30).join("\n"),
    );
  }

  fs.rmSync(actualPath, { force: true });
  console.log(`Snapshot matched: ${snapshotPath}`);
}
