import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";
import { sha256 } from "../differential/sha256.ts";

interface FormatterCase {
  id: string;
  source: string;
  sourceSha256: string;
  expected: string;
  expectedSha256: string;
  options: Record<string, unknown>;
}
interface LinterCase {
  id: string;
  source: string;
  options: { filename: string; enabledRules: string[] };
}
interface SourceBaseline {
  source: string;
  nativeSHA256: string;
  [field: string]: unknown;
}
interface Corpus {
  schema: string;
  issue: number;
  relatedIssue: number;
  originals: { source: string }[];
  sourceBaseline: SourceBaseline;
  formatterCases: FormatterCase[];
  styleCases: unknown[];
  linterCases: LinterCase[];
}
interface BaselineRun {
  id: string;
  originalSource: string;
  referenceSource: string;
  exit: number;
  stderrHex: string;
  stderr: string;
  stdoutHex: string;
  stdout: string;
}
interface Baseline {
  sourceBaseline: SourceBaseline;
  scope: unknown;
  runs: BaselineRun[];
}
interface Attempt {
  id: string;
  args: string[];
  cwd: string;
  config: string;
  status: number | null;
  signal: NodeJS.Signals | null;
  error: { name: string; message: string } | null;
  stdout: string;
  stderr: string;
  stdoutBase64: string;
  stderrBase64: string;
  before: string;
  beforeBase64: string;
  after: string | null;
  afterBase64: string | null;
  readError: string | null;
  expected?: unknown;
  actual?: unknown;
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/css-engine-boundary-3295");
const cliFixture = path.join(root, "tests/_fixtures/differential/css-engine-boundary-cli-3295");
const corpusBytes = fs.readFileSync(path.join(fixture, "cases.json"));
assert.equal(
  sha256(corpusBytes),
  "32b8bb5632bc0755d1540ff828f3e70eb5e41ec45cb0c81de4df67a787f572c1",
);
const corpus: Corpus = JSON.parse(corpusBytes.toString("utf8"));
const expectedBytes = fs.readFileSync(path.join(cliFixture, "expected.json"));
assert.equal(
  sha256(expectedBytes),
  "df020b3786c5213819c07cdacda718968938750910bfcc245090016af818e019",
);
const expected: { cases: Record<string, unknown> } = JSON.parse(expectedBytes.toString("utf8"));
const baselineBytes = fs.readFileSync(path.join(cliFixture, "capture.json"));
assert.equal(
  sha256(baselineBytes),
  "ee393a3e8982c2e46bbbec4732c6d02265c432b7c3bae1d98e9cf201844aa609",
);
const baseline: Baseline = JSON.parse(baselineBytes.toString("utf8"));

void test("original CSS bytes and whole healthy/fallback CLI authorities remain pinned", () => {
  assert.equal(corpus.schema, "vize.css-engine-boundary-3295");
  assert.equal(corpus.issue, 3295);
  assert.equal(corpus.relatedIssue, 4961);
  assert.equal(corpus.formatterCases.length, 12);
  assert.equal(corpus.styleCases.length, 6);
  assert.equal(corpus.linterCases.length, 15);
  assert.deepEqual(
    corpus.originals.map((row) => row.source),
    ["a{opacity:abs(-50%)}", "a{color:hsl(0 abs(-50%) 0%)}", "a{text-size-adjust:calc(5)}"],
  );
  assert.deepEqual(baseline.sourceBaseline, corpus.sourceBaseline);
  assert.equal(baseline.sourceBaseline.source, "3390cb6044d53d655a9d64e112d2618375cff5ca");
  assert.equal(
    baseline.sourceBaseline.nativeSHA256,
    "b884696038e3e6c09054b73d9532a3ab490d2cd296a5770d836561b0be245d1d",
  );
  assert.equal(baseline.runs.length, 15);
  assert.deepEqual(
    Object.keys(expected.cases),
    corpus.linterCases.map((row) => row.id),
  );
  assert.deepEqual(
    baseline.runs.map((row) => row.id),
    Object.keys(expected.cases),
  );
  for (const [index, row] of baseline.runs.entries()) {
    const original = corpus.linterCases[index];
    assert.equal(row.originalSource, original.source);
    assert.equal(Buffer.byteLength(row.referenceSource), Buffer.byteLength(original.source));
    assert.equal(row.exit, 0);
    assert.equal(row.stderrHex, "");
    assert.equal(row.stderr, "");
    assert.equal(Buffer.from(row.stdoutHex, "hex").toString(), row.stdout);
    assert.deepEqual(JSON.parse(row.stdout), expected.cases[row.id]);
  }
  for (const row of corpus.formatterCases) {
    assert.equal(sha256(Buffer.from(row.source)), row.sourceSha256);
    assert.equal(sha256(Buffer.from(row.expected)), row.expectedSha256);
  }
});

await test("source-built CSS consumers survive exact originals and preserve whole healthy outputs", async (t) => {
  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, `${build.binaryPath}.differential-build.json`), "utf8"),
  );
  validateBuildReceipt(receipt, build);
  const destination = path.join(
    root,
    "target/differential/css-engine-boundary-3295/cli-report.json",
  );
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  const report = {
    schema: "vize.css-engine-boundary-cli",
    version: 1,
    issue: 3295,
    buildReceipt: receipt,
    corpusSha256: sha256(corpusBytes),
    cliReferenceSha256: sha256(expectedBytes),
    baselineScope: baseline.scope,
    nativeHandled: 0,
    attempts: [] as Attempt[],
  };
  const persist = () => fs.writeFileSync(destination, JSON.stringify(report, null, 2) + "\n");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-engine-boundary-"));
  const cli = path.join(root, build.binaryPath);
  const invoke = (id: string, args: string[], file: string, config: string): Attempt => {
    const before = fs.readFileSync(file);
    const result = spawnSync(cli, args, {
      cwd: directory,
      env: { ...process.env, NO_COLOR: "1" },
      timeout: 60_000,
      maxBuffer: 8 * 1024 * 1024,
    });
    const row: Attempt = {
      id,
      args,
      cwd: directory,
      config,
      status: result.status,
      signal: result.signal,
      error: result.error ? { name: result.error.name, message: result.error.message } : null,
      stdout: (result.stdout ?? Buffer.alloc(0)).toString(),
      stderr: (result.stderr ?? Buffer.alloc(0)).toString(),
      stdoutBase64: (result.stdout ?? Buffer.alloc(0)).toString("base64"),
      stderrBase64: (result.stderr ?? Buffer.alloc(0)).toString("base64"),
      before: before.toString(),
      beforeBase64: before.toString("base64"),
      after: null,
      afterBase64: null,
      readError: null,
    };
    report.attempts.push(row);
    persist();
    try {
      const after = fs.readFileSync(file);
      row.after = after.toString();
      row.afterBase64 = after.toString("base64");
    } catch (error) {
      row.readError = error instanceof Error ? error.message : String(error);
    }
    persist();
    assert.equal(row.error, null, JSON.stringify(row));
    assert.equal(row.signal, null, JSON.stringify(row));
    assert.equal(row.readError, null, JSON.stringify(row));
    return row;
  };
  try {
    for (const entry of corpus.formatterCases) {
      await t.test(`formatter/${entry.id}`, () => {
        const file = path.join(directory, "App.vue");
        fs.writeFileSync(file, entry.source);
        const config = JSON.stringify({ formatter: entry.options }) + "\n";
        fs.writeFileSync(path.join(directory, "vize.config.json"), config);
        const options = Object.keys(entry.options).length
          ? ["--config", "vize.config.json"]
          : ["--no-config"];
        for (const [pass, mode] of [
          "--check",
          "--write",
          "--write",
          "--write",
          "--check",
        ].entries()) {
          const row = invoke(
            `formatter/${entry.id}/${pass + 1}`,
            ["fmt", ...options, mode, "App.vue"],
            file,
            config,
          );
          row.expected = entry.expected;
          persist();
          assert.equal(
            row.status,
            pass === 0 && entry.source !== entry.expected ? 1 : 0,
            JSON.stringify(row),
          );
          if (mode === "--check")
            assert.equal(row.afterBase64, row.beforeBase64, "check is read-only");
          if (pass !== 0) assert.equal(row.after, entry.expected, `${entry.id} whole fixedpoint`);
        }
        fs.unlinkSync(file);
      });
    }
    for (const entry of corpus.linterCases) {
      await t.test(`linter/${entry.id}`, () => {
        const filename = entry.options.filename;
        const file = path.join(directory, filename);
        fs.writeFileSync(file, entry.source);
        const config =
          JSON.stringify({
            linter: {
              preset: "incremental",
              rules: Object.fromEntries(entry.options.enabledRules.map((rule) => [rule, "warn"])),
            },
          }) + "\n";
        fs.writeFileSync(path.join(directory, "vize.config.json"), config);
        let previous: string | undefined;
        for (let pass = 1; pass <= 2; pass += 1) {
          const row = invoke(
            `linter/${entry.id}/${pass}`,
            [
              "lint",
              "-f",
              "json",
              "--help-level",
              "full",
              "--locale",
              "en",
              "--config",
              "vize.config.json",
              filename,
            ],
            file,
            config,
          );
          row.expected = expected.cases[entry.id];
          persist();
          assert.equal(row.status, 0, JSON.stringify(row));
          assert.equal(row.after, entry.source, "lint preserves the whole original source");
          row.actual = JSON.parse(row.stdout);
          persist();
          assert.deepEqual(row.actual, row.expected, `${entry.id} complete CLI vector`);
          if (previous !== undefined)
            assert.equal(row.stdout, previous, "fresh processes retain complete JSON bytes");
          previous = row.stdout;
        }
        fs.unlinkSync(file);
      });
    }
    assert.equal(report.attempts.length, 90);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
