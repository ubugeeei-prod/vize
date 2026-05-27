import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

test("workspace exposes app e2e task aliases with scoped cache inputs", () => {
  const taskInputs = readRepoFile("tools/vite-plus/task-inputs.ts");
  const taskGroups = readRepoFile("tools/vite-plus/tasks/test-benchmark.ts");
  const testPackage = JSON.parse(readRepoFile("tests", "package.json")) as {
    scripts?: Record<string, string>;
  };

  assert.match(taskInputs, /e2e:\s*\[/);
  assert.match(taskInputs, /"tests\/app\/\*\*"/);
  assert.match(taskInputs, /"tests\/_helpers\/\*\*"/);
  assert.match(taskInputs, /"tests\/_fixtures\/\*\*"/);
  assert.match(taskInputs, /"tests\/snapshots\/\*\*"/);
  assert.match(
    taskGroups,
    /"test:e2e":\s*noCacheTask\(runTasks\("test:e2e:dev", "test:e2e:preview"\)\)/,
  );
  assert.match(taskGroups, /"test:e2e:dev":\s*task\(runInPackages\("test:dev", \["\.\/tests"\]\)/);
  assert.match(
    taskGroups,
    /"test:e2e:preview":\s*task\(runInPackages\("test:preview", \["\.\/tests"\]\)/,
  );
  assert.match(taskGroups, /"test:e2e:vrt":\s*task\(runInPackages\("test:vrt", \["\.\/tests"\]\)/);
  assert.match(taskGroups, /input:\s*cacheInputs\.e2e/);
  assert.match(testPackage.scripts?.["test:dev:ci"] ?? "", /app\/dev\/nuxt-ui\.spec\.ts/);
});
