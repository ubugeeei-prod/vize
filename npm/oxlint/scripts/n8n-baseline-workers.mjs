import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { baselinePartitions, seedOriginalSources } from "./n8n-baseline-partitions.mjs";
import { runBaselineProcesses } from "./n8n-baseline-processes.mjs";
import { joinBaselinePartitions } from "./n8n-baseline-join.mjs";
import { sha256 } from "./n8n-replay-inputs.mjs";

export async function runBaselineWorkers({ packageDir, output, custody, inventory, environment }) {
  assert.equal(sha256(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
  const started = performance.now();
  const checkDeadline = () =>
    assert.ok(performance.now() - started < 900_000, "shared original 900s deadline");
  const seeded = seedOriginalSources(inventory, custody.sources, checkDeadline);
  seeded.budget = { deadlineMs: 900_000, preparationMs: performance.now() - started };
  fs.writeFileSync(
    path.join(output, "baseline-source-store.json"),
    JSON.stringify(seeded, null, 2) + "\n",
  );
  const jobs = baselinePartitions(inventory.files.length).map((partition) => {
    const directory = path.join(output, `baseline-worker-${partition.index}`);
    fs.mkdirSync(directory);
    const calls = path.join(directory, "native-calls.jsonl");
    const configuration = path.join(directory, "custody.json");
    // Input backing files are complete before any worker starts. Only the last
    // worker creates additional cache-control source hashes; ledgers are private.
    fs.writeFileSync(configuration, JSON.stringify({ ...custody, calls }, null, 2) + "\n");
    return {
      index: partition.index,
      partition,
      output: directory,
      configuration,
      calls,
      command: "vp",
      args: ["test", "run", "src/n8n-full-replay.test.ts"],
      cwd: packageDir,
      env: {
        ...process.env,
        ...environment,
        VIZE_N8N_NATIVE_CUSTODY: configuration,
        VIZE_N8N_REPLAY_OUTPUT: path.join(directory, "bridge"),
        VIZE_N8N_BASELINE_PARTITION: JSON.stringify(partition),
      },
    };
  });
  const result = await runBaselineProcesses(jobs, output, { startedAt: started });
  fs.writeFileSync(path.join(output, "bridge-tests.json"), JSON.stringify(result, null, 2) + "\n");
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0);
  assert.equal(sha256(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
  fs.mkdirSync(path.join(output, "bridge"));
  joinBaselinePartitions(inventory, custody, jobs, output);
  return result;
}
