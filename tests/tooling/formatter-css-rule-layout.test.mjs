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
import { diagnostics, observeCss } from "./support/css-rule-layout-semantics.mjs";
import { observeSemantics } from "./support/formatter-template-semantics.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpusPath = "tests/_fixtures/differential/formatter-regressions/css-rule-layout-7926-7966";
const directory = path.join(root, corpusPath);
const raw = fs.readFileSync(path.join(directory, "cases.json"));
const corpus = JSON.parse(raw);

await test("whole original CSS selector lists and adjacent rule gaps", async (t) => {
  assert.equal(corpus.schema, "vize.formatter-css-rule-layout");
  assert.equal(corpus.version, 1);
  assert.deepEqual(corpus.issues, [7926, 7966]);
  assert.equal(corpus.cases.length, 17);
  assert.equal(new Set(corpus.cases.map((row) => row.id)).size, 17);
  const wholeSources = new Map();
  for (const issue of corpus.originalIssueBodies) {
    const body = fs.readFileSync(path.join(directory, issue.file));
    assert.equal(body.length, issue.bytes);
    assert.equal(sha256(body), issue.sha256);
    wholeSources.set(issue.issue, [...body.toString().matchAll(/```vue\n([\s\S]*?)```/g)]);
  }
  for (const [issue, index, name] of [
    [7926, 0, "App"],
    [7926, 1, "Insert"],
    [7966, 0, "Rules"],
  ]) {
    assert.equal(
      fs.readFileSync(path.join(directory, `${name}.vue.txt`), "utf8"),
      wholeSources.get(issue)[index][1],
      "complete unchanged original fenced input",
    );
  }
  assert.deepEqual(
    fs.readFileSync(path.join(directory, "App-crlf.vue.txt")),
    Buffer.from(
      fs.readFileSync(path.join(directory, "App.vue.txt"), "utf8").replaceAll("\n", "\r\n"),
    ),
  );
  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${build.binaryPath}.differential-build.json`)),
  );
  validateBuildReceipt(receipt, build);
  const artifact = path.join(root, "target/differential/formatter-css-rule-layout/cli-report.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const report = {
    schema: corpus.schema,
    issues: corpus.issues,
    corpusSha256: sha256(raw),
    buildReceipt: receipt,
    nativeHandled: 0,
    cliQualified: 0,
    rows: [],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(report, null, 2) + "\n");
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-rule-layout-"));
  let browser;
  try {
    const { chromium } = createRequire(path.join(root, "tests/package.json"))("@playwright/test");
    browser = await chromium.launch();
    for (const fixture of corpus.cases) {
      await t.test(fixture.id, async () => {
        const input = fs.readFileSync(path.join(directory, fixture.source));
        const expected = fs.readFileSync(path.join(directory, fixture.expected));
        assert.equal(input.length, fixture.bytes);
        assert.equal(sha256(input), fixture.sha256);
        assert.equal(expected.length, fixture.expectedBytes);
        assert.equal(sha256(expected), fixture.expectedSha256);
        const file = path.join(temporary, fixture.filename);
        fs.writeFileSync(file, input);
        const config = Buffer.from(JSON.stringify({ formatter: fixture.options ?? {} }) + "\n");
        fs.writeFileSync(path.join(temporary, "vize.config.json"), config);
        const row = {
          id: fixture.id,
          fixture,
          original: input.toString(),
          expected: expected.toString(),
          attempts: [],
          diagnostics: [],
          css: [],
          qualified: false,
        };
        report.rows.push(row);
        persist();
        const observe = async (source, pass) => {
          const complete = diagnostics(source, fixture);
          row.diagnostics.push({ pass, ...complete });
          row.css.push({ pass, ...(await observeCss(browser, complete)) });
          persist();
        };
        await observe(input.toString(), 0);
        if (fixture.states)
          row.originalSemantics = await observeSemantics(input.toString(), {
            ...fixture,
            file: fixture.filename,
          });
        const invoke = (mode) => {
          const args = [
            "fmt",
            mode,
            ...(fixture.options ? ["--config", "vize.config.json"] : ["--no-config"]),
            fixture.filename,
          ];
          const before = fs.readFileSync(file);
          const result = spawnSync(path.join(root, build.binaryPath), args, {
            cwd: temporary,
            env: { ...process.env, NO_COLOR: "1" },
            timeout: 60_000,
            maxBuffer: 8 * 1024 * 1024,
          });
          const after = fs.readFileSync(file);
          const capture = {
            args,
            status: result.status,
            signal: result.signal,
            error: result.error?.message ?? null,
            stdoutBase64: (result.stdout ?? Buffer.alloc(0)).toString("base64"),
            stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
            before: before.toString(),
            after: after.toString(),
          };
          row.attempts.push(capture);
          persist();
          assert.equal(capture.error, null);
          assert.equal(capture.signal, null);
          assert.equal(capture.stderrBase64, "");
          assert.deepEqual(fs.readFileSync(path.join(temporary, "vize.config.json")), config);
          return { capture, before, after };
        };
        const check = invoke("--check");
        assert.equal(check.capture.status, input.equals(expected) ? 0 : 1);
        assert.deepEqual(check.before, check.after, "initial check never writes");
        for (let pass = 1; pass <= 3; pass++) {
          const formatted = invoke("--write");
          assert.equal(formatted.capture.status, 0);
          assert.deepEqual(formatted.after, expected, `whole authored output pass ${pass}`);
          await observe(formatted.after.toString(), pass);
          assert.deepEqual(
            row.css.at(-1).cssRules,
            row.css[0].cssRules,
            "complete real CSSOM rules",
          );
          assert.deepEqual(
            row.css.at(-1).computed,
            row.css[0].computed,
            "complete computed declarations",
          );
          assert.equal(row.css.at(-1).html, row.css[0].html);
        }
        if (fixture.states) {
          row.formattedSemantics = await observeSemantics(expected.toString(), {
            ...fixture,
            file: fixture.filename,
          });
          assert.deepEqual(
            row.formattedSemantics.states,
            row.originalSemantics.states,
            "whole DOM and SSR output",
          );
        }
        const final = invoke("--check");
        assert.equal(final.capture.status, 0);
        assert.deepEqual(final.before, final.after, "final check never writes");
        assert.equal(row.attempts.length, 5);
        row.qualified = true;
        report.cliQualified++;
        persist();
      });
    }
    assert.equal(report.rows.length, 17);
    assert.equal(report.cliQualified, 17);
    assert.equal(
      report.rows.reduce((count, row) => count + row.attempts.length, 0),
      85,
    );
  } finally {
    persist();
    try {
      if (browser) await browser.close();
    } finally {
      fs.rmSync(temporary, { recursive: true, force: true });
    }
  }
});
