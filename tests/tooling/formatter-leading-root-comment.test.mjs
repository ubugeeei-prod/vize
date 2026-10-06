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

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpusPath =
  "tests/_fixtures/differential/formatter-regressions/leading-root-comment-8117/cases.json";
const raw = fs.readFileSync(path.join(root, corpusPath));
const corpus = JSON.parse(raw);
const require = createRequire(path.join(root, "npm/ui/package.json"));
const compiler = require("vue/compiler-sfc");
const vue = require("vue");

function diagnostics(source, fixture) {
  const parsed = compiler.parse(source, { filename: fixture.file });
  const serialize = (error) =>
    typeof error === "string"
      ? error
      : { code: error.code ?? null, message: error.message, loc: error.loc ?? null };
  const observation = {
    vue: vue.version,
    parse: parsed.errors.map(serialize),
    dom: null,
    ssr: null,
  };
  if (parsed.descriptor.template) {
    const options = {
      source: parsed.descriptor.template.content,
      filename: fixture.file,
      id: fixture.id,
      cssVars: parsed.descriptor.cssVars,
      compilerOptions: { expressionPlugins: ["typescript"] },
    };
    for (const target of ["dom", "ssr"]) {
      const result = compiler.compileTemplate({ ...options, ssr: target === "ssr" });
      observation[target] = {
        errors: result.errors.map(serialize),
        tips: result.tips,
        code: result.code,
      };
    }
  }
  return observation;
}

void test("whole Habitica input and first-root attachment controls reach strict fixed points", async (t) => {
  const identity = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, identity);
  assert.equal(corpus.issue, 8117);
  assert.equal(corpus.cases.length, 12);
  const original = fs.readFileSync(path.join(root, corpus.publicOriginal.fixture));
  assert.equal(original.length, corpus.publicOriginal.bytes);
  assert.equal(sha256(original), corpus.publicOriginal.sha256);
  const license = fs.readFileSync(path.join(root, corpus.publicOriginal.license.path));
  assert.equal(license.length, corpus.publicOriginal.license.bytes);
  assert.equal(sha256(license), corpus.publicOriginal.license.sha256);
  const cases = [{ ...corpus.publicOriginal, source: original.toString() }, ...corpus.cases];
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-leading-root-comment-"));
  const report = {
    schema: "vize.formatter-leading-root-comment-observation",
    version: 1,
    corpusPath,
    corpusSha256: sha256(raw),
    historicalMatrix: corpus.historicalMatrix,
    buildReceipt: receipt,
    nativeHandled: 0,
    qualified: 0,
    rows: [],
  };
  try {
    fs.mkdirSync(path.join(directory, ".git"));
    for (const fixture of cases) {
      await t.test(fixture.id, async () => {
        const file = path.join(directory, fixture.file);
        fs.writeFileSync(file, fixture.source);
        if (fixture.eol)
          fs.writeFileSync(
            path.join(directory, "vize.config.json"),
            `${JSON.stringify({ formatter: { endOfLine: fixture.eol } })}\n`,
          );
        const row = { id: fixture.id, original: fixture.source, attempts: [], diagnostics: [] };
        report.rows.push(row);
        const baseline = diagnostics(fixture.source, fixture);
        row.diagnostics.push({ pass: 0, ...baseline });
        if (fixture.states) row.originalSemantics = await observeSemantics(fixture.source, fixture);
        const invoke = (mode) => {
          const argv = [
            "fmt",
            mode,
            ...(fixture.eol ? ["--config", "vize.config.json"] : ["--no-config"]),
            fixture.file,
          ];
          const before = fs.readFileSync(file);
          const result = spawnSync(path.join(root, identity.binaryPath), argv, {
            cwd: directory,
            env: { ...process.env, NO_COLOR: "1" },
            timeout: 30000,
            maxBuffer: 1048576,
          });
          const attempt = {
            argv,
            status: result.status,
            signal: result.signal,
            error: result.error?.message ?? null,
            stdout: result.stdout?.toString() ?? "",
            stderr: result.stderr?.toString() ?? "",
            before: before.toString(),
            after: null,
            afterReadError: null,
          };
          row.attempts.push(attempt);
          let after;
          try {
            after = fs.readFileSync(file);
            attempt.after = after.toString();
          } catch (error) {
            attempt.afterReadError = error.message;
            throw error;
          }
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
          assert.deepEqual(after, first, `${fixture.id}: whole fixedpoint pass ${pass}`);
          if (fixture.expected !== undefined)
            assert.deepEqual(after, Buffer.from(fixture.expected), `${fixture.id}: full reference`);
          if (fixture.expectedLeadingContent)
            assert(after.toString().startsWith(fixture.expectedLeadingContent));
          const observation = diagnostics(after.toString(), fixture);
          row.diagnostics.push({ pass, ...observation });
          if (baseline.dom) {
            assert.deepEqual(baseline.parse, [], `${fixture.id}: original parse diagnostics`);
            assert.deepEqual(baseline.dom.errors, [], `${fixture.id}: original DOM diagnostics`);
            assert.deepEqual(baseline.ssr.errors, [], `${fixture.id}: original SSR diagnostics`);
            assert.deepEqual(observation.parse, [], `${fixture.id}: whole parse diagnostics`);
            assert.deepEqual(observation.dom.errors, [], `${fixture.id}: whole DOM diagnostics`);
            assert.deepEqual(observation.ssr.errors, [], `${fixture.id}: whole SSR diagnostics`);
          } else
            assert.deepEqual(observation, baseline, `${fixture.id}: full unchanged diagnostics`);
          const differences = compareSfcEquivalence(fixture.source, after.toString(), fixture.file);
          assert.deepEqual(differences, [], `${fixture.id}: complete template/block semantics`);
        }
        assert.equal(initial.result.status, first.equals(Buffer.from(fixture.source)) ? 0 : 1);
        if (fixture.states) {
          row.formattedSemantics = await observeSemantics(first.toString(), fixture);
          assert.deepEqual(row.formattedSemantics.states, row.originalSemantics.states);
        }
        const final = invoke("--check");
        assert.equal(final.result.status, 0, `${fixture.id}: final check`);
        assert.deepEqual(final.after, final.before, `${fixture.id}: final check is read-only`);
        row.qualified = true;
        report.qualified++;
      });
    }
    assert.equal(report.qualified, 13);
  } finally {
    const artifact = path.join(
      root,
      "target/differential/formatter-leading-root-comment-8117/cli-report.json",
    );
    fs.mkdirSync(path.dirname(artifact), { recursive: true });
    fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
    t.diagnostic(`Whole original/three outputs/diagnostics/processes: ${artifact}`);
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
