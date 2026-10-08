import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256 } from "../differential/manifest.mjs";
import {
  multiValueReference,
  multiValueComparisons,
} from "./support/css-multi-value-reference.mjs";
import { diagnostics, validateDiagnostics } from "./support/css-rule-layout-semantics.mjs";
import { test } from "node:test";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = path.join(
  root,
  "tests/_fixtures/differential/formatter-regressions/css-continuation-indent-7915",
);
const raw = fs.readFileSync(path.join(directory, "cases.json"));
const corpus = JSON.parse(raw);

await test("public CSS continuation indentation is a configured-unit fixed point", async (t) => {
  assert.equal(corpus.issue, 7915);
  assert.equal(corpus.cases.length, 12);
  const body = fs.readFileSync(path.join(directory, "issue-7915.md"));
  assert.equal(sha256(body), corpus.originalIssueSha256);
  const original = fs.readFileSync(path.join(directory, "Button.vue.txt"));
  assert.equal(sha256(original), corpus.originalSourceSha256);
  assert.equal(original.toString(), body.toString().match(/```vue\n([\s\S]*?)```/)[1]);
  const stock = JSON.parse(fs.readFileSync(path.join(directory, "stock-prettier.json")));
  assert.equal(stock.rows.length, 5);
  for (const row of stock.rows) {
    assert.equal(row.source, original.toString());
    assert.equal(row.output, row.again, "separate complete stock fixed-point witness");
  }
  const selectorStock = JSON.parse(
    fs.readFileSync(path.join(directory, "stock-selector-control.json")),
  );
  const selector = corpus.cases.find((row) => row.id === "selector-colons-and-value-parens");
  assert.equal(selectorStock.input, selector.input);
  assert.equal(selectorStock.expected, selector.expected);
  assert.equal(selectorStock.output, selector.expected, "whole independent stock control");
  assert.equal(selectorStock.again, selectorStock.output);
  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${build.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, build);
  const artifact = path.join(root, "target/differential/css-continuation-indent/cli-report.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const report = {
    issue: 7915,
    corpusSha256: sha256(raw),
    buildReceipt: receipt,
    nativeHandled: 0,
    cliQualified: 0,
    currentReferenceQualified: 0,
    rows: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(report, null, 2) + "\n");
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-continuation-"));
  try {
    for (const fixture of corpus.cases) {
      await t.test(fixture.id, () => {
        for (const key of ["input", "expected"]) {
          const carrier = fs.readFileSync(path.join(directory, fixture[`${key}File`]));
          assert.equal(carrier.toString(), fixture[key]);
          assert.equal(sha256(carrier), fixture[`${key}Sha256`]);
        }
        const qualification = multiValueReference(
          root,
          "css-continuation-indent-7915",
          fixture,
          Buffer.from(fixture.input),
          Buffer.from(fixture.expected),
          raw,
        );
        const file = path.join(temporary, fixture.filename);
        fs.writeFileSync(file, fixture.input);
        const config = Buffer.from(JSON.stringify({ formatter: fixture.options }) + "\n");
        fs.writeFileSync(path.join(temporary, "vize.config.json"), config);
        const row = {
          id: fixture.id,
          fixture,
          currentReference: qualification.reference,
          currentExpected: qualification.expected.toString(),
          attempts: [],
          diagnostics: [],
          qualified: false,
        };
        report.rows.push(row);
        const observe = (source, pass) => {
          const complete = diagnostics(source, fixture);
          row.diagnostics.push({ pass, ...complete });
          persist();
          validateDiagnostics(complete, { ...fixture, expectedParseErrors: [] });
          if (pass > 0) {
            assert.deepEqual(complete.dom, row.diagnostics[0].dom, "whole DOM compiler output");
            assert.deepEqual(complete.ssr, row.diagnostics[0].ssr, "whole SSR compiler output");
          }
        };
        observe(fixture.input, 0);
        for (const [index, oracle] of qualification.cliExpected.entries()) {
          const args = [
            "fmt",
            oracle.mode,
            ...(fixture.options.endOfLine ? ["--config", "vize.config.json"] : ["--no-config"]),
          ];
          if (fixture.options.tabWidth) args.push("--tab-width", `${fixture.options.tabWidth}`);
          if (fixture.options.useTabs) args.push("--use-tabs");
          args.push(fixture.filename);
          const before = fs.readFileSync(file);
          const result = spawnSync(path.join(root, build.binaryPath), args, {
            cwd: temporary,
            env: { ...process.env, NO_COLOR: "1" },
            timeout: 60_000,
            maxBuffer: 8 * 1024 * 1024,
          });
          const capture = {
            args,
            status: result.status,
            signal: result.signal,
            error: result.error?.message ?? null,
            stdoutBase64: (result.stdout ?? Buffer.alloc(0)).toString("base64"),
            stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
            before: before.toString(),
            after: null,
            afterReadError: null,
          };
          row.attempts.push(capture);
          persist();
          let after;
          try {
            after = fs.readFileSync(file);
            capture.after = after.toString();
          } catch (error) {
            capture.afterReadError = { code: error.code ?? null, message: error.message };
            persist();
            throw error;
          }
          persist();
          assert.deepEqual(fs.readFileSync(path.join(temporary, "vize.config.json")), config);
          assert.equal(capture.error, null);
          assert.equal(capture.signal, null);
          assert.equal(capture.status, oracle.status);
          assert.deepEqual(Buffer.from(capture.stdoutBase64, "base64"), Buffer.from(oracle.stdout));
          assert.deepEqual(Buffer.from(capture.stderrBase64, "base64"), Buffer.from(oracle.stderr));
          if (oracle.mode === "--check") assert.deepEqual(after, before, "check never writes");
          else {
            assert.equal(
              after.toString(),
              qualification.expected.toString(),
              `whole pass ${index}`,
            );
            capture.comparisons = multiValueComparisons(
              qualification,
              Buffer.from(fixture.expected),
              after,
            );
            observe(after.toString(), index);
          }
        }
        assert.equal(row.attempts.length, 5);
        row.qualified = true;
        report.cliQualified++;
        if (qualification.reference) report.currentReferenceQualified++;
        persist();
      });
    }
    assert.equal(report.cliQualified, 12);
    assert.equal(report.currentReferenceQualified, 11);
    assert.equal(
      report.rows.reduce((sum, row) => sum + row.attempts.length, 0),
      60,
    );
  } finally {
    persist();
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
