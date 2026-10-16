import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import { isReaped, observeReaped } from "../performance/support/warm-type-backed-processes.ts";

const recovered = { pid: 12187, parent: 11749, birth_ticks: 65896 };

test("the real original400 same-birth zombie witness fails actual reaping", () => {
  for (const state of ["R", "S", "D", "Z", "X"])
    assert.equal(isReaped(recovered, { birth_ticks: recovered.birth_ticks, state }), false);
  assert.equal(isReaped(recovered, null), true);
  assert.equal(isReaped(recovered, { birth_ticks: recovered.birth_ticks + 1, state: "Z" }), true);
});

test("negative candidate reaping retains every same-birth zombie probe until the unchanged bound", async () => {
  let elapsed = 0;
  const result = await observeReaped([recovered], {
    current: () => ({ birth_ticks: recovered.birth_ticks, state: "Z" }),
    now: () => elapsed,
    pause: async () => {
      elapsed += 1_000;
    },
  });
  assert.equal(result.reaped, false);
  assert.equal(result.timeoutMs, 5_000);
  assert.deepEqual(
    result.probes.map((probe) => probe.elapsedMs),
    [0, 1_000, 2_000, 3_000, 4_000, 5_000],
  );
  for (const probe of result.probes)
    assert.deepEqual(probe.observed, [
      { ...recovered, current: { birth_ticks: recovered.birth_ticks, state: "Z" } },
    ]);
});

test("candidate reaping waits for actual absence or PID reuse and preserves the preceding zombie", async () => {
  for (const terminal of [null, { birth_ticks: recovered.birth_ticks + 1, state: "S" }]) {
    let elapsed = 0;
    const result = await observeReaped([recovered], {
      current: () =>
        elapsed === 0 ? { birth_ticks: recovered.birth_ticks, state: "Z" } : terminal,
      now: () => elapsed,
      pause: async () => {
        elapsed += 25;
      },
    });
    assert.equal(result.reaped, true);
    assert.equal(result.probes.length, 2);
    assert.equal(result.probes[0].observed[0].current?.state, "Z");
    assert.deepEqual(result.probes[1].observed[0].current, terminal);
  }
  await assert.rejects(observeReaped([]), /actual identity/u);
});

test("permission and malformed proc reads are retained as failure, never mistaken for absence", async () => {
  for (const failure of [
    new Error("EACCES reading /proc identity"),
    new Error("malformed birth ticks"),
  ]) {
    const result = await observeReaped([recovered], {
      current: () => {
        throw failure;
      },
      now: () => 0,
      pause: async () => {
        throw new Error("a failed read cannot become successful reaping");
      },
    });
    assert.equal(result.reaped, false);
    assert.equal(result.probes.length, 1);
    assert.match(result.probes[0].observed[0].readError ?? "", new RegExp(failure.message));
    assert.equal(result.probes[0].observed[0].current, undefined);
  }
});

test("source/cut candidates and installed-public runs require reaping while baseline evidence stays whole", () => {
  const requests = fs.readFileSync(
    new URL("../performance/support/warm-type-backed-requests.ts", import.meta.url),
    "utf8",
  );
  const publicRunner = fs.readFileSync(
    new URL("../performance/support/warm-type-backed-public.ts", import.meta.url),
    "utf8",
  );
  assert.match(requests, /await awaitRetired\(\[\.\.\.natives\.values\(\)\]\)/u);
  assert.match(requests, /side === "after"/u);
  assert.match(requests, /nativeReaping = await observeReaped\(identities\)/u);
  assert.match(publicRunner, /launch,\s+false,\s+true,/u);
});
