import * as fs from "node:fs";
import * as path from "node:path";

/**
 * Simple CLI snapshot testing.
 * Compares actual output against a stored snapshot file.
 * If the snapshot doesn't exist, creates it (first run).
 * If UPDATE_SNAPSHOTS=1, overwrites existing snapshots.
 */
export function assertSnapshot(snapshotDir: string, name: string, actual: string): void {
  fs.mkdirSync(snapshotDir, { recursive: true });
  const snapshotPath = path.join(snapshotDir, `${name}.snap`);

  if (process.env.UPDATE_SNAPSHOTS || !fs.existsSync(snapshotPath)) {
    fs.writeFileSync(snapshotPath, actual);
    console.log(
      `Snapshot ${process.env.UPDATE_SNAPSHOTS ? "updated" : "created"}: ${snapshotPath}`,
    );
    return;
  }

  const expected = fs.readFileSync(snapshotPath, "utf-8");
  if (actual !== expected) {
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
        `Run with UPDATE_SNAPSHOTS=1 to update.\n\n` +
        diffLines.slice(0, 30).join("\n"),
    );
  }

  console.log(`Snapshot matched: ${snapshotPath}`);
}
