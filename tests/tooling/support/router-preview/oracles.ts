import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fixtureDirectory, hash, relativeMember, type FileIdentity } from "./authority.ts";
import type { Observation } from "./processes.ts";

type OracleAuthority = {
  schema: string;
  version: number;
  provenance: {
    runId: string;
    headRevision: string;
    upstreamRevision: string;
    candidatePacketsAreOracles: boolean;
  };
  scope: {
    controls: number;
    processes: number;
    checkedHosts: string[];
    automaticProjection: string;
    mapper: string;
    lsp: string;
  };
  files: FileIdentity[];
};

export type FrozenControl = { id: string; source: string; invalid: boolean };
export type ProcessPacket = Pick<
  Observation,
  "status" | "signal" | "error" | "stdout" | "stderr" | "stdoutBase64" | "stderrBase64"
>;

export function readOracles(directory = fixtureDirectory) {
  const bytes = fs.readFileSync(path.join(directory, "oracles.json"));
  const authority = JSON.parse(bytes.toString("utf8")) as OracleAuthority;
  assert.equal(authority.schema, "vize.router-preview-independent-oracles");
  assert.equal(authority.version, 1);
  assert.equal(authority.provenance.runId, "40175166770");
  assert.equal(authority.provenance.headRevision, "cb2381db730ff1bf838a0fc1850d7740ee19e041");
  assert.equal(authority.provenance.upstreamRevision, "593d7ad9ff99e95c24be28e962717df0fb2be763");
  assert.equal(authority.provenance.candidatePacketsAreOracles, false);
  assert.deepEqual(authority.scope.checkedHosts, ["check"]);
  assert.equal(authority.scope.automaticProjection, "not-qualified");
  assert.equal(authority.scope.mapper, "not-qualified");
  assert.equal(authority.scope.lsp, "not-qualified");
  const files = new Map<string, Buffer>();
  for (const expected of authority.files) {
    const relative = relativeMember(expected.path);
    assert.equal(files.has(relative), false, relative);
    const bytes = fs.readFileSync(path.join(directory, relative));
    assert.equal(bytes.length, expected.bytes, relative);
    assert.equal(hash(bytes), expected.sha256, relative);
    files.set(relative, bytes);
  }
  const json = <T>(name: string): T => {
    const bytes = files.get(`frozen/${name}`);
    assert.ok(bytes, `frozen authority must declare ${name}`);
    return JSON.parse(bytes.toString("utf8")) as T;
  };
  const controls = json<FrozenControl[]>("controls.json");
  const processes = json<Observation[]>("independent-processes.json");
  assert.equal(controls.length, authority.scope.controls);
  assert.equal(controls.length, 20);
  assert.equal(processes.length, controls.length * 2);
  assert.equal(authority.scope.processes, controls.length * 3);
  const ids = controls.map(({ id }) => id);
  assert.equal(new Set(ids).size, ids.length);
  const independent = new Map(processes.map((record) => [record.label, record]));
  assert.equal(independent.size, processes.length);
  for (const control of controls) {
    for (const host of ["reference", "public-control"]) {
      const record = independent.get(`${control.id}/${host}`);
      assert.ok(record, `complete independent process required: ${control.id}/${host}`);
      packet(record);
      assert.equal(record.signal, null, record.label);
      assert.equal(record.error, null, record.label);
      assert.equal(record.status, control.invalid ? (host === "reference" ? 2 : 1) : 0);
    }
  }
  return { authority, sha256: hash(bytes), files, json, controls, independent };
}

export function packet(record: ProcessPacket): ProcessPacket {
  // Keep byte identity, including invalid UTF-8, independent of the readable view.
  assert.equal(Buffer.from(record.stdoutBase64, "base64").toString("utf8"), record.stdout);
  assert.equal(Buffer.from(record.stderrBase64, "base64").toString("utf8"), record.stderr);
  return {
    status: record.status,
    signal: record.signal,
    error: record.error,
    stdout: record.stdout,
    stderr: record.stderr,
    stdoutBase64: record.stdoutBase64,
    stderrBase64: record.stderrBase64,
  };
}

export function nativeOracle(record: Observation, actualRoot: string): ProcessPacket {
  const expected = packet(record);
  const report = JSON.parse(expected.stdout) as {
    programs: Array<{ compilerOptions: { rootDir: string } }>;
  };
  assert.equal(report.programs.length, 1);
  assert.equal(report.programs[0].compilerOptions.rootDir, record.cwd);
  // Only this protocol path field can change. Authored diagnostic text, file
  // membership, compiler options and their order remain byte-for-byte exact.
  assert.equal(`${JSON.stringify(report, null, 2)}\n`, expected.stdout);
  assert.equal(Buffer.from(expected.stdout, "utf8").toString("base64"), expected.stdoutBase64);
  report.programs[0].compilerOptions.rootDir = actualRoot;
  const stdout = `${JSON.stringify(report, null, 2)}\n`;
  return { ...expected, stdout, stdoutBase64: Buffer.from(stdout).toString("base64") };
}

export function verifyControl(
  oracles: ReturnType<typeof readOracles>,
  id: string,
  source: string,
  actual: { reference: Observation; publicControl: Observation; native: Observation },
  root: string,
) {
  const control = oracles.controls.find((entry) => entry.id === id);
  assert.ok(control, `undeclared control: ${id}`);
  assert.equal(source, control.source, `complete authored input: ${id}`);
  const reference = oracles.independent.get(`${id}/reference`)!;
  const native = oracles.independent.get(`${id}/public-control`)!;
  assert.deepEqual(packet(actual.reference), packet(reference), `${id}: complete reference packet`);
  const expected = nativeOracle(native, root);
  assert.deepEqual(packet(actual.publicControl), expected, `${id}: released manual control packet`);
  assert.deepEqual(packet(actual.native), expected, `${id}: receipted source packet`);
}
