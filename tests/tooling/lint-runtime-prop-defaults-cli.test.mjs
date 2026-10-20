import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";
import { withoutHelp } from "./support/lint-help-policy.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/runtime-prop-defaults-8564");
const read = (name) => JSON.parse(fs.readFileSync(path.join(fixture, name), "utf8"));
await test("runtime props preserve whole scoped original packets and full minimal routes", () => {
  const corpus = read("cases.json");
  const scoped = read("scoped-route-contract.json");
  const controls = read("default-route-controls.json");
  const ownership = read("ownership-supplement.json");
  const projection = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json"), "utf8"),
  );
  assert.equal(corpus.cases.length, 18);
  assert.equal(scoped.cases.length, 18);
  assert.equal(controls.cases.length, 7);
  assert.equal(ownership.cases.length, 12);
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  assert.equal(projection.linter.rules[corpus.rule], undefined);
  const byId = new Map(corpus.cases.map((entry) => [entry.id, entry]));
  const vectors = scoped.cases.map((route) => {
    const entry = byId.get(route.id);
    assert(entry);
    assert.equal(route.sourceSha256, entry.sourceSha256);
    return { suite: "original-scoped", entry, route };
  });
  for (const entry of ownership.cases) {
    vectors.push({
      suite: "ownership-scoped",
      entry,
      route: {
        id: "scoped",
        config: scoped.cliConfig,
        argv: [
          "lint",
          entry.filename,
          "--format",
          "json",
          "--preset",
          "incremental",
          "--config",
          "vite.config.mjs",
          "--locale",
          "en",
          "--help-level",
          "none",
        ],
        afterCliPacket: wholeJson(entry),
        afterStatus: entry.diagnostics.some((finding) => finding.severity === "error") ? 1 : 0,
      },
    });
  }
  for (const control of controls.cases) {
    const entry = byId.get(control.id);
    assert(entry);
    assert.equal(entry.sourceSha256, control.sourceSha256);
    assert.equal(control.routes.length, 7);
    for (const route of control.routes) {
      const preset = ["recommended", "essential", "ecosystem", "incremental"].includes(route.id)
        ? route.id
        : null;
      const config =
        route.id === "explicit-off"
          ? JSON.stringify({ linter: { rules: { [corpus.rule]: "off" } } }) + "\n"
          : route.id === "n8n-projection"
            ? JSON.stringify({ linter: projection.linter }) + "\n"
            : null;
      vectors.push({
        suite: "minimal-full-route",
        entry,
        route: {
          ...route,
          config,
          argv: [
            "lint",
            ...(config ? ["--config", "vite.config.mjs"] : ["--no-config"]),
            ...(preset ? ["--preset", preset] : []),
            "--format",
            "json",
            "--locale",
            "en",
            "--help-level",
            "none",
            entry.filename,
          ],
          afterCliPacket: route.cliPacket,
          afterStatus: route.status,
        },
      });
    }
  }
  assert.equal(vectors.length, 79);
  const artifact = path.join(root, "target/differential/lint-runtime-prop-defaults.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.runtime-prop-defaults-source-cli",
    version: 1,
    corpus: {
      scoped: sha256(JSON.stringify(scoped)),
      controls: sha256(JSON.stringify(controls)),
      ownership: sha256(JSON.stringify(ownership)),
    },
    scope:
      "18 scoped whole sources, 12 ownership controls, 49 complete minimal routes; no whole real-project/default/public qualification",
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-runtime-props-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    for (const { suite, entry, route } of vectors) {
      const bytes = fs.readFileSync(path.join(fixture, entry.sourceFile));
      assert.equal(bytes.toString("utf8"), entry.source);
      assert.equal(sha256(bytes), entry.sourceSha256);
      const file = path.join(directory, entry.filename);
      fs.writeFileSync(file, bytes);
      if (route.config !== null) writeVueConfigFixture(directory, route.config);
      const argv = route.argv.map((arg) => (arg === "vize.config.json" ? "vite.config.mjs" : arg));
      const run = spawnSync(path.join(root, build.binaryPath), argv, {
        cwd: directory,
        env: process.env,
        encoding: "utf8",
        timeout: 60_000,
        maxBuffer: 4 * 1024 * 1024,
      });
      const observed = {
        suite,
        id: entry.id,
        route: route.id,
        source: fs.readFileSync(file, "utf8"),
        sourceSha256: sha256(fs.readFileSync(file)),
        argv,
        originalArgv: route.argv,
        config: route.config,
        expected: withoutHelp(route.afterCliPacket),
        expectedStatus: route.afterStatus,
        status: run.status,
        signal: run.signal,
        error: run.error ? { name: run.error.name, message: run.error.message } : null,
        stdout: run.stdout ?? "",
        stderr: run.stderr ?? "",
      };
      try {
        observed.actual = JSON.parse(observed.stdout);
      } catch (error) {
        observed.parseError = { name: error.name, message: error.message };
      }
      evidence.runs.push(observed);
      persist();
    }
    assert.equal(evidence.runs.length, 79);
    for (const run of evidence.runs) {
      const label = `${run.suite}/${run.id}/${run.route}`;
      assert.equal(run.signal, null, label);
      assert.equal(run.error, null, label);
      assert.equal(run.stderr, "", label);
      assert.equal(run.parseError, undefined, label);
      assert.equal(run.status, run.expectedStatus, label);
      assert.deepEqual(run.actual, run.expected, label);
    }
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
