import assert from "node:assert/strict";
import { test } from "node:test";
import { population } from "./template-alias-paired-11822-public.ts";
import type { Observation } from "./template-alias-paired-11822-types.ts";
import { normalHostArguments } from "./template-alias-paired-11822-hosts.ts";

void test("normal Go host calls admit the mapper without fast, width or profiling overrides", () => {
  const args = normalHostArguments("pinned-host", "authored-config.json");
  assert.deepEqual(args, [
    "pinned-host",
    "--project",
    "authored-config.json",
    "--runExternalCode",
    "--noEmit",
    "--incremental",
    "false",
    "--pretty",
    "false",
  ]);
  assert.ok(
    !args.some((arg) =>
      /^(?:--fast|--checkers|--extendedDiagnostics|--pprofDir)(?:=|$)/u.test(arg),
    ),
  );
});

void test("incomplete paired populations and invalid resource fields cannot publish medians", () => {
  const rows = [8, 1, 4, 3, 9].map(
    (value) =>
      ({
        usage: {
          wall_s: value,
          user_s: value / 2,
          system_s: value / 4,
          max_rss_kib: value * 1024,
          status: 1,
        },
      }) as Observation,
  );
  assert.deepEqual(population(rows, 5), {
    wallSeconds: 4,
    waitedUserSeconds: 2,
    waitedSystemSeconds: 1,
    maximumSingleProcessRssKiB: 4096,
  });
  assert.throws(() => population(rows.slice(0, 4), 5), /incomplete/);
  assert.throws(() => population(rows.slice(0, 4), 4));
  rows[0].usage.wall_s = Number.NaN;
  assert.throws(() => population(rows, 5));
});
