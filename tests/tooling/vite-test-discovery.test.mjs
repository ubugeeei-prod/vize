import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const manifest = JSON.parse(fs.readFileSync(path.join(root, "npm/builder/vite/package.json")));

void test("the actual Vite package command discovers new tests and preserves failures", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-test-discovery-"));
  const records = path.join(directory, "observed.jsonl");
  try {
    fs.mkdirSync(path.join(directory, "scripts"));
    fs.writeFileSync(
      path.join(directory, "package.json"),
      JSON.stringify({
        type: "module",
        scripts: { "test:prepared": manifest.scripts["test:prepared"] },
      }),
    );
    // Observe the preparation boundary without building or loading a native
    // image. The existing js-native-prepared tests own its real native contract.
    fs.writeFileSync(
      path.join(directory, "scripts/run-tests.mjs"),
      `
import assert from 'node:assert/strict';
import fs from 'node:fs';
assert.deepEqual(process.argv.slice(2), ['--require-prepared']);
fs.appendFileSync(process.env.VIZE_DISCOVERY_RECORDS, JSON.stringify({phase:'prepared'})+String.fromCharCode(10));
process.exit(Number(process.env.VIZE_DISCOVERY_PREPARE_STATUS || 0));
`,
    );
    const writeTest = (file, fails = false) => {
      const target = path.join(directory, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(
        target,
        `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { test } from 'node:test';
test(${JSON.stringify(file)}, () => {
  fs.appendFileSync(process.env.VIZE_DISCOVERY_RECORDS, JSON.stringify({phase:'test',file:${JSON.stringify(file)}})+String.fromCharCode(10));
  assert.equal(${fails}, false);
});
`,
      );
    };
    const run = (prepareStatus = 0) => {
      fs.rmSync(records, { force: true });
      const result = spawnSync(process.execPath, ["--run", "test:prepared"], {
        cwd: directory,
        encoding: "utf8",
        timeout: 30_000,
        env: {
          ...process.env,
          NODE_OPTIONS: undefined,
          NODE_TEST_CONTEXT: undefined,
          VIZE_TEST_CASES: undefined,
          VIZE_TEST_CASES_ROOT: undefined,
          TEST_CASES_JOB: undefined,
          PATH: `${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH ?? ""}`,
          VIZE_DISCOVERY_RECORDS: records,
          VIZE_DISCOVERY_PREPARE_STATUS: String(prepareStatus),
        },
      });
      assert.equal(result.error, undefined, result.error?.message);
      assert.equal(result.signal, null);
      const observed = fs.readFileSync(records, "utf8").trim().split("\n").map(JSON.parse);
      assert.deepEqual(observed[0], { phase: "prepared" });
      return { result, observed };
    };
    const expectFiles = (observed, files) => {
      assert.deepEqual(
        observed.slice(1).sort((left, right) => left.file.localeCompare(right.file)),
        [...files]
          .sort((left, right) => left.localeCompare(right))
          .map((file) => ({ phase: "test", file })),
      );
    };
    const initial = ["src/root.test.ts", "src/plugin/nested.test.ts", "src/vite-plus/task.test.ts"];
    for (const file of initial) writeTest(file);
    fs.writeFileSync(
      path.join(directory, "src/plugin/helper.ts"),
      'throw new Error("helper is not a test");',
    );
    const before = run();
    assert.equal(before.result.status, 0, `${before.result.stdout}\n${before.result.stderr}`);
    expectFiles(before.observed, initial);

    const added = ["src/new.test.ts", "src/plugin/future/deep.test.ts"];
    for (const file of added) writeTest(file);
    const after = run();
    assert.equal(after.result.status, 0, `${after.result.stdout}\n${after.result.stderr}`);
    expectFiles(after.observed, [...initial, ...added]);

    writeTest(added[1], true);
    const failedTest = run();
    assert.equal(
      failedTest.result.status,
      1,
      `${failedTest.result.stdout}\n${failedTest.result.stderr}`,
    );
    expectFiles(failedTest.observed, [...initial, ...added]);

    const failedPreparation = run(7);
    // Node's package-script launcher reports a failed script with status 1.
    assert.equal(
      failedPreparation.result.status,
      1,
      `${failedPreparation.result.stdout}\n${failedPreparation.result.stderr}`,
    );
    assert.deepEqual(failedPreparation.observed, [{ phase: "prepared" }]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
