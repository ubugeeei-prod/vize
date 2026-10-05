import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { sha256 } from "../differential/manifest.mjs";
import { compareSfcEquivalence } from "./support/sfc-equivalence.ts";
import { observeSemantics } from "./support/formatter-template-semantics.mjs";
import { sequenceControls } from "./support/formatter-sequence-controls.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpusPath =
  "tests/_fixtures/differential/formatter-regressions/sequence-directive-indent-8023/manifest.json";
const raw = fs.readFileSync(path.join(root, corpusPath));
const corpus = JSON.parse(raw);
const require = createRequire(path.join(root, "npm/ui/package.json"));
const compiler = require("vue/compiler-sfc");
const vue = require("vue");

function diagnostics(source, fixture) {
  const parsed = compiler.parse(source, { filename: fixture.file });
  const options = {
    source: parsed.descriptor.template.content,
    filename: fixture.file,
    id: fixture.id,
    cssVars: parsed.descriptor.cssVars,
    compilerOptions: { expressionPlugins: ["typescript"] },
  };
  const dom = compiler.compileTemplate(options);
  const ssr = compiler.compileTemplate({ ...options, ssr: true });
  const serialize = (error) =>
    typeof error === "string"
      ? error
      : {
          code: error.code ?? null,
          message: error.message,
          loc: error.loc ?? null,
        };
  const observation = {
    vue: vue.version,
    parse: parsed.errors.map(serialize),
    dom: { errors: dom.errors.map(serialize), tips: dom.tips, code: dom.code },
    ssr: { errors: ssr.errors.map(serialize), tips: ssr.tips, code: ssr.code },
  };
  return observation;
}

void test("all four unchanged real project sources reach a whole formatter fixed point", async (t) => {
  const identity = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, identity);
  assert.equal(corpus.issue, 8023);
  assert.equal(corpus.cases.length, 4);
  assert.equal(sequenceControls.length, 40);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-sequence-directive-"));
  const report = {
    schema: "vize.formatter-sequence-directive-observation",
    version: 1,
    corpusPath,
    corpusSha256: sha256(raw),
    historicalMatrix: corpus.historicalMatrix,
    buildReceipt: receipt,
    nativeHandled: 0,
    expectedRealSources: 4,
    expectedRuntimeControls: 40,
    rows: [],
  };
  try {
    fs.mkdirSync(path.join(directory, ".git"));
    const cases = corpus.cases.map((fixture) => {
      const original = fs.readFileSync(path.join(root, fixture.fixture));
      assert.equal(original.length, fixture.bytes, fixture.id);
      assert.equal(sha256(original), fixture.sha256, fixture.id);
      return { ...fixture, source: original.toString() };
    });
    for (const fixture of [...cases, ...sequenceControls]) {
      const file = path.join(directory, fixture.project ?? "controls", fixture.file);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, fixture.source);
      if (fixture.eol) {
        fs.writeFileSync(
          path.join(directory, "vize.config.json"),
          `${JSON.stringify({ formatter: { endOfLine: fixture.eol } })}\n`,
        );
      }
      const row = {
        id: fixture.id,
        kind: fixture.project ? "whole-real-project" : "independent-runtime-control",
        original: fixture.source,
        inputSha256: sha256(Buffer.from(fixture.source)),
        attempts: [],
        diagnostics: [],
        equivalence: [],
      };
      report.rows.push(row);
      const observeDiagnostics = (source, pass) => {
        const observation = diagnostics(source, fixture);
        row.diagnostics.push({ pass, ...observation });
        assert.deepEqual(observation.parse, [], `${fixture.id}: whole SFC parsing`);
        assert.deepEqual(observation.dom.errors, [], `${fixture.id}: whole DOM compilation`);
        assert.deepEqual(observation.ssr.errors, [], `${fixture.id}: whole SSR compilation`);
      };
      observeDiagnostics(fixture.source, 0);
      if (fixture.states) row.originalSemantics = await observeSemantics(fixture.source, fixture);
      const invoke = (mode) => {
        const argv = [
          "fmt",
          mode,
          ...(fixture.eol ? ["--config", "vize.config.json"] : ["--no-config"]),
          path.relative(directory, file),
        ];
        const before = fs.readFileSync(file);
        const result = spawnSync(path.join(root, identity.binaryPath), argv, {
          cwd: directory,
          env: { ...process.env, NO_COLOR: "1" },
          timeout: 30000,
          maxBuffer: 1048576,
        });
        const after = fs.readFileSync(file);
        row.attempts.push({
          argv,
          status: result.status,
          signal: result.signal,
          error: result.error?.message ?? null,
          stdout: result.stdout?.toString() ?? "",
          stderr: result.stderr?.toString() ?? "",
          before: before.toString(),
          after: after.toString(),
          beforeSha256: sha256(before),
          afterSha256: sha256(after),
        });
        assert.equal(result.error, undefined, fixture.id);
        assert.equal(result.signal, null, fixture.id);
        return { result, before, after };
      };
      const initial = invoke("--check");
      assert.deepEqual(initial.after, initial.before, `${fixture.id}: check is read-only`);
      let first;
      for (let pass = 1; pass <= 3; pass++) {
        const { result, after } = invoke("--write");
        assert.equal(result.status, 0, `${fixture.id}: pass ${pass}`);
        if (pass === 1) first = after;
        else assert.deepEqual(after, first, `${fixture.id}: whole pass ${pass} equals pass 1`);
        observeDiagnostics(after.toString(), pass);
        const differences = compareSfcEquivalence(fixture.source, after.toString(), fixture.file);
        row.equivalence.push({ pass, differences });
        assert.deepEqual(differences, [], `${fixture.id}: whole template/block semantics`);
      }
      assert.equal(
        initial.result.status,
        first.equals(Buffer.from(fixture.source)) ? 0 : 1,
        fixture.id,
      );
      if (fixture.states) {
        row.formattedSemantics = await observeSemantics(first.toString(), fixture);
        assert.deepEqual(
          row.formattedSemantics.states,
          row.originalSemantics.states,
          `${fixture.id}: complete DOM/click/SSR states`,
        );
      }
      const final = invoke("--check");
      assert.equal(final.result.status, 0, `${fixture.id}: final check`);
      assert.deepEqual(final.after, final.before, `${fixture.id}: final check is read-only`);
      row.qualified = true;
    }
  } finally {
    const artifact = path.join(
      root,
      "target/differential/formatter-sequence-directive-indent-8023/cli-report.json",
    );
    fs.mkdirSync(path.dirname(artifact), { recursive: true });
    fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
    t.diagnostic(`Whole sources, three outputs, diagnostics and processes: ${artifact}`);
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
