import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  assessCampaign,
  type SessionAssessment,
} from "../performance/support/current-baseline/assessment.ts";
import {
  prepareCampaignDirectory,
  writeJson,
} from "../performance/support/current-baseline/inputs.ts";

// These are synthetic aggregation controls, not actual server evidence.
function sessions(): SessionAssessment[] {
  const report = {
    lanes: { leafBroken: [1, 3], leafRepaired: [2, 4], completion: [1, 2] },
    startup: { spawnToReadyMs: 10, coldOpenMs: 20 },
  } as never;
  return [1, 2, 3].map((index) => ({ index, status: "passed", failure: null, report }));
}

void test("closed three-session aggregate retains all samples with startup tail explicitly unknown", () => {
  const result = assessCampaign(sessions());
  assert.equal(result.status, "passed");
  assert.equal(result.warm?.completion.n, 6);
  assert.equal(result.startup?.spawnToReady.n, 3);
  assert.equal(result.startup?.spawnToReady.p95Ms, null);
});

for (const kind of [
  "missing",
  "failed",
  "duplicate",
  "extra",
  "partial",
  "failure-with-passed-label",
]) {
  void test(`aggregate refuses ${kind} session instead of accepting a favorable subset`, () => {
    const rows = sessions();
    if (kind === "missing") rows.pop();
    if (kind === "failed") {
      rows[1].status = "failed";
      rows[1].failure = "timeout retained";
    }
    if (kind === "duplicate") rows[1].index = 1;
    if (kind === "extra") rows.push(rows[0]);
    if (kind === "partial") rows[1].report = null;
    if (kind === "failure-with-passed-label") rows[1].failure = "oracle failure retained";
    const result = assessCampaign(rows);
    assert.equal(result.status, "failed");
    assert.equal(result.warm, null);
    assert.equal(result.startup, null);
    assert.equal(result.timeoutCount, null);
    assert.deepEqual(result.sessions, rows);
  });
}

void test("a valid write followed by stale retry refuses and removes earlier public acceptance", () => {
  const parent = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-baseline-stale-law-"));
  const output = path.join(parent, "campaign");
  try {
    prepareCampaignDirectory(output);
    writeJson(path.join(output, "assessment.json"), {
      status: "passed",
      source: "synthetic previous",
    });
    fs.writeFileSync(path.join(output, "client.bin"), "raw previous evidence");
    assert.throws(() => prepareCampaignDirectory(output), /stale campaign/);
    assert(!fs.existsSync(path.join(output, "assessment.json")));
    assert.equal(fs.readFileSync(path.join(output, "client.bin"), "utf8"), "raw previous evidence");
  } finally {
    fs.rmSync(parent, { recursive: true, force: true });
  }
});
