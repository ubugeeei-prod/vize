import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

type LintMessage = { message?: string; [key: string]: unknown };
type LintFile = { messages?: LintMessage[]; [key: string]: unknown };
type CorpusCase = {
  id: string;
  role: string;
  expectedDuplicateIds: string[];
  files: Record<string, string>;
};
type Corpus = {
  schema: string;
  publicVersion: string;
  issue: { body: string };
  originalFiles: Record<string, string>;
  cases: CorpusCase[];
};
type ProcessPacket = {
  command: string;
  cwd: string;
  args: string[];
  status: number | null;
  signal: string | null;
  error: { name: string; message: string } | null;
  stdout: string;
  stderr: string;
};
type SourceInput = { path: string; source: string; sha256: string };
type Observation = {
  id: string;
  role: string;
  expectedDuplicateIds: string[];
  inputs: SourceInput[];
  on?: ProcessPacket;
  off?: ProcessPacket;
  plain?: ProcessPacket;
  after?: { path: string; sha256: string }[];
  onBody?: LintFile[];
  offBody?: LintFile[];
  targetSummary?: LintMessage[];
  observationError?: string;
};
type Report = {
  schema: string;
  role: string;
  qualified: false;
  node: string;
  platform: string;
  arch: string;
  corpusSha256: string;
  issueBodySha256: string;
  publicVersion: string;
  cases: Observation[];
  failures: { id?: string; phase?: string; message: string }[];
  version?: ProcessPacket;
  provenance?: {
    cli: string;
    cliSha256: string;
    nativeVersion?: string;
    nativeBuildToken?: string;
    loadedImages: { file: string; sha256: string }[];
  };
};

const args = process.argv.slice(2);
const option = (name: string): string => {
  const index = args.indexOf(name);
  const value = index >= 0 ? args[index + 1] : undefined;
  assert(value, "Missing required argument " + name);
  return value;
};
const cli = path.resolve(option("--cli"));
const output = path.resolve(option("--output"));
const corpusPath = path.resolve(option("--corpus"));
const bytes = fs.readFileSync(corpusPath);
const corpus: Corpus = JSON.parse(bytes.toString("utf8"));
assert.equal(corpus.schema, "vize.cross-file.nuxt-exclusive-id.before.v1");
assert.equal(corpus.publicVersion, "0.601.0");
assert.equal(corpus.cases.length, 13);
const hash = (value: string | Uint8Array): string =>
  createHash("sha256").update(value).digest("hex");
const report: Report = {
  schema: "vize.cross-file.nuxt-exclusive-id.public-before.v1",
  role: "research-only-public-before; not remediation qualification",
  qualified: false,
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  corpusSha256: hash(bytes),
  issueBodySha256: hash(corpus.issue.body),
  publicVersion: corpus.publicVersion,
  cases: [],
  failures: [],
};
fs.mkdirSync(output, { recursive: true });
const save = () =>
  fs.writeFileSync(path.join(output, "before.json"), JSON.stringify(report, null, 2) + "\n");
const env: NodeJS.ProcessEnv = { ...process.env, NO_COLOR: "1", VIZE_LOG: "warn" };
delete env.FORCE_COLOR;
const run = (argv: string[], cwd: string): ProcessPacket => {
  const result = spawnSync(cli, argv, {
    cwd,
    env,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  return {
    command: cli,
    cwd,
    args: argv,
    status: result.status,
    signal: result.signal,
    error: result.error ? { name: result.error.name, message: result.error.message } : null,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
};
const targetMessages = (body: LintFile[]): LintMessage[] =>
  body
    .flatMap((file) => file.messages ?? [])
    .filter((message) => message.message?.includes("vize:croquis/cf/duplicate-id"));
try {
  const realCli = fs.realpathSync(cli);
  const require = createRequire(realCli);
  const native: { getNativeBuildToken?: () => string } = require("@vizejs/native");
  const nativeEntry = require.resolve("@vizejs/native");
  const nativeManifest: { version?: string } = JSON.parse(
    fs.readFileSync(path.join(path.dirname(nativeEntry), "package.json"), "utf8"),
  );
  const images = Object.keys(require.cache).filter((file) => file.endsWith(".node"));
  report.provenance = {
    cli: realCli,
    cliSha256: hash(fs.readFileSync(realCli)),
    nativeVersion: nativeManifest.version,
    nativeBuildToken: native.getNativeBuildToken?.(),
    loadedImages: images.map((file) => ({ file, sha256: hash(fs.readFileSync(file)) })),
  };
  report.version = run(["--version"], output);
  save();
  assert.equal(nativeManifest.version, corpus.publicVersion);
  assert.equal(images.length, 1, "Observe the actual public native image");
  assert(typeof report.provenance.nativeBuildToken === "string");
  assert.match(report.provenance.nativeBuildToken, /^[a-f0-9]{32}$/);
  assert(report.version.stdout.includes(corpus.publicVersion));
  const originalBlocks = [...corpus.issue.body.matchAll(/```(?:ts|vue)\n([\s\S]*?)\n```/g)].map(
    (match) => match[1] + "\n",
  );
  assert.deepEqual(Object.values(corpus.originalFiles), originalBlocks);
  for (const entry of corpus.cases) {
    const work = path.join(output, "apps", entry.id);
    const row: Observation = {
      id: entry.id,
      role: entry.role,
      expectedDuplicateIds: entry.expectedDuplicateIds,
      inputs: [],
    };
    report.cases.push(row);
    for (const [name, source] of Object.entries(entry.files)) {
      assert(!path.isAbsolute(name) && !name.split("/").includes(".."));
      const file = path.join(work, name);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, source);
      row.inputs.push({ path: name, source, sha256: hash(source) });
    }
    const flags = [
      "lint",
      "--no-config",
      "--locale",
      "en",
      "--help-level",
      "none",
      "--format",
      "json",
    ];
    row.on = run([...flags, "--cross-file", "src/**/*.vue"], work);
    row.off = run([...flags, "src/**/*.vue"], work);
    row.plain = run(["lint", "-f", "plain", "--no-config", "--cross-file", "src/**/*.vue"], work);
    row.after = row.inputs.map((input) => ({
      path: input.path,
      sha256: hash(fs.readFileSync(path.join(work, input.path))),
    }));
    save();
    try {
      for (const process of [row.on, row.off, row.plain]) {
        assert.equal(process.error, null);
        assert.equal(process.signal, null);
        assert([0, 1].includes(process.status ?? -1));
      }
      row.onBody = JSON.parse(row.on.stdout) as LintFile[];
      row.offBody = JSON.parse(row.off.stdout) as LintFile[];
      assert(Array.isArray(row.onBody) && Array.isArray(row.offBody));
      const targets = targetMessages(row.onBody);
      row.targetSummary = targets;
      for (const id of entry.expectedDuplicateIds)
        assert(
          targets.some((message) => message.message?.includes("Element ID '" + id + "'")),
          entry.id + ": required existing duplicate control absent: " + id,
        );
      assert.equal(targetMessages(row.offBody).length, 0);
      assert.deepEqual(
        row.after,
        row.inputs.map(({ path, sha256 }) => ({ path, sha256 })),
      );
    } catch (error) {
      row.observationError = error instanceof Error ? error.message : String(error);
      report.failures.push({
        id: entry.id,
        message: error instanceof Error ? error.message : String(error),
      });
    }
    save();
  }
} catch (error) {
  report.failures.push({
    phase: "capture",
    message: error instanceof Error ? error.message : String(error),
  });
} finally {
  save();
}
console.log(
  JSON.stringify({
    role: report.role,
    qualified: false,
    cases: report.cases.length,
    failures: report.failures,
  }),
);
if (report.failures.length) process.exitCode = 1;
