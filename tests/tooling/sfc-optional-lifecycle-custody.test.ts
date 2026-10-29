import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile } from "./support/github-workflows.ts";

const driver = readRepoFile("tools", "benchmarks", "scripts", "sfc-optional-lifecycle.mjs");
const worker = readRepoFile("tools", "benchmarks", "scripts", "sfc-optional-lifecycle-worker.ts");
const ordinary = readRepoFile(
  "npm",
  "builder",
  "vite-musea",
  "src",
  "optional-prop-spacing-native.test.ts",
);
const workflow = readRepoFile(".github", "workflows", "sfc-optional-lifecycle.yml");

void test("issue 10582 keeps the reported and controlled evidence immutable", () => {
  for (const value of [
    "40176237408",
    "121117574432",
    "12592975351",
    "526751a52a8850b3e3e0a968f54df9029e336b72930bea2c7b4975305bb6ff58",
    "40177938970",
    "12592534636",
    "15ee0dd32e3ba9640ca357989a5fe214591233883125bf92ef5ca6cbb3d98320",
    "837681f00ae64f54837138fa0c5eda28fe337d4920b87de4d1fbc1d55587d1dc",
    "22a7d8dfa32482726dc2721aa3fbbb0d163f7a10c75d96a0b2e95ddd630459cf",
  ]) {
    assert.equal(driver.includes(value), true, value);
  }
  assert.match(driver, /author: "ubugeeei"/u);
  assert.match(driver, /attemptsPerImage: 1/u);
});

void test("each fresh image gets one baseline, allocator, or memcheck attempt", () => {
  assert.match(
    driver,
    /imageOrder: \["public", "source", "source-allocator", "source-memcheck"\]/u,
  );
  assert.match(driver, /glibc\.malloc\.tcache_count=0:glibc\.malloc\.perturb=165/u);
  assert.match(driver, /LD_PRELOAD: mallocDebug\.realpath/u);
  assert.match(driver, /"--tool=memcheck"/u);
  assert.match(driver, /"--error-exitcode=97"/u);
  assert.match(driver, /require\.resolve\("tsx\/esm"\)/u);
  assert.match(driver, /"--import"/u);
  assert.equal(driver.includes('require.resolve("tsx/cli")'), false);
  assert.match(driver, /fs\.constants\.COPYFILE_EXCL/u);
  assert.match(worker, /assert\.equal\(identity\.attempt, 1\)/u);
  assert.match(worker, /fileHash\(identity\.retainedBinary\)/u);
  assert.match(worker, /worker process is outside Valgrind memcheck/u);
  assert.match(worker, /malloc debug image is not mapped in the worker/u);
  assert.match(workflow, /install -y --no-install-recommends valgrind/u);
  assert.equal(workflow.includes("setup-rust-sticky-cache"), false);
});

void test("ordinary Musea evidence retains the loaded image and teardown boundary", () => {
  for (const value of [
    "loaded-native-image.node",
    "getNativeBuildToken",
    'trace("native-loaded"',
    'trace("server-close-begin"',
    'trace("server-close-end"',
    'trace("test-complete"',
    'process.once("beforeExit"',
    'process.once("exit"',
  ]) {
    assert.equal(ordinary.includes(value), true, value);
  }
  for (const value of [
    'trace("create-server-begin")',
    'trace("create-server-end")',
    'trace("close-begin")',
    'trace("close-end")',
    'trace("worker-complete")',
  ]) {
    assert.equal(worker.includes(value), true, value);
  }
});
