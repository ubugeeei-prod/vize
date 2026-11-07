import { currentRuleDocumentationPacket } from "./support/current-rule-documentation.ts";
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
import { vueConfigFixtureSource, writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
await test("bare instance names use complete explicit prefix-policy successor packets", () => {
  const fixture = path.join(root, "tests/_fixtures/differential/lint/bare-reserved-8090");
  const corpusPath = path.join(fixture, "cases.json");
  const corpus = JSON.parse(fs.readFileSync(corpusPath, "utf8"));
  const successorsPath = path.join(
    root,
    "tests/_fixtures/differential/lint/reserved-prefix-8090/bare-parent-successors.json",
  );
  const successors = JSON.parse(fs.readFileSync(successorsPath, "utf8"));
  const routeSuccessorsPath = path.join(
    path.dirname(successorsPath),
    "bare-parent-route-successors.json",
  );
  const routeSuccessors = JSON.parse(fs.readFileSync(routeSuccessorsPath, "utf8"));
  const routePackets = new Map(routeSuccessors.cases.map((entry) => [entry.id, entry]));
  const defaultSuccessorsPath = path.join(
    path.dirname(successorsPath),
    "require-define-component-route-successors.json",
  );
  const defaultSuccessors = JSON.parse(fs.readFileSync(defaultSuccessorsPath, "utf8"));
  const defaultPackets = new Map(defaultSuccessors.bareCases.map((entry) => [entry.id, entry]));
  const unusedSuccessorsPath = path.join(
    path.dirname(successorsPath),
    "no-unused-setup-bindings-route-successors.json",
  );
  const unusedSuccessors = JSON.parse(fs.readFileSync(unusedSuccessorsPath, "utf8"));
  const unusedPackets = new Map(unusedSuccessors.bareCases.map((entry) => [entry.id, entry]));
  const propsSuccessorsPath = path.join(
    path.dirname(successorsPath),
    "define-props-destructuring-route-successors.json",
  );
  const propsSuccessors = JSON.parse(fs.readFileSync(propsSuccessorsPath, "utf8"));
  const propsPackets = new Map(propsSuccessors.bareCases.map((entry) => [entry.id, entry]));
  const withCurrentDefault = (entry, findings) => {
    const successor = defaultPackets.get(entry.id);
    const unused = unusedPackets.get(entry.id);
    const props = propsPackets.get(entry.id);
    for (const selected of [successor, unused, props].filter(Boolean)) {
      assert.equal(selected.filename, entry.filename);
      assert.equal(selected.sourceSha256, entry.sourceSha256);
    }
    const additions = [
      ...(successor ? [successor.diagnostic] : []),
      ...(unused ? unused.diagnostics : []),
      ...(props ? props.diagnostics : []),
    ];
    assert.ok(
      findings.every((finding) =>
        additions.every((diagnostic) => diagnostic.start <= finding.start),
      ),
    );
    return [...additions, ...findings];
  };
  assert.equal(routeSuccessors.cases.length, 16);
  assert.equal(defaultSuccessors.bareCases.length, 15);
  assert.equal(unusedSuccessors.bareCases.length, 1);
  assert.equal(propsSuccessors.bareCases.length, 0);
  assert.equal(routeSuccessors.targetSuccessors.sha256, sha256(fs.readFileSync(successorsPath)));
  assert.equal(defaultSuccessors.bases.bare.sha256, sha256(fs.readFileSync(routeSuccessorsPath)));
  assert.equal(unusedSuccessors.bases.bare.sha256, sha256(fs.readFileSync(routeSuccessorsPath)));
  assert.equal(propsSuccessors.bases.bare.sha256, sha256(fs.readFileSync(routeSuccessorsPath)));
  assert.equal(
    routeSuccessors.cases.reduce((count, entry) => count + entry.combinedCliDiagnostics.length, 0),
    12,
  );
  const projectionPath = path.join(root, "tests/_fixtures/n8n-cli-adoption.json");
  const projection = JSON.parse(fs.readFileSync(projectionPath, "utf8"));
  const original = JSON.parse(
    fs.readFileSync(path.join(fixture, "original-wave-ui-accordion.json"), "utf8"),
  );
  original.source = fs.readFileSync(path.join(fixture, original.sourceFile), "utf8");
  assert.equal(corpus.schema, "vize.lint.bare-reserved");
  assert.equal(corpus.version, 1);
  assert.equal(corpus.cases.length, 16);
  assert.equal(successors.cases.length, 16);
  assert.equal(
    successors.cases.reduce((count, entry) => count + entry.diagnostics.length, 0),
    9,
  );
  assert.equal(successors.parentCorpus.sha256, sha256(fs.readFileSync(corpusPath)));
  const successorPackets = new Map(successors.cases.map((entry) => [entry.id, entry]));
  for (const entry of corpus.cases) {
    const successor = successorPackets.get(entry.id);
    assert.equal(successor.filename, entry.filename);
    assert.equal(successor.sourceSha256, entry.sourceSha256);
    const route = routePackets.get(entry.id);
    assert.equal(route.sourceSha256, entry.sourceSha256);
    assert.deepEqual(route.targetDiagnostics, successor.diagnostics);
  }
  assert.equal(corpus.removedBareNames.length, 14);
  assert.equal(
    corpus.cases.reduce((count, entry) => count + entry.beforeDiagnostics.length, 0),
    137,
  );
  assert.equal(
    corpus.cases.reduce((count, entry) => count + entry.diagnostics.length, 0),
    17,
  );
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  assert.equal(projection.linter.rules[corpus.rule], undefined);

  const custodyPaths = [
    "tests/_fixtures/differential/lint/bare-reserved-8090/cases.json",
    "tests/_fixtures/differential/lint/reserved-prefix-8090/bare-parent-successors.json",
    "tests/_fixtures/differential/lint/reserved-prefix-8090/bare-parent-route-successors.json",
    "tests/_fixtures/differential/lint/reserved-prefix-8090/require-define-component-route-successors.json",
    "tests/_fixtures/differential/lint/reserved-prefix-8090/no-unused-setup-bindings-route-successors.json",
    "tests/_fixtures/differential/lint/reserved-prefix-8090/define-props-destructuring-route-successors.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/observe-before.mjs",
    "tests/_fixtures/differential/lint/bare-reserved-8090/primary-sources.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/original-public-before.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/official-provider.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/original-matrix-packet.json.gz",
    "tests/_fixtures/differential/lint/bare-reserved-8090/original-wave-ui-accordion.vue.fixture",
    "tests/_fixtures/differential/lint/bare-reserved-8090/original-wave-ui-accordion.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/historical-snapshot.json",
    "tests/_fixtures/differential/lint/bare-reserved-8090/historical-method-bare.snap.fixture",
    "tests/_fixtures/differential/lint/bare-reserved-8090/historical-method-bare.ts.fixture",
    "tests/_fixtures/n8n-cli-adoption.json",
    "tests/tooling/lint-default-correctness-reference.mjs",
    "tests/tooling/lint-bare-reserved.test.mjs",
    "tests/differential/build-receipt.ts",
  ];
  const custody = () =>
    custodyPaths.map((file) => ({
      path: file,
      sha256: sha256(fs.readFileSync(path.join(root, file))),
    }));
  const artifact = path.join(root, "target/differential/lint-bare-reserved.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.bare-reserved-cli",
    version: 1,
    corpus,
    successors,
    routeSuccessors,
    custody: custody(),
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bare-reserved-"));
  try {
    const build = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
    );
    evidence.build = { expected: build, receipt };
    persist();
    validateBuildReceipt(receipt, build);
    const scenarios = corpus.cases.flatMap((entry) => [
      ...[null, "recommended", "essential", "ecosystem"].map((preset) => {
        const findings = routePackets.get(entry.id).combinedCliDiagnostics;
        return {
          entry,
          preset,
          config: null,
          findings: preset === "essential" ? findings : withCurrentDefault(entry, findings),
        };
      }),
      { entry, preset: "incremental", config: null, findings: [] },
      {
        entry,
        preset: null,
        findings: withCurrentDefault(
          entry,
          routePackets
            .get(entry.id)
            .diagnostics.filter((finding) => finding.rule_name !== "vue/prop-name-casing"),
        ),
        config: JSON.stringify({ linter: { rules: { [corpus.rule]: "off" } } }) + "\n",
      },
      {
        entry,
        preset: null,
        findings: [],
        config: JSON.stringify({ linter: projection.linter }) + "\n",
      },
    ]);
    scenarios.push({
      entry: original,
      preset: null,
      findings: original.diagnostics,
      config:
        JSON.stringify({
          linter: { preset: "incremental", rules: { [corpus.rule]: "error" } },
        }) + "\n",
    });
    for (const { entry, preset, config, findings } of scenarios) {
      const file = path.join(directory, entry.filename);
      const bytes = fs.readFileSync(
        path.join(fixture, entry.sourceFile ?? entry.filename + ".fixture"),
      );
      assert.equal(bytes.toString("utf8"), entry.source);
      assert.equal(sha256(bytes), entry.sourceSha256);
      fs.writeFileSync(file, bytes);
      const configSource = config ? vueConfigFixtureSource(config) : null;
      const configFile = config ? writeVueConfigFixture(directory, config) : null;
      const argv = [
        "lint",
        ...(configFile ? ["-c", path.basename(configFile)] : ["--no-config"]),
        ...(preset ? ["--preset", preset] : []),
        "--format",
        "json",
        "--locale",
        "en",
        "--help-level",
        "none",
        entry.filename,
      ];
      const run = spawnSync(path.join(root, build.binaryPath), argv, {
        cwd: directory,
        env: process.env,
        encoding: "utf8",
        timeout: 60_000,
        maxBuffer: 4 * 1024 * 1024,
      });
      const observed = {
        id: entry.id,
        argv,
        config,
        configSource,
        status: run.status,
        signal: run.signal,
        error: run.error ? { name: run.error.name, message: run.error.message } : null,
        stdout: run.stdout ?? "",
        stderr: run.stderr ?? "",
        expected: currentRuleDocumentationPacket(withoutHelp(wholeJson(entry, findings))),
        expectedStatus: findings.some((finding) => finding.severity === "error") ? 1 : 0,
      };
      evidence.runs.push(observed);
      persist();
      observed.input = {
        text: fs.readFileSync(file, "utf8"),
        sha256: sha256(fs.readFileSync(file)),
      };
      persist();
      assert.equal(observed.error, null);
      assert.equal(observed.signal, null);
      assert.equal(observed.status, observed.expectedStatus, JSON.stringify(observed));
      assert.equal(observed.stderr, "");
      observed.actual = JSON.parse(observed.stdout);
      persist();
      assert.deepEqual(observed.actual, observed.expected, entry.id);
      assert.equal(observed.input.text, entry.source);
      assert.equal(observed.input.sha256, entry.sourceSha256);
      if (configFile) {
        assert.equal(fs.readFileSync(configFile, "utf8"), configSource);
        fs.unlinkSync(configFile);
      }
      fs.unlinkSync(file);
    }
    assert.equal(evidence.runs.filter((run) => run.id !== original.id).length, 112);
    assert.equal(evidence.runs.filter((run) => run.id === original.id).length, 1);
    assert.deepEqual(custody(), evidence.custody);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
