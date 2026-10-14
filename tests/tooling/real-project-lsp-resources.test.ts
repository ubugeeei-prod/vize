import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { test } from "node:test";
import { sampleSessionResources } from "./support/real-project-lsp-resources.ts";

test(
  "resource sampling covers the whole process tree of a real session",
  { skip: process.platform === "win32" },
  async () => {
    // A parent with one live child stands in for `vize lsp` and a tsgo worker.
    const parent = spawn(
      process.execPath,
      [
        "-e",
        "require('node:child_process').spawn(process.execPath, ['-e', 'setTimeout(() => {}, 30000)'], { stdio: 'ignore' }); setTimeout(() => {}, 30000);",
      ],
      { stdio: "ignore" },
    );
    try {
      await new Promise((resolve) => setTimeout(resolve, 500));
      const stop = sampleSessionResources({ processId: parent.pid }, 50);
      await new Promise((resolve) => setTimeout(resolve, 200));
      const evidence = stop();
      assert.ok(evidence, "a live process tree yields evidence");
      assert.ok(evidence.samples >= 2, `samples: ${evidence.samples}`);
      assert.ok(evidence.peakRssKiB > 0, `peak RSS: ${evidence.peakRssKiB}`);
      assert.ok(evidence.peakProcesses >= 2, `processes: ${evidence.peakProcesses}`);
    } finally {
      parent.kill("SIGKILL");
    }
  },
);

test("sessions without an OS process report no resources", () => {
  assert.equal(sampleSessionResources({})(), null);
  const throwing = {
    get processId(): number {
      throw new Error("vize lsp process id is unavailable");
    },
  };
  assert.equal(sampleSessionResources(throwing)(), null);
});
