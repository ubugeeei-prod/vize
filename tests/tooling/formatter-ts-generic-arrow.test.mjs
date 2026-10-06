import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { qualifyReference } from "../differential/formatter-vite-observation.mjs";
import { sha256 } from "../differential/manifest.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const corpusPath = "tests/_fixtures/differential/formatter-regressions/ts-generic-arrow-7965";
const corpus = path.join(root, corpusPath);

void test("original TS generics retain full bytes while TSX keeps disambiguation", async (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ts-generic-arrow-"));
  const report = {
    schema: "vize.formatter.ts-generic-arrow-observation",
    version: 1,
    issue: 7965,
    nativeHandled: 0,
    cliQualified: 0,
    referenceQualified: 0,
    rows: [],
  };
  try {
    const identity = expectedBuildIdentity(root);
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, `${identity.binaryPath}.differential-build.json`)),
    );
    validateBuildReceipt(receipt, identity);
    report.build = { expected: identity, receipt };
    const manifestRaw = fs.readFileSync(path.join(corpus, "manifest.json"));
    const manifest = JSON.parse(manifestRaw);
    const sourceRaw = fs.readFileSync(path.join(corpus, "source.json"));
    const original = JSON.parse(sourceRaw);
    assert.equal(manifest.issue, 7965);
    assert.equal(manifest.cases.length, 12);
    report.corpus = { path: corpusPath, manifestSha256: sha256(manifestRaw), source: original };
    for (const [file, expected] of Object.entries(original.files)) {
      const bytes = fs.readFileSync(path.join(corpus, file));
      assert.equal(bytes.length, expected.bytes);
      assert.equal(sha256(bytes), expected.sha256);
    }

    const require = createRequire(path.join(root, "package.json"));
    const packagePath = fs.realpathSync(require.resolve("oxfmt/package.json"));
    const packageRaw = fs.readFileSync(packagePath);
    const version = JSON.parse(packageRaw).version;
    assert.equal(version, manifest.reference.workspaceVersion);
    qualifyReference(root, require, version);
    const entry = fs.realpathSync(require.resolve("oxfmt"));
    const { format } = await import(pathToFileURL(entry).href);
    assert.equal(typeof format, "function");
    report.reference = {
      version,
      packagePath,
      packageSha256: sha256(packageRaw),
      entry,
      entrySha256: sha256(fs.readFileSync(entry)),
      lockSha256: sha256(fs.readFileSync(path.join(root, "pnpm-lock.yaml"))),
    };
    const invoke = (cwd, argv) => {
      const result = spawnSync(path.join(root, identity.binaryPath), argv, {
        cwd,
        env: { ...process.env, NO_COLOR: "1" },
        encoding: "utf8",
        timeout: 30000,
        maxBuffer: 1048576,
      });
      const observation = {
        argv,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout ?? "",
        stderr: result.stderr ?? "",
      };
      return observation;
    };

    const reported = path.join(directory, "reported");
    fs.mkdirSync(path.join(reported, ".git"), { recursive: true });
    const originalFiles = { "Generic.vue": "Generic.vue.txt", "plain.ts": "plain.ts.txt" };
    for (const [file, fixture] of Object.entries(originalFiles))
      fs.copyFileSync(path.join(corpus, fixture), path.join(reported, file));
    report.originalCommand = invoke(reported, [
      "fmt",
      "--write",
      "--no-config",
      "Generic.vue",
      "plain.ts",
    ]);
    report.originalOutputs = Object.fromEntries(
      Object.keys(originalFiles).map((file) => {
        const actual = fs.readFileSync(path.join(reported, file));
        return [file, { source: actual.toString(), sha256: sha256(actual) }];
      }),
    );
    assert.equal(report.originalCommand.error, null);
    assert.equal(report.originalCommand.signal, null);
    assert.equal(report.originalCommand.status, 0);
    assert.equal(report.originalCommand.stdout, "");
    assert.equal(
      report.originalCommand.stderr,
      "Found 2 file(s)\n\nFormatted 2 file(s)\n  2 file(s) unchanged\n",
    );
    for (const [file, fixture] of Object.entries(originalFiles))
      assert.deepEqual(
        Buffer.from(report.originalOutputs[file].source),
        fs.readFileSync(path.join(corpus, fixture)),
      );

    for (const fixture of manifest.cases) {
      await t.test(fixture.id, async () => {
        const row = { id: fixture.id, fixture, reference: [], attempts: [] };
        report.rows.push(row);
        assert.equal(Buffer.byteLength(fixture.source), fixture.bytes);
        assert.equal(sha256(Buffer.from(fixture.source)), fixture.sha256);
        assert.equal(Buffer.byteLength(fixture.expected), fixture.expectedBytes);
        assert.equal(sha256(Buffer.from(fixture.expected)), fixture.expectedSha256);
        const options = {
          printWidth: 100,
          tabWidth: 2,
          semi: true,
          singleQuote: false,
          trailingComma: "all",
          vueIndentScriptAndStyle: false,
          endOfLine: fixture.endOfLine ?? "lf",
        };
        let reference = fixture.source;
        for (let pass = 1; pass <= 3; pass++) {
          const result = await format(fixture.file, reference, options);
          row.reference.push({ pass, input: reference, ...result });
          assert.deepEqual(result.errors, []);
          assert.equal(result.code, fixture.expected, "whole actual Oxfmt reference");
          reference = result.code;
        }
        report.referenceQualified++;
        const cwd = path.join(directory, fixture.id);
        fs.mkdirSync(path.join(cwd, ".git"), { recursive: true });
        const file = path.join(cwd, fixture.file);
        fs.writeFileSync(file, fixture.source);
        if (fixture.endOfLine)
          fs.writeFileSync(
            path.join(cwd, "vize.config.json"),
            `${JSON.stringify({ formatter: { endOfLine: fixture.endOfLine } })}\n`,
          );
        const call = (mode) => {
          const before = fs.readFileSync(file);
          const result = invoke(cwd, [
            "fmt",
            mode,
            ...(fixture.endOfLine ? ["--config", "vize.config.json"] : ["--no-config"]),
            fixture.file,
          ]);
          const after = fs.readFileSync(file);
          row.attempts.push({
            ...result,
            before: before.toString(),
            after: after.toString(),
            beforeSha256: sha256(before),
            afterSha256: sha256(after),
          });
          assert.equal(result.error, null);
          assert.equal(result.signal, null);
          assert.equal(result.stdout, "");
          const changed = before.toString() !== fixture.expected;
          const expectedStderr = [
            "Found 1 file(s)\n",
            changed
              ? `${mode === "--check" ? "Would reformat" : "Reformatted"}: ${fixture.file}\n`
              : "",
            `\n${mode === "--check" ? "Checked" : "Formatted"} 1 file(s)\n`,
            mode === "--check"
              ? `  1 file(s) ${changed ? "would be reformatted" : "already formatted"}\n`
              : `  1 file(s) ${changed ? "reformatted" : "unchanged"}\n`,
          ].join("");
          assert.equal(result.stderr, expectedStderr);
          return { result, before, after };
        };
        const initial = call("--check");
        assert.deepEqual(initial.after, initial.before);
        assert.equal(initial.result.status, fixture.source === fixture.expected ? 0 : 1);
        for (let pass = 1; pass <= 3; pass++) {
          const { result, after } = call("--write");
          assert.equal(result.status, 0);
          assert.deepEqual(after, Buffer.from(fixture.expected), `whole pass ${pass}`);
        }
        const final = call("--check");
        assert.equal(final.result.status, 0);
        assert.deepEqual(final.after, final.before);
        row.qualified = true;
        report.cliQualified++;
      });
    }
    assert.equal(report.rows.length, 12);
    assert.equal(report.referenceQualified, 12);
    assert.equal(report.cliQualified, 12);
  } finally {
    try {
      const artifact = path.join(root, "target/differential/formatter-ts-generic-arrow.json");
      fs.mkdirSync(path.dirname(artifact), { recursive: true });
      fs.writeFileSync(artifact, `${JSON.stringify(report, null, 2)}\n`);
      t.diagnostic(`Whole originals, real Oxfmt and source-bound CLI: ${artifact}`);
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});
