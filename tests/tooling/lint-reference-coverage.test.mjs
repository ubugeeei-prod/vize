import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createRequire } from "node:module";
import { gunzipSync } from "node:zlib";
import { test } from "node:test";
import {
  baselineConfig,
  resolveBaselineRuntime,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const fixtures = path.join(root, "tests/_fixtures/differential/linter/reference-coverage-8849");
const contract = JSON.parse(fs.readFileSync(path.join(fixtures, "contract.json")));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const requireBench = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));

await test("reviewed coverage preserves complete frozen packets and exact source identities", () => {
  assert.equal(contract.cases.length, 2);
  for (const entry of contract.cases) {
    const source = fs.readFileSync(path.join(fixtures, entry.source));
    const bytes = gunzipSync(fs.readFileSync(path.join(fixtures, entry.packet)));
    assert.equal(hash(source), entry.sourceSha256);
    assert.equal(hash(bytes), entry.packetSha256);
    const packet = JSON.parse(bytes);
    assert.equal(packet.project, entry.project);
    assert.equal(packet.revision, entry.revision);
    assert.equal(packet.preset, entry.preset);
    assert.deepEqual(packet.divergence.falsePositives, [entry.finding]);
    assert.equal(packet.budget.maxFalsePositiveCount, 0);
    assert.equal(packet.budget.maxFalseNegativeCount, 0);
    const nativeBytes = gunzipSync(fs.readFileSync(path.join(fixtures, entry.nativePacket)));
    assert.equal(hash(nativeBytes), entry.nativePacketSha256);
    const native = JSON.parse(nativeBytes);
    assert.deepEqual(JSON.parse(native.stdout), native.parsed);
    const file = native.parsed.find((file) => file.file === entry.finding.file);
    const finding = file.messages.filter((finding) => finding.ruleId === entry.finding.ruleId);
    assert.equal(finding.length, 1);
    assert.equal(finding[0].message, entry.finding.message);
    assert.equal(finding[0].severity, entry.finding.severity === "warning" ? 1 : 2);
    assert.deepEqual(entry.options, []);
    assert.deepEqual(entry.upstreamOptions, []);
  }
});

await test("pinned reference AST and visitors miss only the two proven coverage cases", async () => {
  const runtime = resolveBaselineRuntime();
  for (const [name, version] of Object.entries(contract.cases[0].runtimeVersions)) {
    assert.equal(requireBench(`${name}/package.json`).version, version, name);
  }
  const eslint = new runtime.ESLint({
    cwd: root,
    overrideConfigFile: true,
    overrideConfig: baselineConfig(runtime, {
      "vue/v-slot-style": "warn",
      "vue/no-mutating-props": "error",
    }),
  });
  const lint = async (source) =>
    (await eslint.lintText(source, { filePath: "Reference.vue" }))[0].messages;
  const slot = fs.readFileSync(path.join(fixtures, contract.cases[0].source), "utf8");
  const card = fs.readFileSync(path.join(fixtures, contract.cases[1].source), "utf8");
  assert.deepEqual(await lint(slot), []);
  assert.deepEqual(await lint(card), []);
  const classAst = runtime.vueParser.parseForESLint(card, {
    parser: runtime.scriptParser,
    sourceType: "module",
    ecmaVersion: "latest",
  }).ast;
  const exported = classAst.body.find(
    (node) => node.type === "ExportDefaultDeclaration",
  ).declaration;
  assert.equal(exported.type, "ClassDeclaration");
  const prop = exported.body.body.find((node) => node.key?.name === "notebook");
  assert.equal(prop.decorators[0].expression.callee.name, "Prop");
  const bare = '<template><RouterView #="{ Component }" /></template>';
  const attribute =
    runtime.vueParser.parseForESLint(bare).ast.templateBody.children[0].startTag.attributes[0];
  assert.equal(attribute.directive, false);
  assert.equal(attribute.key.type, "VIdentifier");
  const evidence = JSON.parse(fs.readFileSync(path.join(fixtures, "oracle-source-evidence.json")));
  const compiler = requireBench("@vue/compiler-sfc");
  assert.equal(compiler.version, evidence.compilerVersion);
  for (const source of evidence.sources) {
    const packageRoot = path.dirname(requireBench.resolve(`${source.package}/package.json`));
    assert.equal(hash(fs.readFileSync(path.join(packageRoot, source.path))), source.sha256);
  }
  const outputs = ["#", "#default", "v-slot"].map((name) => {
    const result = compiler.compileTemplate({
      source: `<RouterView ${name}="{ Component }"><component :is="Component" /></RouterView>`,
      filename: "Reference.vue",
      id: "reference-coverage",
    });
    assert.deepEqual(result.errors, []);
    return result.code;
  });
  assert.equal(outputs[0], outputs[1]);
  assert.equal(outputs[0], outputs[2]);
  // Positive controls: both visitors run when their parser/declaration surface is supported.
  const style = await lint(bare.replace("#=", "#default="));
  assert.equal(style.length, 1);
  assert.equal(style[0].ruleId, "vue/v-slot-style");
  assert.equal(style[0].severity, 1);
  const mutation = await lint(
    '<script>export default { props: ["notebook"] }</script><template><draggable v-model="notebook.notes" /></template>',
  );
  assert.equal(mutation.length, 1);
  assert.equal(mutation[0].ruleId, "vue/no-mutating-props");
  assert.equal(mutation[0].severity, 2);
  // Neighbor negatives retain ordinary valid declarations and read-only prop access.
  assert.deepEqual(await lint(bare.replace("#=", "v-slot=")), []);
  assert.deepEqual(
    await lint(card.replace('v-model="notebook.notes"', ':model-value="notebook.notes"')),
    [],
  );
  assert.deepEqual(
    await lint(
      '<script>export default { props: ["notebook"] }</script><template><draggable :model-value="notebook.notes" /></template>',
    ),
    [],
  );
});

await test("actual report retains complete raw packets and scores a neighboring changed source", () => {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reference-coverage-"));
  try {
    for (const entry of contract.cases) {
      const projectDir = path.join(cwd, entry.project);
      const sourcePath = path.join(projectDir, entry.finding.file);
      fs.mkdirSync(path.dirname(sourcePath), { recursive: true });
      const source = fs.readFileSync(path.join(fixtures, entry.source), "utf8");
      fs.writeFileSync(sourcePath, source);
      const fake = path.join(projectDir, "fake-vize.mjs");
      fs.writeFileSync(
        fake,
        `#!/usr/bin/env node\nprocess.stdout.write(${JSON.stringify(JSON.stringify([{ file: entry.finding.file, messages: [entry.finding] }]))});\n`,
      );
      fs.chmodSync(fake, 0o755);
      const registry = path.join(projectDir, "registry.json");
      fs.writeFileSync(
        registry,
        JSON.stringify({
          projects: [
            {
              id: entry.project,
              revision: entry.revision,
              fixturePath: projectDir,
              vueGlobs: [entry.finding.file],
              coverage: ["linter"],
            },
          ],
        }),
      );
      const run = (messages = [entry.finding], budgetMode = "record-only", expectedStatus = 0) => {
        fs.writeFileSync(
          fake,
          `#!/usr/bin/env node\nprocess.stdout.write(${JSON.stringify(JSON.stringify([{ file: entry.finding.file, messages }]))});\n`,
        );
        const result = spawnSync(
          process.execPath,
          [
            "tools/commands/fixtures/lint-divergence-report.ts",
            "--registry",
            registry,
            "--output-dir",
            path.join(projectDir, "report"),
            "--vize-bin",
            fake,
            "--preset",
            "ecosystem",
            "--budget-mode",
            budgetMode,
          ],
          { cwd: root, encoding: "utf8", timeout: 120000 },
        );
        assert.equal(result.status, expectedStatus, `${result.stdout}\n${result.stderr}`);
        return JSON.parse(
          fs.readFileSync(path.join(projectDir, "report", `${entry.project}-lint-divergence.json`)),
        );
      };
      const packet = run();
      assert.deepEqual(packet.raw.patinaFindings, [entry.finding]);
      assert.deepEqual(packet.raw.patinaResults, [
        { file: entry.finding.file, messages: [entry.finding] },
      ]);
      assert.equal(packet.raw.eslintResults.length, 1);
      assert.deepEqual(packet.baseline.runtimeVersions, entry.runtimeVersions);
      const covered = packet.divergence.patinaOutsideBaselineSurface;
      assert.equal(covered.length, 1);
      const { referenceCoverage, ...retained } = covered[0];
      assert.deepEqual(retained, entry.finding);
      assert.equal(referenceCoverage.category, "reference-coverage-gap");
      assert.equal(referenceCoverage.sourceSha256, entry.sourceSha256);
      assert.deepEqual(packet.divergence.falsePositives, []);
      assert.equal(packet.budget.maxFalsePositiveCount, 0);
      assert.equal(packet.budget.maxFalseNegativeCount, 0);
      assert.equal(packet.divergence.summary.referenceCoverageIntegrityErrorCount, 0);
      assert.equal(packet.divergence.referenceCoverageExpectations.length, 1);
      assert.equal(packet.divergence.referenceCoverageExpectations[0].observedCount, 1);
      const missing = run([], "enforce", 1);
      assert.deepEqual(missing.raw.patinaFindings, []);
      assert.equal(missing.divergence.referenceCoverageExpectations[0].observedCount, 0);
      assert.equal(missing.divergence.summary.referenceCoverageIntegrityErrorCount, 1);
      assert.equal(missing.budget.verdict, "unusable");
      assert.equal(missing.budget.passed, false);
      assert.match(missing.budget.unusableReason, /expected exactly one native finding/);
      const duplicated = run([entry.finding, entry.finding]);
      assert.equal(duplicated.raw.patinaFindings.length, 2);
      assert.equal(duplicated.divergence.patinaOutsideBaselineSurface.length, 1);
      assert.deepEqual(duplicated.divergence.falsePositives, [entry.finding]);
      assert.equal(duplicated.divergence.referenceCoverageExpectations[0].observedCount, 2);
      assert.equal(duplicated.budget.verdict, "unusable");
      // A file may remain on disk while the actual measured corpus excludes it.
      // The complete native answer must then stay scored, with no integrity exemption.
      const registered = fs.readFileSync(registry, "utf8");
      const foreign = JSON.parse(registered);
      foreign.projects[0].vueGlobs = ["Foreign.vue"];
      fs.writeFileSync(path.join(projectDir, "Foreign.vue"), "<template><div /></template>\n");
      fs.writeFileSync(registry, JSON.stringify(foreign));
      const unmeasured = run([entry.finding], "enforce", 1);
      assert.deepEqual(unmeasured.raw.patinaFindings, [entry.finding]);
      assert.deepEqual(unmeasured.divergence.patinaOutsideBaselineSurface, []);
      assert.deepEqual(unmeasured.divergence.referenceCoverageExpectations, []);
      assert.deepEqual(unmeasured.divergence.falsePositives, [entry.finding]);
      assert.equal(unmeasured.budget.maxFalsePositiveCount, 0);
      assert.equal(unmeasured.budget.maxFalseNegativeCount, 0);
      assert.equal(unmeasured.budget.verdict, "breached");
      assert.equal(unmeasured.budget.passed, false);
      fs.writeFileSync(registry, registered);
      fs.writeFileSync(sourcePath, source + "\n");
      const changed = run();
      assert.deepEqual(changed.divergence.patinaOutsideBaselineSurface, []);
      assert.deepEqual(changed.divergence.referenceCoverageExpectations, []);
      assert.equal(changed.divergence.summary.referenceCoverageIntegrityErrorCount, 0);
      assert.deepEqual(changed.divergence.falsePositives, [entry.finding]);
      assert.deepEqual(changed.raw.patinaFindings, packet.raw.patinaFindings);
      const changedMissing = run([]);
      assert.deepEqual(changedMissing.divergence.referenceCoverageExpectations, []);
      assert.equal(changedMissing.divergence.summary.referenceCoverageIntegrityErrorCount, 0);
      const withoutSource = (entries) =>
        entries.map(({ source: _source, ...retained }) => retained);
      assert.deepEqual(
        withoutSource(changed.raw.eslintResults),
        withoutSource(packet.raw.eslintResults),
      );
      for (let index = 0; index < packet.raw.eslintResults.length; index++) {
        const previous = packet.raw.eslintResults[index].source;
        if (previous != null)
          assert.equal(changed.raw.eslintResults[index].source, previous + "\n");
      }
    }
  } finally {
    fs.rmSync(cwd, { recursive: true, force: true });
  }
});
