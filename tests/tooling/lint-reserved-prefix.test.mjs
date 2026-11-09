import { currentRuleDocumentationPacket } from "./support/current-rule-documentation.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256 } from "./lint-default-correctness-reference.mjs";
import { withoutHelp } from "./support/lint-help-policy.mjs";
import { currentWholeJson } from "./support/lint-current-correctness-corpus.mjs";
import { vueConfigFixtureSource, writeVueConfigFixture } from "./support/vue-config-fixture.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
await test("exact reserved membership and groups preserve complete CLI packets", () => {
  const fixture = path.join(root, "tests/_fixtures/differential/lint/reserved-prefix-8090");
  const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
  const originals = JSON.parse(fs.readFileSync(path.join(fixture, "originals.json"), "utf8"));
  const successorPath = "../reserved-suppression-8090/original-prefix-successors.json";
  const successors = JSON.parse(fs.readFileSync(path.join(fixture, successorPath), "utf8"));
  assert.equal(successors.cases.length, 21);
  for (const [index, entry] of originals.cases.entries()) {
    assert.equal(successors.cases[index].id, entry.id);
    assert.equal(successors.cases[index].sourceSha256, entry.sourceSha256);
  }
  const companions = JSON.parse(
    fs.readFileSync(path.join(fixture, "default-route-companions.json"), "utf8"),
  );
  const companionPackets = new Map(companions.cases.map((entry) => [entry.id, entry]));
  const defaultSuccessorsPath = path.join(
    fixture,
    "require-define-component-route-successors.json",
  );
  const defaultSuccessors = JSON.parse(fs.readFileSync(defaultSuccessorsPath, "utf8"));
  const defaultPackets = new Map(defaultSuccessors.coreCases.map((entry) => [entry.id, entry]));
  const unusedSuccessorsPath = path.join(fixture, "no-unused-setup-bindings-route-successors.json");
  const unusedSuccessors = JSON.parse(fs.readFileSync(unusedSuccessorsPath, "utf8"));
  const unusedPackets = new Map(unusedSuccessors.coreCases.map((entry) => [entry.id, entry]));
  const propsSuccessorsPath = path.join(
    fixture,
    "define-props-destructuring-route-successors.json",
  );
  const propsSuccessors = JSON.parse(fs.readFileSync(propsSuccessorsPath, "utf8"));
  const propsPackets = new Map(propsSuccessors.coreCases.map((entry) => [entry.id, entry]));
  const reservedKeysSuccessorsPath = path.join(fixture, "no-reserved-keys-route-successors.json");
  const reservedKeysSuccessors = JSON.parse(fs.readFileSync(reservedKeysSuccessorsPath, "utf8"));
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
    const companionAdditions = companionPackets
      .get(entry.id)
      .diagnostics.filter(
        (diagnostic) =>
          diagnostic.rule_name === "vue/prop-name-casing" &&
          !findings.some((finding) => JSON.stringify(finding) === JSON.stringify(diagnostic)),
      );
    assert.ok(
      findings.every((finding) =>
        additions.every((diagnostic) => diagnostic.start <= finding.start),
      ),
    );
    return [...additions, ...findings, ...companionAdditions];
  };
  assert.equal(companions.cases.length, 20);
  assert.equal(defaultSuccessors.coreCases.length, 18);
  assert.equal(unusedSuccessors.coreCases.length, 1);
  assert.equal(propsSuccessors.coreCases.length, 1);
  assert.equal(reservedKeysSuccessors.coreCases.length, 1);
  assert.equal(
    companions.targetCorpus.sha256,
    sha256(fs.readFileSync(path.join(fixture, "cases.json"))),
  );
  assert.equal(
    defaultSuccessors.bases.core.sha256,
    sha256(fs.readFileSync(path.join(fixture, "default-route-companions.json"))),
  );
  assert.equal(
    unusedSuccessors.bases.core.sha256,
    sha256(fs.readFileSync(path.join(fixture, "default-route-companions.json"))),
  );
  assert.equal(
    propsSuccessors.bases.core.sha256,
    sha256(fs.readFileSync(path.join(fixture, "default-route-companions.json"))),
  );
  assert.equal(
    reservedKeysSuccessors.bases.core.sha256,
    sha256(fs.readFileSync(path.join(fixture, "default-route-companions.json"))),
  );
  assert.equal(
    reservedKeysSuccessors.bases.bare.sha256,
    sha256(fs.readFileSync(path.join(fixture, "bare-parent-route-successors.json"))),
  );
  assert.equal(
    reservedKeysSuccessors.bases.producer.sha256,
    sha256(fs.readFileSync(path.join(fixture, reservedKeysSuccessors.bases.producer.path))),
  );
  assert.equal(
    companions.cases.reduce((count, entry) => count + entry.diagnostics.length, 0),
    63,
  );
  for (const entry of corpus.cases) {
    const companion = companionPackets.get(entry.id);
    assert.equal(companion.filename, entry.filename);
    assert.equal(companion.sourceSha256, entry.sourceSha256);
  }
  const sourceIndex = JSON.parse(
    fs.readFileSync(path.join(fixture, "original-sources.json"), "utf8"),
  );
  const projection = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json"), "utf8"),
  );
  assert.equal(corpus.schema, "vize.lint.reserved-prefix");
  assert.equal(corpus.version, 1);
  assert.equal(corpus.cases.length, 20);
  assert.equal(corpus.reservedDollarNames.length, 24);
  assert.equal(
    corpus.cases.reduce((count, entry) => count + entry.diagnostics.length, 0),
    152,
  );
  assert.equal(originals.cases.length, 21);
  assert.equal(
    originals.cases.reduce((count, entry) => count + entry.diagnostics.length, 0),
    1,
  );
  assert.equal(Object.keys(projection.linter.rules).length, 51);
  assert.equal(projection.linter.rules[corpus.rule], undefined);

  const custodyPaths = [
    "api-order-successors.json",
    "api-order-main-source-custody.json",
    "api-order-main-source-refusal.log.gz",
    "cases.json",
    "originals.json",
    successorPath,
    "original-sources.json",
    "bare-parent-successors.json",
    "default-route-companions.json",
    "require-define-component-route-successors.json",
    "no-unused-setup-bindings-route-successors.json",
    "define-props-destructuring-route-successors.json",
    "no-reserved-keys-route-successors.json",
    "primary-sources.json",
    "observe-before.mjs",
    "original-public-before.json",
    "official-provider.json",
    "provider-supplement.json",
    "initial-provider-expectations.json.gz",
    "initial-observer.mjs.fixture",
    "historical-computed-underscore.json",
    "historical-computed-underscore.snap.fixture",
    "historical-computed-underscore.ts.fixture",
    ...sourceIndex.matrices.map((entry) => entry.path),
    ...sourceIndex.sources.map((entry) => entry.sourceFile),
  ];
  const custody = () => [
    ...custodyPaths.map((file) => ({
      path: path.relative(root, path.join(fixture, file)),
      sha256: sha256(fs.readFileSync(path.join(fixture, file))),
    })),
    ...[
      "tests/tooling/lint-default-correctness-reference.mjs",
      "tests/tooling/lint-reserved-prefix.test.mjs",
      "tests/tooling/reserved-prefix-reference.test.mjs",
      "tests/differential/build-receipt.ts",
      "tests/_fixtures/n8n-cli-adoption.json",
    ].map((file) => ({ path: file, sha256: sha256(fs.readFileSync(path.join(root, file))) })),
  ];
  const artifact = path.join(root, "target/differential/lint-reserved-prefix.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.reserved-prefix-cli",
    version: 1,
    corpus,
    originals,
    companions,
    custody: custody(),
    runs: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reserved-prefix-"));
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
        const current = companionPackets.get(entry.id).combinedCliDiagnostics;
        const findings =
          preset === "essential"
            ? current.filter((diagnostic) => diagnostic.rule_name !== "vue/prop-name-casing")
            : withCurrentDefault(entry, current);
        return {
          entry,
          preset,
          config: null,
          findings,
        };
      }),
      { entry, preset: "incremental", config: null, findings: [] },
      {
        entry,
        preset: null,
        findings: withCurrentDefault(entry, companionPackets.get(entry.id).diagnostics),
        config: JSON.stringify({ linter: { rules: { [corpus.rule]: "off" } } }) + "\n",
      },
      {
        entry,
        preset: null,
        findings: [],
        config: JSON.stringify({ linter: projection.linter }) + "\n",
      },
    ]);
    scenarios.push(
      ...reservedKeysSuccessors.coreCases.map((successor) => {
        const entry = corpus.cases.find((entry) => entry.id === successor.id);
        assert.ok(entry, successor.id);
        assert.equal(successor.filename, entry.filename);
        assert.equal(successor.sourceSha256, entry.sourceSha256);
        return {
          entry,
          preset: null,
          findings: successor.diagnostics,
          config:
            JSON.stringify({
              linter: {
                preset: "incremental",
                rules: { [reservedKeysSuccessors.rule]: "error" },
              },
            }) + "\n",
        };
      }),
      ...originals.cases.map((entry, index) => ({
        entry,
        preset: null,
        findings: successors.cases[index].diagnostics,
        config:
          JSON.stringify({ linter: { preset: "incremental", rules: { [corpus.rule]: "error" } } }) +
          "\n",
      })),
    );
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
        expected: currentRuleDocumentationPacket(withoutHelp(currentWholeJson(entry, findings))),
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
    assert.equal(evidence.runs.length, 162);
    assert.deepEqual(custody(), evidence.custody);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
