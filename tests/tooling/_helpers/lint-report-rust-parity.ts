import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  root,
  command,
  referenceFixtures,
  object,
  parseReport,
  type Oracle,
  type Report,
  type ReportSource,
} from "./lint-report-schema.ts";
export * from "./lint-report-schema.ts";
const sources = { original: referenceFixtures.original, updated: referenceFixtures.updated };
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
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
export function fixture() {
  fs.mkdirSync(path.join(root, "target"), { recursive: true });
  const directory = fs.mkdtempSync(path.join(root, "target", "lint-report-parity-"));
  const output = path.join(directory, "output");
  const transcript = path.join(directory, "processes.jsonl");
  const response = path.join(directory, "response.json");
  const binary = path.join(directory, "fake-vize.mjs");
  const registry = path.join(directory, "registry.json");
  fs.writeFileSync(binary, referenceFixtures.fakeVizeSource);
  fs.chmodSync(binary, 0o755);
  const project = (id = "parity", extra: Record<string, unknown> = {}) => ({
    id,
    revision: "0".repeat(40),
    fixturePath: path.relative(root, directory),
    vueGlobs: ["App.vue"],
    coverage: ["linter"],
    ...extra,
  });
  const write = (file: string, contents: string) => {
    fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    fs.writeFileSync(path.join(directory, file), contents);
  };
  write("App.vue", "<template><div /></template>\n");
  const configure = (
    projects = [project()],
    value: Record<string, unknown> = { results: [{ file: "App.vue", messages: [] }] },
  ) => {
    fs.writeFileSync(registry, JSON.stringify({ projects }));
    fs.writeFileSync(response, JSON.stringify(value));
  };
  configure();
  return {
    directory,
    output,
    transcript,
    binary,
    project,
    write,
    configure,
    args: [
      "--registry",
      registry,
      "--output-dir",
      output,
      "--vize-bin",
      binary,
      "--timeout-ms",
      "30000",
    ],
    clean: () => fs.rmSync(directory, { recursive: true, force: true }),
  };
}
export type Fixture = ReturnType<typeof fixture>;
export function evidenceDirectory() {
  const directory =
    process.env.VIZE_LINT_REPORT_PARITY_DIR ??
    path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "lint-report-parity");
  fs.mkdirSync(directory, { recursive: true });
  return directory;
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
    path.join(
      evidenceDirectory(),
      `${String(++measurement).padStart(3, "0")}-${randomUUID()}-raw.json`,
    ),
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
