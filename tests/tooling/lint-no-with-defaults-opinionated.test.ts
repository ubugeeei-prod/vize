import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.ts";

interface CorpusCase {
  id: string;
  input: string;
  preset?: string;
  maxWarnings?: number;
  spans: [number, number][];
}

interface Corpus {
  schema: string;
  version: number;
  issue: number;
  baseline: string;
  sources: Record<string, { filename: string; bytes: number; sha256: string }>;
  cases: CorpusCase[];
}

const root = fileURLToPath(new URL("../../", import.meta.url));
const fixture = path.join(
  root,
  "tests/_fixtures/differential/linter/no-with-defaults-opinionated-8338",
);
const rule = "script/no-with-defaults";
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function location(bytes: Buffer, offset: number) {
  const lines = bytes.subarray(0, offset).toString("utf8").split("\n");
  return { line: lines.length, column: Array.from(lines.at(-1)!).length + 1 };
}

function expected(filename: string, bytes: Buffer, entry: CorpusCase) {
  return [
    {
      file: filename,
      messages: entry.spans.map(([start, end]) => {
        const from = location(bytes, start);
        const to = location(bytes, end);
        return {
          ruleId: rule,
          ruleDocsPath: "docs/content/rules/type-and-script.md",
          severity: 1,
          message: `[vize:${rule}] Prefer destructuring defaults over withDefaults (Vue 3.5+)`,
          line: from.line,
          column: from.column,
          endLine: to.line,
          endColumn: to.column,
          help: "Use destructuring with defaults: const { count = 0, name = 'default' } = defineProps<Props>()",
        };
      }),
      errorCount: 0,
      warningCount: entry.spans.length,
    },
  ];
}

await test("opinionated activates the existing withDefaults warning through the source-built CLI", () => {
  const corpus = JSON.parse(fs.readFileSync(path.join(fixture, "cases.json"), "utf8")) as Corpus;
  assert.equal(corpus.schema, "vize.lint.no-with-defaults-opinionated");
  assert.equal(corpus.version, 1);
  assert.equal(corpus.issue, 8338);
  assert.equal(corpus.baseline, "f46c40071c78b191f382956a88aef1bef7380b16");
  assert.equal(corpus.cases.length, 13);
  assert.equal(new Set(corpus.cases.map((entry) => entry.id)).size, 13);
  for (const [input, pin] of Object.entries(corpus.sources)) {
    const bytes = fs.readFileSync(path.join(fixture, input));
    assert.equal(bytes.length, pin.bytes, input);
    assert.equal(sha256(bytes), pin.sha256, input);
  }

  const build = expectedBuildIdentity(root);
  const receipt = JSON.parse(
    fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8"),
  );
  validateBuildReceipt(receipt, build);
  const binary = path.join(root, build.binaryPath);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-opinionated-with-defaults-"));
  const artifact = path.join(root, "target/differential/no-with-defaults-opinionated-8338.json");
  fs.mkdirSync(path.dirname(artifact), { recursive: true });
  const evidence = {
    schema: "vize.lint.no-with-defaults-opinionated.receipt",
    version: 1,
    issue: 8338,
    corpus,
    build: { expected: build, receipt },
    nativeHandled: 0,
    cliQualified: 0,
    runs: [] as unknown[],
  };
  const persist = () => fs.writeFileSync(artifact, JSON.stringify(evidence, null, 2) + "\n");
  try {
    for (const entry of corpus.cases) {
      const pin = corpus.sources[entry.input];
      assert.ok(pin, entry.id);
      const bytes = fs.readFileSync(path.join(fixture, entry.input));
      const destination = path.join(directory, pin.filename);
      fs.writeFileSync(destination, bytes);
      const oracle = expected(pin.filename, bytes, entry);
      const args = [
        "lint",
        "--no-config",
        "--format",
        "json",
        "--locale",
        "en",
        "--help-level",
        "full",
      ];
      if (entry.preset !== undefined) args.push("--preset", entry.preset);
      if (entry.maxWarnings !== undefined) args.push("--max-warnings", String(entry.maxWarnings));
      args.push(pin.filename);
      const overWarningLimit =
        entry.maxWarnings !== undefined && entry.spans.length > entry.maxWarnings;
      for (let repeat = 0; repeat < 2; repeat++) {
        const process = spawnSync(binary, args, {
          cwd: directory,
          encoding: "utf8",
          timeout: 60_000,
          maxBuffer: 4 * 1024 * 1024,
          env: { ...globalThis.process.env, NO_COLOR: "1" },
        });
        const run = {
          args,
          status: process.status,
          signal: process.signal,
          error: process.error
            ? { name: process.error.name, message: process.error.message }
            : null,
          stdout: process.stdout ?? "",
          stderr: process.stderr ?? "",
        };
        evidence.runs.push({
          entry,
          repeat,
          source: bytes.toString("utf8"),
          sha256: sha256(bytes),
          oracle,
          run,
        });
        persist();
        assert.equal(run.error, null, JSON.stringify(run));
        assert.equal(run.signal, null, JSON.stringify(run));
        assert.equal(run.status, overWarningLimit ? 1 : 0, JSON.stringify(run));
        assert.equal(run.stderr, overWarningLimit ? "\nToo many warnings (1 > max 0)\n" : "");
        assert.deepEqual(JSON.parse(run.stdout), oracle, entry.id);
        assert.equal(sha256(fs.readFileSync(destination)), pin.sha256);
        evidence.cliQualified++;
      }
    }
    assert.equal(evidence.cliQualified, 26);
  } finally {
    persist();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
