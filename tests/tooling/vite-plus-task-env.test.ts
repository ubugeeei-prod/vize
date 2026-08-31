import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { testAndBenchmarkTasks } from "../../config/vite-plus/tasks/test-benchmark.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type TaskShape = {
  env?: string[];
};

test("source coverage tracks the Nuxt stress iteration environment", () => {
  const coverage = testAndBenchmarkTasks["coverage:source"] as TaskShape;

  assert.deepEqual(coverage.env, ["VIZE_NUXT_CONFIG_ITERATIONS"]);
});

test("Vite Plus forwards tracked task environment without leaking an undeclared probe", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vp-task-env-"));
  const output = path.join(root, "env.json");
  const vp = process.env.VIZE_VP_BIN ?? path.join(repoRoot, "node_modules", ".bin", "vp");

  try {
    fs.writeFileSync(path.join(root, "package.json"), '{"name":"vize-task-env-probe"}\n');
    fs.writeFileSync(
      path.join(root, "probe.mjs"),
      `import fs from "node:fs";\nfs.writeFileSync(${JSON.stringify(output)}, JSON.stringify({ tracked: process.env.VIZE_TRACKED_ENV ?? null, hidden: process.env.VIZE_HIDDEN_ENV ?? null }));\n`,
    );
    fs.writeFileSync(
      path.join(root, "vite.config.mjs"),
      'export default { run: { tasks: { probe: { command: "node probe.mjs", env: ["VIZE_TRACKED_ENV"] } } } };\n',
    );

    const result = spawnSync(vp, ["run", "--workspace-root", "probe"], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        VIZE_TRACKED_ENV: "forwarded",
        VIZE_HIDDEN_ENV: "must-not-leak",
      },
    });

    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.deepEqual(JSON.parse(fs.readFileSync(output, "utf8")), {
      tracked: "forwarded",
      hidden: null,
    });
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});
