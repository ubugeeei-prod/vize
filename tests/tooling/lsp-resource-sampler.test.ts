import assert from "node:assert/strict";
import { test } from "node:test";
import {
  createProcessHistory,
  ownedProcessTree,
  type ProcessEntry,
} from "../../tools/support/compat/davinci/lib/lsp-resource-sampler.ts";

const entry = (
  pid: number,
  parent: number,
  birth_ticks: number,
  command: string,
  executable: string,
  rss_bytes = 100,
  cpu_seconds = 1,
): ProcessEntry => ({
  pid,
  parent,
  birth_ticks,
  command,
  executable,
  rss_bytes,
  cpu_seconds,
});

test("the resource sampler retains a descendant that exits before the final sample", () => {
  const history = createProcessHistory();
  const root = entry(10, 1, 100, "vize", "/bin/vize");
  const forked = entry(20, 10, 200, "vize", "/bin/vize", 20, 0);
  const corsa = entry(20, 10, 200, "tsc", "/bin/tsc", 400, 3);

  history.observe(0, [root, forked]);
  history.observe(1, [root, corsa]);
  history.observe(2, [root]);

  const descendants = history.entries().filter((entry) => entry.pid !== root.pid);
  assert.deepEqual(descendants, [
    {
      pid: 20,
      birth_ticks: 200,
      first_sample: 0,
      last_sample: 1,
      samples: 2,
      parent_pids: [10],
      commands: ["vize", "tsc"],
      executables: ["/bin/vize", "/bin/tsc"],
      peak_rss_bytes: 400,
      last_rss_bytes: 400,
      cpu_seconds: 3,
    },
  ]);
});

test("PID reuse creates a new identity and a reused root cannot qualify", () => {
  const history = createProcessHistory();
  history.observe(0, [entry(20, 10, 200, "tsc", "/bin/tsc")]);
  history.observe(1, [entry(20, 10, 201, "other", "/bin/other")]);
  assert.deepEqual(
    history.entries().map((entry) => [entry.pid, entry.birth_ticks, entry.commands]),
    [
      [20, 200, ["tsc"]],
      [20, 201, ["other"]],
    ],
  );

  const reusedRoot = new Map([
    [10, entry(10, 1, 101, "unrelated", "/bin/unrelated")],
    [20, entry(20, 10, 200, "tsc", "/bin/tsc")],
  ]);
  assert.throws(
    () => ownedProcessTree(reusedRoot, { pid: 10, birth_ticks: 100 }),
    /PID was reused/u,
  );
});

test("only a live chain to the original root is selected", () => {
  const root = entry(10, 1, 100, "vize", "/bin/vize");
  const child = entry(20, 10, 200, "tsc", "/bin/tsc");
  const grandchild = entry(30, 20, 300, "worker", "/bin/worker");
  const reparented = entry(40, 1, 400, "retired", "/bin/retired");
  const unrelated = entry(50, 1, 500, "other", "/bin/other");
  const table = new Map(
    [root, child, grandchild, reparented, unrelated].map((entry) => [entry.pid, entry]),
  );
  assert.deepEqual(
    ownedProcessTree(table, root).map((entry) => entry.pid),
    [10, 20, 30],
  );
});
