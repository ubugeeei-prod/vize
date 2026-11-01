import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { isDeepStrictEqual } from "node:util";
import { asArray, asRecord } from "../../../support/json-value.ts";
import {
  cases,
  expected,
  repaired,
  sha256,
  sources,
  type Role,
  type SourceFile,
} from "./contract.ts";

export type Recorder = { root: string; json: (name: string, value: unknown) => void };
export function recorder(root: string): Recorder {
  return {
    root,
    json: (name, value) => writeFileSync(join(root, name), JSON.stringify(value, null, 2) + "\n"),
  };
}
export function childEnvironment(environment = process.env): NodeJS.ProcessEnv {
  return Object.fromEntries(
    Object.entries(environment).filter(
      ([name]) => !/(?:TOKEN|SECRET|PASSWORD|CREDENTIAL)/iu.test(name),
    ),
  );
}
export function capture(record: Recorder, label: string, argv: string[], cwd: string) {
  const start = new Date().toISOString();
  const result = spawnSync(argv[0]!, argv.slice(1), {
    cwd,
    env: childEnvironment(),
    timeout: 600_000,
    maxBuffer: 64 * 1024 * 1024,
    encoding: "buffer",
    stdio: ["ignore", "pipe", "pipe"],
  });
  return retain(record, label, argv, cwd, start, result);
}
export function retain(
  record: Recorder,
  label: string,
  argv: string[],
  cwd: string,
  start: string,
  result: {
    pid?: number;
    status: number | null;
    signal: string | null;
    error?: Error;
    stdout: Buffer | null;
    stderr: Buffer | null;
  },
) {
  const decode = (raw: Buffer | null) => {
    if (raw === null) return { text: null, utf8Error: null };
    try {
      return { text: new TextDecoder("utf-8", { fatal: true }).decode(raw), utf8Error: null };
    } catch (error) {
      return { text: null, utf8Error: String(error) };
    }
  };
  if (result.stdout !== null)
    writeFileSync(join(record.root, label + ".stdout.raw"), result.stdout);
  if (result.stderr !== null)
    writeFileSync(join(record.root, label + ".stderr.raw"), result.stderr);
  const out = decode(result.stdout),
    err = decode(result.stderr);
  const packet = {
    label,
    argv,
    cwd,
    start,
    end: new Date().toISOString(),
    pid: result.pid ?? null,
    selectedCliPid: null,
    selectedBackendPid: null,
    status: result.status,
    signal: result.signal,
    error: result.error ? { name: result.error.name, message: result.error.message } : null,
    stdout: out.text,
    stderr: err.text,
    stdoutUtf8Error: out.utf8Error,
    stderrUtf8Error: err.utf8Error,
    stdoutImage:
      result.stdout === null
        ? null
        : { bytes: result.stdout.length, sha256: sha256(result.stdout) },
    stderrImage:
      result.stderr === null
        ? null
        : { bytes: result.stderr.length, sha256: sha256(result.stderr) },
    pidScope: "spawned npm/npx companion PID only; original CLI/backend selection unobserved",
  };
  record.json(label + ".process.json", packet);
  return packet;
}
export type Packet = ReturnType<typeof retain>;
export function healthy(packet: Packet, status: number) {
  assert.equal(packet.error, null, packet.label);
  assert.equal(packet.signal, null, packet.label);
  assert.equal(packet.stdoutUtf8Error, null);
  assert.equal(packet.stderrUtf8Error, null);
  assert.equal(packet.status, status, packet.label);
  assert.ok(Number.isSafeInteger(packet.pid) && Number(packet.pid) > 0);
}
function writeSources(root: string, files: SourceFile[]) {
  rmSync(join(root, "src/Importer.vue"), { force: true });
  for (const file of files) {
    const path = join(root, file.path);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, file.source);
  }
}
export function captureInputs(root: string, files: SourceFile[]) {
  return files.map((file) => {
    const path = join(root, file.path),
      raw = readFileSync(path);
    return {
      path: file.path,
      absolutePath: path,
      source: new TextDecoder("utf-8", { fatal: true }).decode(raw),
      bytes: raw.length,
      sha256: sha256(raw),
    };
  });
}
export function parsePacket(packet: Packet): unknown {
  return JSON.parse(packet.stdout ?? "");
}
export function expectedProgress(cwd: string, files: number) {
  assert.ok(files === 3 || files === 4);
  return (
    "Building Corsa virtual project for " +
    files +
    " files under " +
    cwd +
    "...\nRunning Corsa diagnostics for " +
    files +
    " files...\n"
  );
}
export function assessJson(packet: Packet, correct: unknown, accept: boolean) {
  const parsed: unknown = parsePacket(packet),
    value = asRecord(parsed);
  const count = value.errorCount;
  assert.ok(count === 0 || count === 1, "unexpected errors or degraded result");
  healthy(packet, count);
  assert.equal(
    packet.stderr,
    expectedProgress(packet.cwd, Number(asRecord(correct).fileCount)),
    "whole normal progress must match the source renderer; warnings/backend extras refuse",
  );
  assert.equal(value.warningCount, 0);
  assert.ok(asArray(value.files).length > 0);
  const matchesAuthoredLaw = isDeepStrictEqual(parsed, correct);
  if (accept) assert.deepEqual(parsed, correct, "entire public JSON must satisfy the authored law");
  return { parsed, matchesAuthoredLaw };
}
export function execute(
  root: string,
  role: Role,
  npx: string,
  original: SourceFile[],
  record: Recorder,
  verifyRoute: () => void,
) {
  const attempt = (label: string, argv: string[]) => {
    try {
      verifyRoute();
    } catch (error) {
      return retain(record, label, argv, root, new Date().toISOString(), {
        pid: undefined,
        status: null,
        signal: null,
        stdout: null,
        stderr: null,
        error: error instanceof Error ? error : new Error(String(error)),
      });
    }
    return capture(record, label, argv, root);
  };
  const initial = sources(original, role, "Original");
  writeSources(root, initial);
  const textBefore = captureInputs(root, initial);
  record.json("original-text.inputs-before.json", textBefore);
  const text = attempt("original-text", [process.execPath, npx, "vize", "check"]);
  const textAfter = captureInputs(root, initial);
  record.json("original-text.inputs-after.json", textAfter);
  const outcomes = cases.map((id) => {
    const files = sources(original, role, id);
    writeSources(root, files);
    const before = captureInputs(root, files);
    record.json(id + ".inputs-before.json", before);
    const packet = attempt(id, [process.execPath, npx, "vize", "check", "-f", "json"]);
    const after = captureInputs(root, files);
    record.json(id + ".inputs-after.json", after);
    let parsed: unknown = null,
      parseError: string | null = null;
    try {
      parsed = parsePacket(packet);
    } catch (error) {
      parseError = String(error);
    }
    return {
      id,
      files,
      before,
      after,
      packet,
      parsed,
      parseError,
      expected: expected(original, id),
      expectedStatus: repaired(id) ? 0 : 1,
    };
  });
  // All ten complete process attempts, original bytes and parse errors precede every verdict.
  record.json("complete-observations.json", {
    role,
    text: { before: textBefore, after: textAfter, packet: text },
    outcomes,
  });
  return { textBefore, textAfter, text, outcomes };
}

export function assess(role: Role, observation: ReturnType<typeof execute>, record: Recorder) {
  const { textBefore, textAfter, text, outcomes } = observation;
  assert.deepEqual(textAfter, textBefore);
  const assessments = outcomes.map((row) => {
    assert.deepEqual(row.after, row.before, "authored input mutated during the command");
    return { id: row.id, ...assessJson(row.packet, row.expected, role === "after") };
  });
  const originalResult = assessments[0]!;
  const textErrors = asRecord(originalResult.parsed).errorCount;
  assert.ok(textErrors === 0 || textErrors === 1);
  healthy(text, textErrors);
  assert.equal(
    text.stderr,
    expectedProgress(text.cwd, 3),
    "whole original progress must match; warnings/backend extras refuse",
  );
  assert.ok(text.stdout !== null);
  assert.ok(!/warn|fallback|degrad|failed|panic|timeout/iu.test(text.stdout));
  if (role === "after") {
    assert.equal((text.stdout.match(/TS2322/gu) ?? []).length, 1);
    assert.ok(text.stdout.includes("src/plain.ts"));
    assert.ok(text.stdout.includes("Type 'string' is not assignable to type 'number'."));
    assert.ok(/5[: ,]+7|5:7/u.test(text.stdout), "authored line/column must remain visible");
  }
  record.json("semantic-assessment.json", {
    role,
    assessments,
    originalText: text,
    scope:
      role === "before"
        ? "complete installed baseline witness; no correctness acceptance"
        : "original text plus all nine whole authored JSON laws",
    publicAcceptance: false,
    semanticMatch: role === "after",
    wsl2Acceptance: false,
    selectedOriginalCliPid: null,
    selectedOriginalBackendPid: null,
  });
  return assessments;
}

export function assessCaptured(
  rows: { role: Role; outcomes: ReturnType<typeof execute>; owned: Recorder }[],
  record: Recorder,
) {
  const verdicts = rows.map((row) => {
    try {
      return { role: row.role, value: assess(row.role, row.outcomes, row.owned), error: null };
    } catch (error) {
      return {
        role: row.role,
        value: null,
        error: error instanceof Error ? error : new Error(String(error)),
      };
    }
  });
  record.json(
    "all-semantic-assessments.json",
    verdicts.map((row) => ({
      role: row.role,
      value: row.value,
      error: row.error ? { name: row.error.name, message: row.error.message } : null,
      publicAcceptance: false,
    })),
  );
  const failed = verdicts.find((row) => row.error !== null);
  if (failed) throw failed.error;
  return verdicts.map((row) => {
    assert.ok(row.value);
    return row.value;
  });
}
