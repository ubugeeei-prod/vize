import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "crates/vize_patina/tests/fixtures/css-utility-literal-tokens");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

await test("whole original utility-class reports exclude literal CSS tokens", async (t) => {
  const manifest = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8"));
  assert.equal(manifest.issue, 7980);
  assert.deepEqual(manifest.reporter, {
    login: "ubugeeei",
    id: 71201308,
    coauthor: "ubugeeei <71201308+ubugeeei@users.noreply.github.com>",
  });
  assert.equal(manifest.cases.length, 10);
  assert.equal(new Set(manifest.cases.map((entry) => entry.id)).size, 10);
  const originalIssue = fs.readFileSync(path.join(fixture, "original-issue.md"), "utf8");
  assert.equal(sha256(originalIssue), manifest.originalIssueBodySha256);
  const originalVue = [...originalIssue.matchAll(/```vue\n([\s\S]*?)```/g)].map((row) => row[1]);
  assert.equal(originalVue.length, 2);
  for (const [index, name] of ["original-note.vue.txt", "original-quote.vue.txt"].entries()) {
    assert.equal(fs.readFileSync(path.join(fixture, name), "utf8"), originalVue[index]);
  }
  const config = fs.readFileSync(path.join(fixture, manifest.config.file));
  assert.equal(config.length, manifest.config.bytes);
  assert.equal(sha256(config), manifest.config.sha256);
  assert.equal(config.toString("utf8"), originalIssue.match(/```json\n([\s\S]*?)```/)[1]);
  assert.equal(
    fs.readFileSync(path.join(fixture, "original-note-crlf.vue.txt"), "utf8"),
    originalVue[0].replaceAll("\n", "\r\n"),
  );
  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${build.binaryPath}.differential-build.json`), "utf8"),
  );
  validateBuildReceipt(receipt, build);
  const cli = path.join(root, build.binaryPath);
  const output = path.join(root, "target/differential/css-utility-literal-tokens-cli.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const evidence = {
    schema: "vize.css-utility-literal-tokens",
    version: 1,
    issue: 7980,
    build,
    receipt,
    nativeHandled: 0,
    cliQualified: 0,
    runs: [],
  };
  const persist = () => fs.writeFileSync(output, JSON.stringify(evidence, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-utility-tokens-"));
  try {
    fs.writeFileSync(path.join(directory, "vize.config.json"), config);
    for (const entry of manifest.cases) {
      await t.test(entry.id, () => {
        const source = fs.readFileSync(path.join(fixture, entry.source));
        assert.equal(source.length, entry.bytes);
        assert.equal(sha256(source), entry.sha256);
        const expectedBytes = fs.readFileSync(path.join(fixture, entry.expected));
        assert.equal(expectedBytes.length, entry.expectedBytes);
        assert.equal(sha256(expectedBytes), entry.expectedSha256);
        const expected = JSON.parse(expectedBytes.toString("utf8"));
        const physical = path.join(directory, entry.filename);
        fs.writeFileSync(physical, source);
        const captures = [];
        try {
          for (const format of ["json", "plain", "json"]) {
            const args = [
              "lint",
              "-f",
              format,
              "--help-level",
              "none",
              "--locale",
              "en",
              entry.filename,
            ];
            const result = spawnSync(cli, args, {
              cwd: directory,
              timeout: 60_000,
              maxBuffer: 8 * 1024 * 1024,
            });
            const row = {
              case: entry,
              source: source.toString("utf8"),
              config: config.toString("utf8"),
              args,
              status: result.status,
              signal: result.signal,
              error: result.error
                ? { name: result.error.name, message: result.error.message }
                : null,
              stdoutBase64: (result.stdout ?? Buffer.alloc(0)).toString("base64"),
              stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
              expected: expected[format],
            };
            captures.push(row);
            evidence.runs.push(row);
            persist();
            assert.equal(row.error, null, JSON.stringify(row));
            assert.equal(row.signal, null, JSON.stringify(row));
            assert.equal(row.status, 0, JSON.stringify(row));
            assert.equal(row.stderrBase64, "", JSON.stringify(row));
            const stdout = Buffer.from(row.stdoutBase64, "base64").toString("utf8");
            if (format === "json") {
              row.actual = JSON.parse(stdout);
              persist();
              assert.deepEqual(row.actual, expected.json, entry.id);
            } else {
              assert.equal(stdout, expected.plain, entry.id);
            }
            assert.deepEqual(fs.readFileSync(physical), source);
            assert.deepEqual(fs.readFileSync(path.join(directory, "vize.config.json")), config);
            evidence.cliQualified++;
            persist();
          }
          assert.equal(
            captures[0].stdoutBase64,
            captures[2].stdoutBase64,
            "whole repeated JSON bytes",
          );
        } finally {
          fs.rmSync(physical, { force: true });
        }
      });
    }
    assert.equal(evidence.runs.length, 30);
    assert.equal(evidence.cliQualified, 30);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
