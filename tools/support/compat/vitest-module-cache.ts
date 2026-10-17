import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const fixture = path.join(repo, "tools/support/compat/vitest/fixtures/fs-module-cache");
const args = process.argv.slice(2);
assert.equal(
  args.length,
  4,
  "usage: node vitest-module-cache.ts --candidate ENTRY --output DIRECTORY",
);
assert.equal(args[0], "--candidate");
assert.equal(args[2], "--output");
const candidate = fs.realpathSync(path.resolve(args[1]));
const output = path.resolve(args[3]);
assert.ok(
  !fs.existsSync(output),
  "use a new evidence directory; existing cache evidence is preserved",
);
fs.mkdirSync(output, { recursive: true });
const requireFixture = createRequire(path.join(fixture, "package.json"));
const vitest = path.join(path.dirname(requireFixture.resolve("vitest/package.json")), "vitest.mjs");
const baseline = requireFixture.resolve("@vizejs/vite-plugin");
const official = requireFixture.resolve("@vitejs/plugin-vue");
const versions = Object.fromEntries(
  [
    "vitest",
    "vite",
    "vue",
    "@vitejs/plugin-vue",
    "happy-dom",
    "@vue/test-utils",
    "@vizejs/vite-plugin",
  ].map((name) => [
    name,
    JSON.parse(fs.readFileSync(path.join(fixture, "node_modules", name, "package.json"), "utf8"))
      .version,
  ]),
);
assert.equal(versions.vitest, "5.0.5");
assert.equal(versions.vite, "8.3.4");
assert.equal(
  versions["@vizejs/vite-plugin"],
  "0.458.0",
  "keep the reported broken release as the baseline",
);

type CacheFile = { file: string; sha256: string; modifiedMs: number };
type Execution = { pid: number; text: string };
function cacheSnapshot(directory: string): CacheFile[] {
  if (!fs.existsSync(directory)) return [];
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) return cacheSnapshot(file);
      assert.ok(entry.isFile(), "the persistent cache must contain regular files");
      return [
        {
          file,
          sha256: crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"),
          modifiedMs: fs.statSync(file).mtimeMs,
        },
      ];
    })
    .sort((left, right) => (left.file < right.file ? -1 : left.file > right.file ? 1 : 0));
}
function executions(file: string): Execution[] {
  return fs.existsSync(file)
    ? fs
        .readFileSync(file, "utf8")
        .trim()
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line))
    : [];
}
// The existing preparation wrapper records the source build. Preserve and
// authenticate its actual addon bytes without changing the child environment;
// the published baseline still loads its own separate installed native package.
const preparationFile = path.join(repo, "npm/native/.artifacts/native/js-test-preparation.json");
if (fs.existsSync(preparationFile)) {
  const bytes = fs.readFileSync(preparationFile);
  const preparation = JSON.parse(bytes.toString("utf8"));
  const binding = fs.realpathSync(path.join(repo, "npm/native", preparation.binding));
  assert.equal(
    crypto.createHash("sha256").update(fs.readFileSync(binding)).digest("hex"),
    preparation.sha256,
  );
  assert.equal(preparation.nativeDir, fs.realpathSync(path.join(repo, "npm/native")));
  const sourceSha = process.env.SOURCE_SHA ?? process.env.GITHUB_SHA;
  if (sourceSha) assert.equal(preparation.head, sourceSha);
  const requireCandidate = createRequire(path.join(repo, "npm/builder/vite/package.json"));
  const nativeEntry = fs.realpathSync(requireCandidate.resolve("@vizejs/native"));
  assert.equal(nativeEntry, fs.realpathSync(path.join(repo, "npm/native/index.js")));
  requireCandidate("@vizejs/native");
  assert.ok(
    Object.keys(requireCandidate.cache).some((file) => file === binding),
    "the genuine source addon must actually load without a platform-package fallback",
  );
  fs.writeFileSync(path.join(output, "source-native-preparation.json"), bytes);
  fs.writeFileSync(
    path.join(output, "source-native-binding.json"),
    JSON.stringify({ nativeEntry, binding, sha256: preparation.sha256, loaded: true }, null, 2),
  );
}

const observations: object[] = [];
function campaign(name: string, entry: string, cache: boolean) {
  const directory = path.join(output, name);
  const cacheDirectory = path.join(directory, ".vitest-cache");
  const receipt = path.join(directory, "executions.jsonl");
  fs.mkdirSync(directory);
  fs.cpSync(path.join(fixture, "src"), path.join(directory, "src"), { recursive: true });
  fs.writeFileSync(path.join(directory, "package.json"), '{"private":true,"type":"module"}\n');
  fs.symlinkSync(
    path.join(fixture, "node_modules"),
    path.join(directory, "node_modules"),
    "junction",
  );
  // The same config, project root and physical cache survive every CLI process.
  fs.writeFileSync(
    path.join(directory, "vitest.config.ts"),
    [
      'import { defineConfig } from "vitest/config";',
      `import plugin from ${JSON.stringify(entry)};`,
      `export default defineConfig({ plugins: [plugin()], test: { environment: "happy-dom", fsModuleCache: ${cache}, fsModuleCachePath: ${JSON.stringify(cacheDirectory)} } });`,
    ].join("\n"),
  );
  function run(
    phase: string,
    expectedText = "Hello vize",
    outcome: "pass" | "resolution-failure" | "assertion-failure" = "pass",
  ) {
    const before = cacheSnapshot(cacheDirectory);
    const beforeExecutions = executions(receipt).length;
    const reportFile = path.join(directory, `${phase}.json`);
    const result = spawnSync(
      process.execPath,
      [vitest, "run", "--reporter=json", `--outputFile=${reportFile}`],
      {
        cwd: directory,
        env: {
          ...process.env,
          DEBUG: "vitest:cache:fs",
          VIZE_CACHE_TEST_RECEIPT: receipt,
          VIZE_CACHE_EXPECTED_TEXT: expectedText,
        },
        encoding: "utf8",
        timeout: 90_000,
        maxBuffer: 16 * 1024 * 1024,
      },
    );
    fs.writeFileSync(path.join(directory, `${phase}.stdout`), result.stdout ?? "");
    fs.writeFileSync(path.join(directory, `${phase}.stderr`), result.stderr ?? "");
    assert.ifError(result.error);
    assert.equal(result.signal, null, "the original CLI process must finish without termination");
    const report = JSON.parse(fs.readFileSync(reportFile, "utf8"));
    const actualExecutions = executions(receipt);
    const after = cacheSnapshot(cacheDirectory);
    const observation = {
      campaign: name,
      phase,
      expectedText,
      outcome,
      status: result.status,
      tests: report.numTotalTests,
      passed: report.numPassedTests,
      failed: report.numFailedTests,
      before,
      after,
      executions: actualExecutions.slice(beforeExecutions),
    };
    observations.push(observation);
    fs.writeFileSync(
      path.join(output, "observations.json"),
      JSON.stringify({ versions, candidate, baseline, observations }, null, 2),
    );
    if (outcome === "resolution-failure") {
      assert.equal(result.status, 1);
      assert.equal(
        actualExecutions.length,
        beforeExecutions,
        "the reported broken warm run executes no test",
      );
      assert.equal(report.numTotalTests, 0);
      assert.match(JSON.stringify(report), /Cannot find module.*Hello\.vue\?vue&vize/);
    } else {
      assert.equal(result.status, outcome === "pass" ? 0 : 1);
      assert.equal(report.numTotalTests, 1);
      assert.equal(report.numPassedTests, outcome === "pass" ? 1 : 0);
      assert.equal(report.numFailedTests, outcome === "pass" ? 0 : 1);
      assert.equal(
        actualExecutions.length,
        beforeExecutions + 1,
        "each fresh CLI process must execute the genuine mount assertion",
      );
      assert.ok(Number.isInteger(actualExecutions.at(-1)?.pid));
      assert.equal(
        new Set(actualExecutions.map(({ pid }) => pid)).size,
        actualExecutions.length,
        "every observation comes from a different real test process",
      );
      if (outcome === "assertion-failure") assert.match(JSON.stringify(report), /AssertionError/);
    }
    if (cache && phase !== "cold") {
      assert.ok(before.length > 0, "warm processes retain an already populated physical cache");
      assert.ok(
        before.some((old) =>
          after.some((next) => next.file === old.file && next.sha256 === old.sha256),
        ),
        "do not clear or replace the persistent cache between processes",
      );
    }
    if (cache && outcome !== "resolution-failure" && phase !== "cold") {
      assert.match(result.stderr, /\[read\].*hello\.test\.ts.*is cached/);
      if (phase !== "changed-sfc")
        assert.deepEqual(
          after,
          before,
          "warm runs must retain the complete cache bytes, file identities and timestamps",
        );
    }
    return { before, after, stderr: result.stderr, execution: actualExecutions.at(-1) };
  }
  return { run, directory, cacheDirectory, receipt };
}

const broken = campaign("published-0.458.0", baseline, true);
broken.run("cold");
broken.run("warm", "Hello vize", "resolution-failure");
broken.run("warm-again", "Hello vize", "resolution-failure");
for (const [name, entry, cache] of [
  ["official-vue", official, true],
  ["reported-no-cache-control", baseline, false],
] as const) {
  const control = campaign(name, entry, cache);
  for (const phase of ["cold", "warm", "warm-again"]) control.run(phase);
}
const fixed = campaign("candidate", candidate, true);
fixed.run("cold");
fixed.run("warm");
const warmAgain = fixed.run("warm-again");
assert.match(
  warmAgain.stderr,
  /\[read\].*Hello\.vue\?vue&vize.*is cached/,
  "the compiled SFC itself must be served from the persistent cache",
);
fixed.run("genuine-assertion-failure", "deliberately incorrect", "assertion-failure");
fixed.run("warm-after-failure");
const component = path.join(fixed.directory, "src/Hello.vue");
const original = fs.readFileSync(component, "utf8");
assert.ok(original.includes("Hello {{ name }}"));
fs.writeFileSync(component, original.replace("Hello {{ name }}", "Updated {{ name }}"));
const changed = fixed.run("changed-sfc", "Updated vize");
assert.equal(changed.execution?.text, "Updated vize");
assert.match(changed.stderr, /\[write\].*Hello\.vue\?vue&vize.*is cached/);
assert.ok(
  changed.after.some((file) => !changed.before.some((old) => old.file === file.file)),
  "changing only the SFC must create a new content cache identity",
);
fixed.run("changed-sfc-warm", "Updated vize");
fixed.run("old-output-refusal", "Hello vize", "assertion-failure");
console.log(JSON.stringify({ success: true, phases: observations.length, output, versions }));
