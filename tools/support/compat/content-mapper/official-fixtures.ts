import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

export interface Original {
  issue: number;
  files: Record<string, string>;
  baselineCodes: number[];
  expectedCodes: number[];
  expectedTerms: string[];
  baselineTerms: string[];
  baselineStdout?: string;
  baselineStderr?: string;
  expectedStdout?: string;
  expectedStderr?: string;
  baselineStatus?: number;
  expectedStatus?: number;
}
export interface Case {
  name: string;
  original: number;
  files: Record<string, string>;
  codes: number[];
  terms?: string[];
  build?: boolean;
  stdout?: string;
  stderr?: string;
  status?: number;
}
export interface Fixture {
  typescript: string;
  vue: string;
  baselineVize: string;
  originals: Original[];
  controls: Case[];
}

export function loadOfficialFixture(file: string): Fixture {
  const fixture = JSON.parse(fs.readFileSync(file, "utf8")) as Fixture;
  assert.equal(fixture.typescript, "7.1.0-dev.20261017.1");
  assert.equal(fixture.vue, "3.5.20");
  assert.match(fixture.baselineVize, /^\d+\.\d+\.\d+$/u);
  assert.ok(Array.isArray(fixture.originals) && fixture.originals.length > 0);
  assert.ok(Array.isArray(fixture.controls));
  const issues = new Set(fixture.originals.map((row) => row.issue));
  assert.equal(issues.size, fixture.originals.length);
  for (const row of fixture.originals) {
    assert.ok(Number.isSafeInteger(row.issue) && row.issue > 0);
    for (const field of ["baselineCodes", "expectedCodes"] as const)
      assert.ok(Array.isArray(row[field]) && row[field].every(Number.isSafeInteger));
    for (const field of ["baselineTerms", "expectedTerms"] as const)
      assert.ok(
        Array.isArray(row[field]) && row[field].every((value) => typeof value === "string"),
      );
    for (const field of [
      "baselineStdout",
      "baselineStderr",
      "expectedStdout",
      "expectedStderr",
    ] as const)
      if (Object.hasOwn(row, field)) assert.equal(typeof row[field], "string", field);
    for (const field of ["baselineStatus", "expectedStatus"] as const)
      if (Object.hasOwn(row, field)) assert.ok(row[field] === 0 || row[field] === 2, field);
    if (row.baselineStatus !== undefined)
      assert.equal(row.baselineStatus, row.baselineCodes.length ? 2 : 0);
  }
  for (const row of officialCases(fixture)) {
    assert.match(row.name, /^[a-z0-9][a-z0-9-]*$/u);
    assert.ok(issues.has(row.original));
    assert.ok(Array.isArray(row.codes) && row.codes.every(Number.isSafeInteger));
    assert.ok(Object.hasOwn(row.files, "tsconfig.json"));
    for (const [name, content] of Object.entries(row.files)) {
      assert.ok(!path.isAbsolute(name) && !name.split(/[\\/]/u).includes(".."));
      assert.equal(typeof content, "string");
    }
    for (const field of ["stdout", "stderr"] as const)
      if (Object.hasOwn(row, field)) assert.equal(typeof row[field], "string", field);
    if (row.status !== undefined) assert.equal(row.status, row.codes.length ? 2 : 0);
  }
  return fixture;
}

export function officialCases(fixture: Fixture): Case[] {
  return [
    ...fixture.originals.map((row) => ({
      name: `issue-${row.issue}`,
      original: row.issue,
      files: structuredClone(row.files),
      codes: [...row.expectedCodes],
      terms: [...row.expectedTerms],
      ...(row.expectedStdout !== undefined ? { stdout: row.expectedStdout } : {}),
      ...(row.expectedStderr !== undefined ? { stderr: row.expectedStderr } : {}),
      ...(row.expectedStatus !== undefined ? { status: row.expectedStatus } : {}),
    })),
    ...structuredClone(fixture.controls),
  ];
}

export const officialDiagnostics = (stdout: string) =>
  [...stdout.matchAll(/^([^\r\n]+)\((\d+),(\d+)\): error TS(\d+): (.*)$/gmu)].map((match) => ({
    file: match[1],
    line: Number(match[2]),
    column: Number(match[3]),
    code: Number(match[4]),
    message: match[5],
  }));

interface Result {
  signal: string | null;
  error: unknown;
  status: number | null;
  stdout: { bytes: number };
  stderr: { bytes: number };
  stdoutText: string;
  stderrText: string;
}

export function assertOfficialDiagnostics(
  row: Case,
  original: Original,
  baseline: boolean,
  result: Result,
  diagnostics = officialDiagnostics(result.stdoutText),
) {
  const codes = baseline ? original.baselineCodes : row.codes;
  const status = baseline ? original.baselineStatus : row.status;
  assert.equal(result.signal, null, row.name);
  assert.equal(result.error, null, row.name);
  assert.equal(result.status, status ?? (codes.length ? 2 : 0), row.name);
  assert.equal(result.stderr.bytes, 0, row.name);
  assert.deepEqual(
    diagnostics.map((diagnostic) => diagnostic.code).sort((left, right) => left - right),
    [...codes].sort((left, right) => left - right),
    row.name,
  );
  const terms = baseline ? original.baselineTerms : (row.terms ?? []);
  for (const term of terms)
    assert.ok(
      diagnostics.some((diagnostic) => diagnostic.message.includes(term)),
      `${row.name} must retain the original diagnostic term ${term}`,
    );
  const stdout = baseline ? original.baselineStdout : row.stdout;
  const stderr = baseline ? original.baselineStderr : row.stderr;
  if (stdout !== undefined) assert.equal(result.stdoutText, stdout, `${row.name}: whole stdout`);
  if (stderr !== undefined) assert.equal(result.stderrText, stderr, `${row.name}: whole stderr`);
  if (!codes.length) assert.equal(result.stdout.bytes, 0, row.name);
  if (row.build)
    assert.deepEqual(
      diagnostics
        .map((diagnostic) => diagnostic.file)
        .sort((left, right) => left.localeCompare(right)),
      ["strict-css/App.vue", "strict-props/App.vue"],
    );
  if (!baseline)
    for (const diagnostic of diagnostics) {
      const source = row.files[diagnostic.file];
      assert.ok(source !== undefined, `diagnostic must name authored file: ${row.name}`);
      const line = source.split(/\r?\n/u)[diagnostic.line - 1];
      assert.ok(
        line !== undefined && diagnostic.column >= 1 && diagnostic.column <= line.length + 1,
        `diagnostic must map into authored source: ${row.name}`,
      );
    }
  return diagnostics;
}
