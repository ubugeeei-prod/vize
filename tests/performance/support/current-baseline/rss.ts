import assert from "node:assert/strict";
import { loadLspChurnBudget } from "../churn-metrics.ts";
import { elapsedMs, object, statistics } from "./wire.ts";

export function auditRss(
  rows: Record<string, unknown>[],
  start: Record<string, unknown>,
  expectedBinary: string,
  backend: Record<string, unknown>,
) {
  assert.equal(rows[0]?.kind, "start");
  assert.equal(rows[0].pid, start.pid);
  assert.equal(rows[0].startTicks, start.startTicks);
  assert.equal(rows[0].intervalMs, 50);
  const finish = rows.at(-1)!;
  assert.equal(finish.kind, "finish");
  assert.equal(finish.stopped, true);
  assert.equal(finish.failure, null);
  assert.deepEqual(finish.survivorsBeforeForcedCleanup, []);
  let last = BigInt(String(rows[0].ns));
  let rootObserved = false;
  let backendObserved = false;
  let rootMaxKiB = 0;
  let treeMaxKiB = 0;
  let processesMax = 0;
  const gapsMs: number[] = [];
  const known = new Set<string>([`${String(start.pid)}@${String(start.startTicks)}`]);
  for (const row of rows.slice(1, -1)) {
    assert.equal(row.kind, "sample");
    const ns = BigInt(String(row.ns));
    assert(ns >= last);
    gapsMs.push(Number(ns - last) / 1e6);
    last = ns;
    assert(Array.isArray(row.members));
    const members = row.members.map(object);
    const owned = new Set(
      members
        .filter((member) => known.has(`${String(member.pid)}@${String(member.startTicks)}`))
        .map((member) => member.pid),
    );
    for (let pass = 0; pass < members.length; pass += 1) {
      for (const member of members) if (owned.has(member.parentPid)) owned.add(member.pid);
    }
    const identities = new Set<string>();
    let total = 0;
    for (const member of members) {
      assert(
        Number.isSafeInteger(member.rssKiB) && Number(member.rssKiB) > 0,
        "missing RSS is not zero",
      );
      assert(Number.isSafeInteger(member.pid) && Number.isSafeInteger(member.parentPid));
      assert.match(String(member.startTicks), /^\d+$/);
      assert(owned.has(member.pid), "foreign process cannot enter server RSS tree");
      const identity = `${String(member.pid)}@${String(member.startTicks)}`;
      assert(!identities.has(identity));
      identities.add(identity);
      known.add(identity);
      total += Number(member.rssKiB);
      if (member.pid === start.pid) {
        assert.equal(member.startTicks, start.startTicks);
        assert.equal(member.executableSha256, expectedBinary);
        rootObserved = true;
        rootMaxKiB = Math.max(rootMaxKiB, Number(member.rssKiB));
      }
      if (member.executableSha256 === backend.sha256 && member.executable === backend.path)
        backendObserved = true;
    }
    treeMaxKiB = Math.max(treeMaxKiB, total);
    processesMax = Math.max(processesMax, members.length);
  }
  assert(
    rootObserved && backendObserved,
    "actual source server and exact real backend must be observed",
  );
  assert(gapsMs.length > 0);
  assert(BigInt(String(finish.ns)) >= last, "complete sampler footer follows final sample");
  const { budget } = loadLspChurnBudget("misskey-lsp-churn");
  assert(rootMaxKiB <= budget.maxPeakRssMiB * 1024);
  assert(treeMaxKiB <= budget.maxPeakProcessTreeRssMiB * 1024);
  assert(processesMax <= budget.maxProcessTreeSize);
  return {
    samples: gapsMs.length,
    rootMaxKiB,
    treeMaxKiB,
    processesMax,
    samplingGaps: statistics(gapsMs),
    firstSampleAfterSpawnMs: elapsedMs(String(start.spawnNs), String(rows[1].ns)),
    limits:
      "sampled maxima at actual intervals; missed short-lived processes, RSS sum is not PSS or true peak",
  };
}
