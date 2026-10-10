// Installed replay of the preserved bounded configuration/entries contract; not all51 parity.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { installedCliRuntime } from "./n8n-installed-cli.ts";
import type {
  Corpus,
  CliPacket,
  AuthoredFixture,
  Projection,
  AuthoredCapture,
} from "./n8n-installed-contract-types.ts";
import { verifyCorpus } from "../../../npm/oxlint/scripts/n8n-replay-inputs.mjs";
import {
  root,
  projection as frozenProjection,
  validateProjection,
  expectedScopedPacket,
} from "./n8n-cli-config-inputs.mjs";
import {
  captureAuthoredOracle as legacyCaptureAuthoredOracle,
  assertAuthoredOracle,
  loadAuthoredCases,
} from "./n8n-cli-config-oracle.mjs";
import {
  copyOriginals,
  assertUnchanged,
  writeConfig,
  writeJson,
  sha256,
} from "./n8n-cli-config-workspace.mjs";

assert.ok(process.argv[2] && process.argv[3], "new output and reviewed installed plan required");
const output = path.resolve(process.argv[2]);
const projection = frozenProjection as Projection;
// JS inference misses provider fields assigned after capture construction. The original whole oracle still validates the result.
const captureAuthoredOracle = legacyCaptureAuthoredOracle as unknown as AuthoredCapture;
validateProjection();
const producer = installedCliRuntime(output, process.argv[3]);
const { identity, binary, installConfigPackages, runCli } = producer;
const corpus = verifyCorpus() as Corpus;
assert.equal(corpus.revision, projection.fixtureRevision);
assert.deepEqual(corpus.packages, projection.packageVueFileCounts);
writeJson(path.join(output, "original-inputs.json"), corpus);
const workspace = producer.workspace("originals");
const packageFiles = installConfigPackages(workspace);
copyOriginals(workspace, corpus);
writeJson(path.join(output, "public-config-package.json"), packageFiles);

const summary = {
  schema: "vize.n8n.cli-config-replay",
  version: 1,
  source: identity,
  projectionSha256: sha256(
    fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json")),
  ),
  originalFiles: corpus.files.length,
  scriptlessFiles: corpus.files.filter(({ scriptless }) => scriptless).length,
  packages: {} as Record<
    string,
    { files: number; baselineErrors: number; scopedErrors: number; scopedWarnings: number }
  >,
  scopeControls: [] as Array<{
    packageRoot: string;
    file: string;
    rule: string;
    sourceSha256: string;
  }>,
  authored: [] as Array<{ configuration: string; caseId: string; packets: CliPacket[] }>,
  qualification:
    "configuration/entries and owned witnesses; not all51 semantic or full monorepo parity",
};

for (const [packageRoot, count] of Object.entries(corpus.packages)) {
  const files = corpus.files
    .filter(({ file }) => file.startsWith(packageRoot + "/"))
    .map(({ file }) => file.slice(packageRoot.length + 1));
  assert.equal(files.length, count);
  const packetRoot = path.join(output, "packets", packageRoot);
  const baselineConfig = writeConfig(workspace, packageRoot, "baseline");
  const scopedConfig = writeConfig(workspace, packageRoot, "scoped");
  const invoke = (config: { path: string }, name: string, outsideCwd = false) =>
    runCli({
      binary,
      workspace,
      packageRoot,
      files,
      config,
      outsideCwd,
      receiptPath: path.join(packetRoot, name + ".json"),
    });
  const baseline = invoke(baselineConfig, "baseline");
  const scoped = invoke(scopedConfig, "scoped");
  const repeated = invoke(scopedConfig, "scoped-repeat");
  assert.deepEqual(
    scoped,
    baseline.map((packet) => expectedScopedPacket(packet, packageRoot)),
    packageRoot + ": only literal rule/severity scopes may change the whole report",
  );
  assert.deepEqual(repeated, scoped, packageRoot + ": deterministic complete repeat");
  const outside = invoke(scopedConfig, "scoped-outside-cwd", true);
  // Named recording changes file only after retaining the complete raw report.
  const recordedOutside = outside.map((packet) => ({
    ...packet,
    file: packet.file.startsWith(packageRoot + "/")
      ? packet.file.slice(packageRoot.length + 1)
      : packet.file,
  }));
  assert.deepEqual(
    recordedOutside,
    scoped,
    packageRoot + ": entries are config-directory relative",
  );
  summary.packages[packageRoot] = {
    files: count,
    baselineErrors: baseline.reduce((n, packet) => n + packet.errorCount, 0),
    scopedErrors: scoped.reduce((n, packet) => n + packet.errorCount, 0),
    scopedWarnings: scoped.reduce((n, packet) => n + packet.warningCount, 0),
  };
  assertUnchanged(workspace, [...corpus.files, ...corpus.licenses, ...packageFiles]);
}

// Prove all six literal exemptions with positive findings and adjacent clean
// scope controls. These owned sources live separately from unchanged originals.
const controls = producer.workspace("scope-controls");
installConfigPackages(controls);
for (const [packageRoot, scope] of Object.entries(projection.scopes)) {
  const files = [];
  for (const entry of scope.entries) {
    const disabled = Object.keys(entry.linter.rules);
    assert.equal(disabled.length, 1);
    const rule = disabled[0];
    const source =
      rule === "vue/no-multiple-template-root"
        ? '<script setup lang="ts">\nimport MyWidget from "./MyWidget.vue";\n</script>\n<template><MyWidget fooBar="value" v-html="\'unsafe\'" /><p /></template>\n'
        : '<template><ul><li v-for="item in [1]" v-html="String(item)" /></ul></template>\n';
    for (const file of entry.files) {
      for (const destination of [file, file.replace(/\.vue$/u, ".neighbor.vue")]) {
        const absolute = path.join(controls, packageRoot, destination);
        fs.mkdirSync(path.dirname(absolute), { recursive: true });
        fs.writeFileSync(absolute, source);
        files.push(destination);
      }
      summary.scopeControls.push({ packageRoot, file, rule, sourceSha256: sha256(source) });
    }
  }
  const baselineConfig = writeConfig(controls, packageRoot, "baseline");
  const scopedConfig = writeConfig(controls, packageRoot, "scoped");
  const baseline = runCli({
    binary,
    workspace: controls,
    packageRoot,
    files,
    config: baselineConfig,
    receiptPath: path.join(output, "scope-control-packets", packageRoot, "baseline.json"),
  });
  const scoped = runCli({
    binary,
    workspace: controls,
    packageRoot,
    files,
    config: scopedConfig,
    receiptPath: path.join(output, "scope-control-packets", packageRoot, "scoped.json"),
  });
  assert.deepEqual(
    scoped,
    baseline.map((packet) => expectedScopedPacket(packet, packageRoot)),
  );
  for (const { file, rule } of summary.scopeControls.filter(
    (control) => control.packageRoot === packageRoot,
  )) {
    for (const name of [file, file.replace(/\.vue$/u, ".neighbor.vue")]) {
      const packet = baseline.find((packet) => packet.file === name);
      assert.ok(
        packet!.messages.some(({ ruleId }) => ruleId === rule),
        name + ": positive rule witness",
      );
      assert.ok(
        packet!.messages.some(({ ruleId }) => ruleId === "vue/no-v-html"),
        name + ": unrelated rule retained",
      );
    }
    assert.ok(
      scoped
        .find((packet) => packet.file === file.replace(/\.vue$/u, ".neighbor.vue"))!
        .messages.some(({ ruleId }) => ruleId === rule),
      file + ": adjacent filename is not exempt",
    );
  }
}

const oracle = await captureAuthoredOracle({
  onRecorded(packet) {
    writeJson(
      path.join(
        output,
        "authored-oracle",
        (packet.phase ?? packet.configuration + "--" + packet.caseId) + ".json",
      ),
      packet,
    );
  },
});
writeJson(path.join(output, "authored-oracle/complete.json"), oracle);
assertAuthoredOracle(oracle);
const authored = producer.workspace("authored-cli");
installConfigPackages(authored);
const fixture = loadAuthoredCases() as AuthoredFixture;
for (const capture of oracle.recorded.captures) {
  const sourceCase = fixture.cases.find(({ id }) => id === capture.caseId)!;
  const configuration = oracle.recorded.configurations.find(
    ({ id }) => id === capture.configuration,
  )!;
  const file = capture.caseId + ".vue";
  fs.writeFileSync(path.join(authored, file), sourceCase.source);
  const config = writeConfig(authored, "", capture.configuration, configuration.ruleOptions);
  const packets = runCli({
    binary,
    workspace: authored,
    packageRoot: "",
    files: [file],
    config,
    receiptPath: path.join(
      output,
      "authored-cli-packets",
      capture.configuration + "--" + capture.caseId + ".json",
    ),
  });
  summary.authored.push({ configuration: capture.configuration, caseId: capture.caseId, packets });
}
// Retain every authored response before the whole oracle laws can reject one.
writeJson(path.join(output, "authored-cli-packets/complete.json"), summary.authored);
for (const capture of oracle.recorded.captures) {
  const { packets } = summary.authored.find(
    ({ configuration, caseId }) =>
      configuration === capture.configuration && caseId === capture.caseId,
  )!;
  const actualRuleIds = packets.flatMap(({ messages }) => messages.map(({ ruleId }) => ruleId));
  const inverse = Object.fromEntries(
    Object.entries(projection.oracleRules).map(([rule, oracle]) => [oracle, rule]),
  );
  const expectedRuleIds = capture.packets.flatMap(({ messages }) =>
    messages.map(({ ruleId }) => inverse[ruleId]),
  );
  assert.deepEqual(actualRuleIds, expectedRuleIds, capture.configuration + "/" + capture.caseId);
  const expectation = fixture.cliExpectations.find(
    ({ configuration, caseId }) =>
      configuration === capture.configuration && caseId === capture.caseId,
  );
  assert.ok(expectation, "every owned witness requires a whole CLI envelope law");
  assert.deepEqual(packets, expectation.packets, capture.configuration + "/" + capture.caseId);
}
assertUnchanged(workspace, [...corpus.files, ...corpus.licenses, ...packageFiles]);
assert.deepEqual(verifyCorpus(), corpus, "licensed original fixture remains byte-identical");
producer.recheck();
writeJson(path.join(output, "summary.json"), summary);
console.log(
  "Installed CLI configuration/entries replay passed: " +
    JSON.stringify({
      files: summary.originalFiles,
      scriptless: summary.scriptlessFiles,
      packages: Object.keys(summary.packages).length,
      scopedPaths: summary.scopeControls.length,
      authored: summary.authored.length,
    }),
);
