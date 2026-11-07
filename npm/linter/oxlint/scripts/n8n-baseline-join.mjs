import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  baselineModes,
  baselinePartitions,
  validatePartitionRows,
} from "./n8n-baseline-partitions.mjs";
import { effectiveRules, manifest, sha256, verifyCorpus } from "./n8n-replay-inputs.mjs";

export function joinBaselinePartitions(inventory, custody, jobs, output) {
  assert.equal(jobs.length, 2);
  assert.deepEqual(verifyCorpus(), inventory);
  const partitions = baselinePartitions(inventory.files.length);
  const vectorBytes = [],
    nativeBytes = [];
  const totals = {};
  let limits;
  for (const [index, job] of jobs.entries()) {
    assert.deepEqual(job.partition, partitions[index]);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(job.output, "bridge/inventory.json"))),
      inventory,
    );
    const summary = JSON.parse(
      fs.readFileSync(path.join(job.output, "bridge/bridge-summary.json")),
    );
    assert.deepEqual(summary.partition, job.partition);
    assert.equal(summary.files, inventory.files.length);
    assert.equal(summary.scriptless, manifest.corpus.scriptlessFiles.length);
    assert.deepEqual(summary.disabledFiles, inventory.disabledFiles);
    if (limits) assert.deepEqual(summary.limits, limits);
    else limits = summary.limits;
    const bytes = fs.readFileSync(path.join(job.output, "bridge/bridge-vectors.jsonl"));
    assert.equal(sha256(bytes), summary.vectorSha256);
    assert.ok(bytes.length > 0 && bytes.at(-1) === 10, "complete append-only vector rows");
    const rows = bytes
      .toString("utf8")
      .split("\n")
      .slice(0, -1)
      .map((line) => JSON.parse(line));
    const checkedTotals = validatePartitionRows(inventory, job.partition, rows, (file, mode) =>
      mode.startsWith("frozen") ? manifest.adoption.rules : effectiveRules(file),
    );
    assert.deepEqual(summary.totals, checkedTotals);
    for (const mode of baselineModes) {
      const total = (totals[mode] ??= { coldCalls: 0, warmCalls: 0, elapsedMs: 0 });
      total.coldCalls += summary.totals[mode].coldCalls;
      total.warmCalls += summary.totals[mode].warmCalls;
      total.elapsedMs += summary.totals[mode].elapsedMs;
    }
    const configuration = JSON.parse(fs.readFileSync(job.configuration));
    assert.deepEqual(configuration, { ...custody, calls: job.calls });
    vectorBytes.push(bytes);
    const journal = fs.readFileSync(job.calls);
    assert.ok(journal.length > 0 && journal.at(-1) === 10, "complete native ledger");
    nativeBytes.push(journal);
    const controls = path.join(job.output, "bridge/cache-controls.jsonl");
    assert.equal(
      fs.existsSync(controls),
      job.partition.controls,
      "all original controls exactly once",
    );
    if (job.partition.controls)
      fs.copyFileSync(
        controls,
        path.join(output, "bridge/cache-controls.jsonl"),
        fs.constants.COPYFILE_EXCL,
      );
  }
  assert.deepEqual(
    fs.readFileSync(path.join(jobs[0].output, "bridge/catalog.json")),
    fs.readFileSync(path.join(jobs[1].output, "bridge/catalog.json")),
  );
  for (const name of ["inventory.json", "catalog.json"])
    fs.copyFileSync(
      path.join(jobs[0].output, "bridge", name),
      path.join(output, "bridge", name),
      fs.constants.COPYFILE_EXCL,
    );
  const vectors = Buffer.concat(vectorBytes);
  fs.writeFileSync(path.join(output, "bridge/bridge-vectors.jsonl"), vectors, { flag: "wx" });
  // Native arguments/results/PIDs/source paths remain byte-exact. Existing
  // finishN8nReplay still verifies every ledger event and original call count.
  fs.writeFileSync(custody.calls, Buffer.concat(nativeBytes), { flag: "wx" });
  fs.writeFileSync(
    path.join(output, "bridge/bridge-summary.json"),
    JSON.stringify(
      {
        totals,
        files: inventory.files.length,
        scriptless: manifest.corpus.scriptlessFiles.length,
        disabledFiles: inventory.disabledFiles,
        vectorSha256: sha256(vectors),
        limits,
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  assert.deepEqual(verifyCorpus(), inventory);
}
