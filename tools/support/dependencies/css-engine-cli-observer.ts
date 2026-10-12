// Diagnostic source execution: preserve all90 whole processes without rewriting expectations.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const [binaryArgument, destination, profile] = process.argv.slice(2);
assert(binaryArgument && destination && ["dev", "release"].includes(profile));
const root = process.cwd();
const binary = path.resolve(binaryArgument);
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const corpusBytes = fs.readFileSync(
  "tests/_fixtures/differential/css-engine-boundary-3295/cases.json",
);
assert.equal(
  digest(corpusBytes),
  "32b8bb5632bc0755d1540ff828f3e70eb5e41ec45cb0c81de4df67a787f572c1",
);
const corpus = JSON.parse(corpusBytes.toString());
assert.equal(corpus.formatterCases.length, 12);
assert.equal(corpus.linterCases.length, 15);
const expectedBytes = fs.readFileSync(
  "tests/_fixtures/differential/css-engine-boundary-cli-3295/expected.json",
);
assert.equal(
  digest(expectedBytes),
  "df020b3786c5213819c07cdacda718968938750910bfcc245090016af818e019",
);
const expected = JSON.parse(expectedBytes.toString());
const git = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" });
assert.equal(git.status, 0);
const report = {
  schema: "vize.css-engine-vendor-cli-observations",
  sourceRevision: git.stdout.trim(),
  binarySha256: digest(fs.readFileSync(binary)),
  cliVersion: null as string | null,
  versionAttempt: null as Record<string, unknown> | null,
  profile,
  corpusSha256: digest(corpusBytes),
  beforeReferenceSha256: digest(expectedBytes),
  expectationsRecaptured: false,
  publicationQualification: false,
  completed: false,
  attempts: [] as Record<string, unknown>[],
};
const persist = () => fs.writeFileSync(destination, JSON.stringify(report, null, 2) + "\n");
persist();
const version = spawnSync(binary, ["--version"], { encoding: "utf8", timeout: 60_000 });
report.versionAttempt = {
  status: version.status,
  signal: version.signal,
  error: version.error ? { name: version.error.name, message: version.error.message } : null,
  stdout: version.stdout,
  stderr: version.stderr,
};
persist();
assert.equal(version.error, undefined, version.stderr);
assert.equal(version.signal, null, version.stderr);
assert.equal(version.status, 0, version.stderr);
report.cliVersion = version.stdout.trim();
persist();
const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-vendor-observer-"));
function invoke(id: string, args: string[], file: string, config: string) {
  const before = fs.readFileSync(file);
  const result = spawnSync(binary, args, {
    cwd: directory,
    env: { ...process.env, NO_COLOR: "1" },
    timeout: 60_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  const row: Record<string, unknown> = {
    id,
    args,
    cwd: directory,
    config,
    status: result.status,
    signal: result.signal,
    error: result.error ? { name: result.error.name, message: result.error.message } : null,
    stdout: stdout.toString(),
    stdoutBase64: stdout.toString("base64"),
    stderr: stderr.toString(),
    stderrBase64: stderr.toString("base64"),
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
}
try {
  for (const entry of corpus.formatterCases) {
    const file = path.join(directory, "App.vue");
    fs.writeFileSync(file, entry.source);
    const config = JSON.stringify({ formatter: entry.options }) + "\n";
    fs.writeFileSync(path.join(directory, "vize.config.json"), config);
    const options = Object.keys(entry.options).length
      ? ["--config", "vize.config.json"]
      : ["--no-config"];
    for (const [pass, mode] of ["--check", "--write", "--write", "--write", "--check"].entries()) {
      const row = invoke(
        `formatter/${entry.id}/${pass + 1}`,
        ["fmt", ...options, mode, "App.vue"],
        file,
        config,
      );
      row.beforeExpected = entry.expected;
      persist();
      if (mode === "--check") assert.equal(row.afterBase64, row.beforeBase64);
    }
    fs.unlinkSync(file);
  }
  for (const entry of corpus.linterCases) {
    const filename = entry.options.filename;
    const file = path.join(directory, filename);
    fs.writeFileSync(file, entry.source);
    const config =
      JSON.stringify({
        linter: {
          preset: "incremental",
          rules: Object.fromEntries(
            entry.options.enabledRules.map((rule: string) => [rule, "warn"]),
          ),
        },
      }) + "\n";
    fs.writeFileSync(path.join(directory, "vize.config.json"), config);
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
      row.beforeExpected = expected.cases[entry.id];
      persist();
      assert.equal(row.after, entry.source);
      row.actual = JSON.parse(row.stdout as string);
      persist();
    }
    fs.unlinkSync(file);
  }
  assert.equal(report.attempts.length, 90);
  report.completed = true;
  persist();
} finally {
  persist();
  fs.rmSync(directory, { recursive: true, force: true });
}
console.log(
  JSON.stringify({
    attempts: report.attempts.length,
    sourceRevision: report.sourceRevision,
    expectationsRecaptured: false,
    publicationQualification: false,
    root,
  }),
);
