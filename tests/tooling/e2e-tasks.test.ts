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
  const config = readRepoFile("vite.config.ts");

  assert.match(config, /e2e:\s*\[/);
  assert.match(config, /"tests\/app\/\*\*"/);
  assert.match(config, /"tests\/_helpers\/\*\*"/);
  assert.match(
    config,
    /"test:e2e":\s*noCacheTask\(runTasks\("test:e2e:dev", "test:e2e:preview"\)\)/,
  );
  assert.match(config, /"test:e2e:dev":\s*task\(runInPackages\("test:dev", \["\.\/tests"\]\)/);
  assert.match(
    config,
    /"test:e2e:preview":\s*task\(runInPackages\("test:preview", \["\.\/tests"\]\)/,
  );
  assert.match(config, /"test:e2e:vrt":\s*task\(runInPackages\("test:vrt", \["\.\/tests"\]\)/);
  assert.match(config, /input:\s*cacheInputs\.e2e/);
});
