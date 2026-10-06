import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { sha256 } from "../differential/manifest.mjs";
import { diagnostics, validateDiagnostics } from "./support/css-rule-layout-semantics.mjs";
import { test } from "node:test";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = path.join(
  root,
  "tests/_fixtures/differential/formatter-regressions/hugged-interpolation-indent-7969",
);
const raw = fs.readFileSync(path.join(directory, "cases.json"));
const corpus = JSON.parse(raw);

await test("sole hugged interpolations share their parent layout on all three passes", async (t) => {
  assert.equal(corpus.issue, 7969);
  assert.equal(corpus.cases.length, 13);
  const body = fs.readFileSync(path.join(directory, "issue-7969.md"));
  assert.equal(sha256(body), corpus.originalIssueSha256);
  const original = fs.readFileSync(path.join(directory, "Names.vue.txt"));
  assert.equal(sha256(original), corpus.originalSourceSha256);
  assert.equal(original.toString(), body.toString().match(/```vue\n([\s\S]*?)```/)[1]);
  const originalConfig = fs.readFileSync(path.join(directory, "vize.config.json.txt"));
  assert.equal(sha256(originalConfig), corpus.originalConfigSha256);
  assert.equal(originalConfig.toString(), body.toString().match(/```json\n([\s\S]*?)```/)[1]);
  const stock = JSON.parse(fs.readFileSync(path.join(directory, "stock-prettier.json")));
  assert.equal(stock.version, "3.8.3");
  assert.deepEqual(
    stock.rows.map((row) => row.id),
    ["original", "original-crlf", "first-pass-wrapped"],
  );
  for (const row of stock.rows) {
    const fixture = corpus.cases.find((item) => item.id === row.id);
    assert.equal(row.input, fixture.input);
    assert.equal(row.output, fixture.expected, "separate whole stock reference");
    assert.equal(row.again, row.output, "separate stock fixed point");
  }
  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${build.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, build);
  const artifact = path.join(
    root,
    "target/differential/hugged-interpolation-indent/cli-report.json",
  );
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const report = {
    issue: 7969,
    corpusSha256: sha256(raw),
    buildReceipt: receipt,
    nativeHandled: 0,
    cliQualified: 0,
    rows: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(report, null, 2) + "\n");
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-hugged-interpolation-"));
  try {
    for (const fixture of corpus.cases) {
      await t.test(fixture.id, () => {
        for (const key of ["input", "expected"]) {
          const carrier = fs.readFileSync(path.join(directory, fixture[`${key}File`]));
          assert.equal(carrier.toString(), fixture[key]);
          assert.equal(sha256(carrier), fixture[`${key}Sha256`]);
        }
        const file = path.join(temporary, fixture.filename);
        fs.writeFileSync(file, fixture.input);
        const config = Buffer.from(fixture.config);
        assert.deepEqual(JSON.parse(fixture.config), { formatter: fixture.options });
        fs.writeFileSync(path.join(temporary, "vize.config.json"), config);
        const row = { id: fixture.id, fixture, attempts: [], diagnostics: [], qualified: false };
        report.rows.push(row);
        const observe = (source, pass) => {
          const complete = diagnostics(source, fixture);
          row.diagnostics.push({ pass, ...complete });
          persist();
          assert.equal(complete.vue, "3.5.35");
          validateDiagnostics(complete, { ...fixture, expectedParseErrors: [] });
          if (pass > 0) {
            assert.deepEqual(complete.dom, row.diagnostics[0].dom, "whole DOM compiler output");
            assert.deepEqual(complete.ssr, row.diagnostics[0].ssr, "whole SSR compiler output");
          }
        };
        observe(fixture.input, 0);
        for (const [index, oracle] of fixture.cliExpected.entries()) {
          const args = ["fmt", oracle.mode, fixture.filename];
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
            assert.equal(after.toString(), fixture.expected, `whole pass ${index}`);
            observe(after.toString(), index);
          }
        }
        assert.equal(row.attempts.length, 5);
        row.qualified = true;
        report.cliQualified++;
        persist();
      });
    }
    assert.equal(report.cliQualified, 13);
    assert.equal(
      report.rows.reduce((sum, row) => sum + row.attempts.length, 0),
      65,
    );
  } finally {
    persist();
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
