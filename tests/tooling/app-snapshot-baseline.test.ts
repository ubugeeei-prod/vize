import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { assertSnapshot } from "../_helpers/snapshot.ts";

test("app snapshot verification never creates or rewrites an unapproved baseline", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-snapshot-"));
  const previous = process.env.UPDATE_SNAPSHOTS;
  const baseline = path.join(directory, "app.snap");
  try {
    for (const value of [undefined, "0", "false"]) {
      if (value === undefined) delete process.env.UPDATE_SNAPSHOTS;
      else process.env.UPDATE_SNAPSHOTS = value;
      assert.throws(
        () => assertSnapshot(directory, "app", "unexpected"),
        /Missing snapshot baseline/,
      );
      assert.equal(fs.existsSync(baseline), false);
    }
    process.env.UPDATE_SNAPSHOTS = "1";
    assertSnapshot(directory, "app", "reviewed");
    assert.equal(fs.readFileSync(baseline, "utf8"), "reviewed");
    for (const value of [undefined, "0", "false"]) {
      if (value === undefined) delete process.env.UPDATE_SNAPSHOTS;
      else process.env.UPDATE_SNAPSHOTS = value;
      assertSnapshot(directory, "app", "reviewed");
      assert.throws(() => assertSnapshot(directory, "app", "regression"), /Snapshot mismatch/);
      assert.equal(fs.readFileSync(baseline, "utf8"), "reviewed");
    }
    process.env.UPDATE_SNAPSHOTS = "1";
    assertSnapshot(directory, "app", "new reviewed baseline");
    assert.equal(fs.readFileSync(baseline, "utf8"), "new reviewed baseline");
  } finally {
    if (previous === undefined) delete process.env.UPDATE_SNAPSHOTS;
    else process.env.UPDATE_SNAPSHOTS = previous;
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
