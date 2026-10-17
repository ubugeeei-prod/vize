import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
export const root = path.resolve(import.meta.dirname, "../../..");
export const command = "tools/commands/fixtures/lint-divergence-report";
export const contractDirectory = "tests/_fixtures/differential/linter/reference-coverage-8849";
export const originalRef = "49545646fde08ade1000c4a33b38c9dd1ac4fb99";
export const updatedRef = "7421901d05ac2e252636dd8ee426e8783255378d";
export type Oracle = "original" | "updated";
interface Source {
  ref: string;
  files: [string, string, number][];
}
export const referenceFixtures = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../_fixtures/differential/linter/reference-coverage-8849/report-references.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as Record<Oracle, Source> & {
  fakeVizeSource: string;
  invalidArguments: string[][];
  parserVectors: { token: string; reason: string; js: string }[];
  missingModernPaths: string[][];
  seams: Record<string, unknown>;
};
const sources = { original: referenceFixtures.original, updated: referenceFixtures.updated };
assert.equal(sources.original.ref, originalRef);
assert.equal(sources.updated.ref, updatedRef);
type Fields = Record<string, unknown>;
interface Finding extends Fields {
  file: string;
  line: number | bigint;
}
interface Raw {
  eslintResults: Fields[];
  patinaFindings: Finding[];
  patinaResults: (Fields & { file: string; messages: Fields[] })[];
}
interface Divergence extends Fields {
  schema: string;
  version: number;
  sha256: string;
  summary: Record<string, number>;
  referenceCoverageExpectations: { observedCount: number }[];
  falsePositives: Finding[];
  falseNegatives: (Finding & { upstreamRuleId: string })[];
}
interface Baseline {
  durationMs?: number;
  runtimeVersions?: unknown;
  comparedRuleCount: number;
  parseErrorResults: Fields[];
}
interface Budget {
  passed: boolean;
  verdict: string;
  maxFalseNegativeCount: number;
  maxFalsePositiveCount: number;
}
interface BaseReport {
  evidence: { runtime?: unknown };
  budget: Budget;
}
interface ProjectReport extends BaseReport {
  schema: "vize.fixtureLintDivergenceRun";
  baseline: Baseline;
  raw: Raw;
  divergence: Divergence;
  files: { comparedCount: number; expectedCount: number | null };
}
interface IndexReport extends BaseReport {
  schema: "vize.fixtureLintDivergenceIndex";
  projects: Record<string, number>[];
}
interface OriginalProjectReport extends Omit<ProjectReport, "raw" | "divergence"> {
  divergence: Omit<Divergence, "referenceCoverageExpectations">;
}
type Report = ProjectReport | OriginalProjectReport | IndexReport;
export type ReportSource = Oracle | "typescript";
interface ProcessCapture {
  argv: string[];
  cwd: string;
  lang: string;
  lcAll: string;
  stdin: string;
  stdout: string;
  stderr: string;
  status: number;
}
export function object(value: unknown): Fields {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Fields;
}
function reportValue(value: unknown, source: ReportSource): Report {
  assert.ok(["original", "updated", "typescript"].includes(source));
  const row = object(value);
  assert.equal(typeof row.schema, "string");
  assert.equal(
    object(object(row.evidence).runtime).name,
    source === "typescript" ? "node" : "rust-script",
  );
  object(row.budget);
  if (row.schema === "vize.fixtureLintDivergenceRun") {
    object(row.baseline);
    const divergence = object(row.divergence),
      baseline = object(row.baseline);
    object(row.files);
    const summary = object(divergence.summary);
    if (source === "original") {
      for (const [owner, key] of [
        [row, "raw"],
        [baseline, "runtimeVersions"],
        [divergence, "referenceCoverageExpectations"],
        [summary, "referenceCoverageIntegrityErrorCount"],
      ] as const)
        assert.equal(Object.hasOwn(owner, key), false, `original must not contain ${key}`);
    } else {
      const raw = object(row.raw);
      object(baseline.runtimeVersions);
      for (const key of ["patinaResults", "patinaFindings", "eslintResults"])
        assert.ok(Array.isArray(raw[key]));
      assert.ok(Array.isArray(divergence.referenceCoverageExpectations));
      assert.equal(typeof summary.referenceCoverageIntegrityErrorCount, "number");
    }
  } else {
    assert.equal(row.schema, "vize.fixtureLintDivergenceIndex");
    assert.ok(Array.isArray(row.projects));
  }
  return row as unknown as Report;
}
export function reportFile(result: Result, name: string): ProjectReport {
  const value = result.files[name];
  assert.ok(
    typeof value === "object" && value.schema === "vize.fixtureLintDivergenceRun" && "raw" in value,
  );
  return value;
}
export function indexFile(result: Result): IndexReport {
  const value = result.files["lint-divergence-summary.json"];
  assert.ok(typeof value === "object" && value.schema === "vize.fixtureLintDivergenceIndex");
  return value;
}
export function markdown(result: Result, name: string): string {
  const value = result.files[name];
  assert.equal(typeof value, "string");
  return value as string;
}
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
export function evidenceDirectory() {
  const directory =
    process.env.VIZE_LINT_REPORT_PARITY_DIR ??
    path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "lint-report-parity");
  fs.mkdirSync(directory, { recursive: true });
  return directory;
}
export function parseReport(text: string, source: ReportSource): Report {
  return reportValue(
    JSON.parse(text, (_key: string, value: unknown, context?: { source?: string }) => {
      if (
        typeof value === "number" &&
        !Number.isSafeInteger(value) &&
        typeof context?.source === "string" &&
        /^-?\d+$/u.test(context.source)
      ) {
        return BigInt(context.source);
      }
      return value;
    }) as unknown,
    source,
  );
}
export function writeSeamReceipt() {
  fs.writeFileSync(
    path.join(evidenceDirectory(), "finite-successor-seams.json"),
    JSON.stringify(referenceFixtures.seams, null, 2) + "\n",
  );
}
export function materializeFrozenReporters() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-frozen-lint-report-"));
  const paths = {} as Record<Oracle, string>;
  try {
    for (const [name, source] of Object.entries(sources)) {
      for (const [file, expectedHash, expectedBytes] of source.files) {
        const result = spawnSync("git", ["show", `${source.ref}:${file}`], { cwd: root });
        assert.equal(
          result.status,
          0,
          `Fetch exact frozen reference ${source.ref}: ${result.stderr.toString()}`,
        );
        assert.equal(result.stdout.length, expectedBytes, file);
        assert.equal(hash(result.stdout), expectedHash, `${source.ref}:${file}`);
        if (file === `${command}.rs`) {
          assert.equal(
            result.stdout.toString().trimEnd().split("\n").length,
            name === "original" ? 1958 : 2072,
          );
        }
        const target = path.join(directory, name, file);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.writeFileSync(target, result.stdout);
      }
      paths[name as Oracle] = path.join(directory, name, `${command}.rs`);
    }
    fs.writeFileSync(
      path.join(evidenceDirectory(), "frozen-source-custody.json"),
      JSON.stringify(sources, null, 2) + "\n",
    );
    return { paths, clean: () => fs.rmSync(directory, { recursive: true, force: true }) };
  } catch (error) {
    fs.rmSync(directory, { recursive: true, force: true });
    throw error;
  }
}
export interface Fixture {
  directory: string;
  output: string;
  transcript: string;
  args: string[];
}
let measurement = 0;
export function runReporter(
  input: Fixture,
  source: string,
  args = input.args,
  sourceVersion: ReportSource = "typescript",
) {
  const rust = sourceVersion !== "typescript";
  fs.rmSync(input.output, { recursive: true, force: true });
  fs.rmSync(input.transcript, { force: true });
  const result = spawnSync(rust ? "rust-script" : process.execPath, [source, ...args], {
    cwd: root,
    encoding: "utf8",
    timeout: 240_000,
    maxBuffer: 8 * 1024 * 1024,
    env: { ...process.env, VIZE_REPO_ROOT: root, LANG: "C", LC_ALL: "C" },
  });
  const retainedFiles = Object.fromEntries(
    fs.existsSync(input.output)
      ? fs
          .readdirSync(input.output)
          .map((name) => [name, fs.readFileSync(path.join(input.output, name)).toString("base64")])
      : [],
  );
  fs.writeFileSync(
    path.join(evidenceDirectory(), `${String(++measurement).padStart(3, "0")}-raw.json`),
    JSON.stringify(
      {
        source,
        sourceVersion,
        args,
        status: result.status,
        signal: result.signal,
        error: result.error?.message,
        stdout: result.stdout,
        stderr: result.stderr,
        filesBase64: retainedFiles,
        transcriptBase64: fs.existsSync(input.transcript)
          ? fs.readFileSync(input.transcript).toString("base64")
          : null,
      },
      null,
      2,
    ) + "\n",
  );
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null, `reporter terminated by ${result.signal}`);
  const files: Record<string, Report | string> = {};
  const rawFiles: Record<string, string> = {};
  const rawUnknownTokens: Record<string, { keys: string[]; tokens: Record<string, string> }> = {};
  if (fs.existsSync(input.output)) {
    for (const name of fs.readdirSync(input.output).sort()) {
      const text = fs.readFileSync(path.join(input.output, name), "utf8");
      rawFiles[name] = text;
      const raw = text.match(/"unknown"\s*:\s*(\{[^}]*\})/u)?.[1];
      if (raw) {
        const keys: string[] = [],
          tokens: Record<string, string> = {};
        for (const match of raw.matchAll(
          /("(?:\\.|[^"\\])*")\s*:\s*(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/gu,
        )) {
          const key: unknown = JSON.parse(match[1]);
          assert.equal(typeof key, "string");
          keys.push(key as string);
          tokens[key as string] = match[2];
        }
        rawUnknownTokens[name] = { keys, tokens };
      }
    }
  }
  const processes = fs.existsSync(input.transcript)
    ? fs
        .readFileSync(input.transcript, "utf8")
        .trim()
        .split("\n")
        .filter(Boolean)
        .map((row): ProcessCapture => {
          const value = object(JSON.parse(row) as unknown);
          for (const key of ["cwd", "lang", "lcAll", "stdin", "stdout", "stderr"])
            assert.equal(typeof value[key], "string");
          assert.ok(Array.isArray(value.argv));
          assert.equal(typeof value.status, "number");
          return value as unknown as ProcessCapture;
        })
    : [];
  const captured = {
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
    processes,
    files,
    rawUnknownTokens,
  };
  for (const [name, text] of Object.entries(rawFiles))
    files[name] = name.endsWith(".json") ? parseReport(text, sourceVersion) : text;
  for (const process of processes) {
    assert.equal(process.stdin, "");
    assert.equal(process.lang, "C");
    assert.equal(process.lcAll, "C");
    assert.equal(process.cwd, input.directory);
  }
  return captured;
}
export type Result = ReturnType<typeof runReporter>;
function canonical(value: unknown, key = ""): string {
  if (typeof value === "bigint") return String(value);
  if (Array.isArray(value)) return `[${value.map((item) => canonical(item)).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((name) => `${JSON.stringify(name)}:${canonical(object(value)[name], name)}`)
      .join(",")}}`;
  }
  // serde retains the two ratio fields as f64 even when they are zero or one.
  if (["falsePositiveRatio", "falseNegativeRatio"].includes(key) && Number.isInteger(value))
    return `${String(value)}.0`;
  return JSON.stringify(value);
}
function digestPayload(divergence: Divergence) {
  return Object.fromEntries(
    Object.entries(divergence).filter(
      ([key]) => !["schema", "version", "project", "upstream", "sha256"].includes(key),
    ),
  );
}
export function assertDigest(divergence: Divergence) {
  const { schema, version, sha256 } = divergence;
  const payload = digestPayload(divergence);
  assert.equal(schema, "vize.fixtureLintDivergence");
  assert.equal(version, 1);
  assert.equal(sha256, hash(canonical(payload)), "complete independently derived digest preimage");
}
function hostSeams(result: Result, rust: boolean) {
  const rustc = rust ? spawnSync("rustc", ["--version"], { encoding: "utf8" }) : undefined;
  if (rustc) assert.equal(rustc.status, 0);
  for (const artifact of Object.values(result.files)) {
    if (typeof artifact !== "object") continue;
    assert.deepEqual(
      artifact.evidence.runtime,
      rust
        ? { name: "rust-script", version: rustc!.stdout.trim() }
        : { name: "node", version: process.version },
    );
    delete artifact.evidence.runtime;
    if ("baseline" in artifact) {
      assert.ok(typeof artifact.baseline.durationMs === "number");
      assert.ok(Number.isSafeInteger(artifact.baseline.durationMs));
      assert.ok(artifact.baseline.durationMs >= 0 && artifact.baseline.durationMs <= 240_000);
      delete artifact.baseline.durationMs;
    }
  }
}
function removeOriginalAdditions(artifact: ProjectReport) {
  const require = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
  const versions = Object.fromEntries(
    ["eslint", "eslint-plugin-vue", "vue-eslint-parser", "@typescript-eslint/parser"].map(
      (name) => [name, object(require(`${name}/package.json`) as unknown).version],
    ),
  );
  assert.deepEqual(
    artifact.baseline.runtimeVersions,
    artifact.baseline.comparedRuleCount ? versions : {},
  );
  assert.deepEqual(Object.keys(artifact.raw).sort(), [
    "eslintResults",
    "patinaFindings",
    "patinaResults",
  ]);
  assert.ok(Array.isArray(artifact.raw.eslintResults));
  assert.ok(Array.isArray(artifact.raw.patinaResults));
  assert.deepEqual(
    artifact.raw.patinaFindings,
    artifact.raw.patinaResults.flatMap((entry) =>
      entry.messages.map((finding) => ({ ...finding, file: entry.file })),
    ),
  );
  assert.deepEqual(artifact.divergence.referenceCoverageExpectations, []);
  assert.equal(artifact.divergence.summary.referenceCoverageIntegrityErrorCount, 0);
  assertDigest(artifact.divergence);
  Reflect.deleteProperty(artifact, "raw");
  delete artifact.baseline.runtimeVersions;
  Reflect.deleteProperty(artifact.divergence, "referenceCoverageExpectations");
  delete artifact.divergence.summary.referenceCoverageIntegrityErrorCount;
  artifact.divergence.sha256 = hash(canonical(digestPayload(artifact.divergence)));
}
export function assertParity(expected: Result, actual: Result, oracle: Oracle) {
  expected = structuredClone(expected);
  actual = structuredClone(actual);
  const oldHeader = `Usage: rust-script ${command}.rs [options]`;
  if (expected.stdout.startsWith(`${oldHeader}\n`)) {
    const [header, ...remaining] = actual.stdout.split("\n");
    assert.ok([oldHeader, `Usage: node ${command}.ts [options]`].includes(header));
    assert.deepEqual(remaining, expected.stdout.split("\n").slice(1));
    actual.stdout = expected.stdout;
  }
  hostSeams(expected, true);
  hostSeams(actual, false);
  if (oracle === "original") {
    assert.deepEqual(expected.rawUnknownTokens, {});
    actual.rawUnknownTokens = {};
  }
  if (oracle === "original" && actual.files["lint-divergence-summary.json"]) {
    for (const row of indexFile(actual).projects) {
      assert.equal(row.referenceCoverageIntegrityErrorCount, 0);
      delete row.referenceCoverageIntegrityErrorCount;
    }
  }
  for (const [file, artifact] of Object.entries(actual.files)) {
    if (!file.endsWith("-lint-divergence.json")) continue;
    assert.ok(
      typeof artifact === "object" &&
        artifact.schema === "vize.fixtureLintDivergenceRun" &&
        "raw" in artifact,
    );
    assertDigest(artifact.divergence);
    if (oracle === "original") {
      removeOriginalAdditions(artifact);
      assert.equal(artifact.divergence.summary.patinaOutsideBaselineSurfaceCount, 0);
      const markdownName = file.replace(/\.json$/u, ".md");
      let markdownText = markdown(actual, markdownName);
      const additions = [
        `Patina findings outside reference coverage: ${artifact.divergence.summary.patinaOutsideBaselineSurfaceCount}\n`,
        "Reference coverage integrity errors: 0\n",
        "### Patina findings outside reference coverage: none\n\n",
      ];
      for (const addition of additions) {
        assert.equal(
          markdownText.split(addition).length,
          2,
          `exact Markdown addition: ${addition}`,
        );
        markdownText = markdownText.replace(addition, "");
        actual.files[markdownName] = markdownText;
      }
    }
  }
  assert.deepEqual(actual, expected);
}
