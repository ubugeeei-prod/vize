import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const fixture = path.join(
  root,
  "tests/_fixtures/differential/lint-regressions/computed-local-reactivity-8756",
);
const read = (name) => fs.readFileSync(path.join(fixture, name), "utf8");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const original = read("ComputedLocal.vue.txt");
const controls = JSON.parse(read("controls.json"));
const manifest = JSON.parse(read("source.json"));
const rule = "type/no-reactivity-loss";

await test("whole third-party computed input, configs and warning-positive controls keep source custody", () => {
  assert.equal(manifest.issue, 8756);
  assert.deepEqual(manifest.reporter, { id: 102337893, login: "naitokosuke" });
  for (const [name, pin] of Object.entries(manifest.files)) {
    const bytes = fs.readFileSync(path.join(fixture, name));
    assert.equal(bytes.length, pin.bytes, name);
    assert.equal(digest(bytes), pin.sha256, name);
  }
  const issue = read("original-issue.md");
  assert.equal(issue.match(/```vue\n([\s\S]*?)```/)[1], original);
  const configs = [...issue.matchAll(/```json\n([\s\S]*?)```/g)].map((match) => match[1]);
  assert.deepEqual(configs, [read("vize.config.json.txt"), read("tsconfig.json.txt")]);
  assert.equal(controls.length, 35);
  assert.equal(new Set(controls.map((entry) => entry.id)).size, controls.length);
  assert.equal(manifest.historicalWarnings, 2);
  assert.equal(manifest.expectedOriginalWarnings, 0);
});

await test("source CLI reruns current ref reads and retains real snapshot/deferred/custom warnings", () => {
  // Existing tooling jobs supply this authenticated source CLI; no builder or
  // fallback to an unrelated installed version belongs to this regression.
  const build = expectedBuildIdentity(root);
  const binary = path.join(root, build.binaryPath);
  const receipt = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
  validateBuildReceipt(receipt, build);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-computed-8756-"));
  const output = path.join(root, "target/differential/lint-computed-local-reactivity-8756.json");
  const evidence = {
    schema: "vize.lint.computed-local-reactivity-8756.cli",
    version: 1,
    build,
    receipt,
    manifest,
    qualified: 0,
    runs: [],
    status: "PENDING",
  };
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const save = () => fs.writeFileSync(output, JSON.stringify(evidence, null, 2) + "\n");
  try {
    const extension = process.platform === "win32" ? ".exe" : "";
    const provider = path.join(
      root,
      "node_modules/@typescript",
      `typescript-${process.platform}-${process.arch}`,
    );
    const native = path.join(provider, "lib", `tsc${extension}`);
    const nativePackage = JSON.parse(fs.readFileSync(path.join(provider, "package.json"), "utf8"));
    evidence.corsa = { package: nativePackage, binarySha256: digest(fs.readFileSync(native)) };
    save();
    assert.equal(nativePackage.version, "7.0.2");
    const env = { ...process.env, NO_COLOR: "1", CORSA_BIN: native };
    delete env.VIZE_TEST_DISABLE_TSGO;
    const probe = spawnSync(native, ["--version"], { cwd: directory, env, timeout: 30_000 });
    evidence.corsa.probe = {
      status: probe.status,
      signal: probe.signal,
      error: probe.error?.message ?? null,
      stdoutBase64: probe.stdout?.toString("base64") ?? null,
      stderrBase64: probe.stderr?.toString("base64") ?? null,
    };
    save();
    assert.equal(probe.error, undefined);
    assert.equal(probe.signal, null);
    assert.equal(probe.status, 0);
    assert.equal(probe.stdout.toString("utf8").trim(), "Version 7.0.2");
    assert.deepEqual(probe.stderr, Buffer.alloc(0));
    const modules = path.join(root, "tests/node_modules");
    evidence.vue = {
      package: JSON.parse(fs.readFileSync(path.join(modules, "vue/package.json"), "utf8")),
      typesSha256: digest(fs.readFileSync(path.join(modules, "vue/dist/vue.d.ts"))),
    };
    save();
    fs.symlinkSync(
      modules,
      path.join(directory, "node_modules"),
      process.platform === "win32" ? "junction" : "dir",
    );
    writeVueConfigFixture(directory, read("vize.config.json.txt"));
    fs.writeFileSync(path.join(directory, "tsconfig.json"), read("tsconfig.json.txt"));
    for (const scenario of [
      { id: "original", source: original, expectedSnapshotAndSpreadWarnings: 0 },
      ...controls,
    ]) {
      const filename = "ComputedLocal.vue";
      fs.writeFileSync(path.join(directory, filename), scenario.source);
      const args = ["lint", filename, "--format", "json", "--locale", "en", "--help-level", "none"];
      const result = spawnSync(binary, args, {
        cwd: directory,
        timeout: 60_000,
        maxBuffer: 8 * 1024 * 1024,
        env,
      });
      const run = {
        args,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdoutBase64: result.stdout?.toString("base64") ?? null,
        stderrBase64: result.stderr?.toString("base64") ?? null,
      };
      // Keep the full input and both raw streams before any fallible assertion.
      evidence.runs.push({ ...scenario, bytes: Buffer.byteLength(scenario.source), run });
      save();
      assert.equal(result.error, undefined, scenario.id);
      assert.equal(result.signal, null, scenario.id);
      const expectedErrors = scenario.expectedCliErrors ?? [];
      assert.equal(result.status, expectedErrors.length === 0 ? 0 : 1, JSON.stringify(run));
      assert.deepEqual(result.stderr, Buffer.alloc(0), scenario.id);
      const packet = JSON.parse(result.stdout.toString("utf8"));
      assert.equal(packet.length, 1, scenario.id);
      assert.equal(packet[0].file, filename, scenario.id);
      assert.deepEqual(
        packet[0].messages.filter((message) => message.severity === 2),
        expectedErrors,
        scenario.id,
      );
      assert.equal(packet[0].errorCount, expectedErrors.length, scenario.id);
      const findings = packet[0].messages.filter((message) => message.ruleId === rule);
      const snapshots = findings.filter((message) =>
        /Assigning 'held\.value'|Spreading 'h\.shape'/.test(message.message),
      );
      assert.equal(snapshots.length, scenario.expectedSnapshotAndSpreadWarnings, scenario.id);
      // Rerun-scope aliases stay fresh; only setup-scope snapshots keep warnings.
      assert.equal(findings.length, scenario.expectedSnapshotAndSpreadWarnings, scenario.id);
      assert.equal(fs.readFileSync(path.join(directory, filename), "utf8"), scenario.source);
      evidence.qualified++;
      save();
    }
    assert.equal(evidence.qualified, 36);
    evidence.status = "PASS";
  } catch (error) {
    evidence.status = "FAIL";
    evidence.error = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    save();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
