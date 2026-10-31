import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, readJsonRecord } from "../../json-value.ts";
import { sha256, type DeclarationGraph } from "../computed_event_custody.ts";

export const profiles = {
  before: {
    version: "0.620.0",
    source: "e361d3873712eb11db018827713b1434be898261",
    candidate: "5695be444d8e5d889d2337fc8e22a974458b93ea",
    run: "41216373606",
    declarations: 9,
    edges: 9,
  },
  head: {
    version: "0.621.0",
    source: "3279664693067173a8da325f189dca917557fec5",
    candidate: "41cbfb604e9ee10015390f278afa0775e53ae368",
    run: "41219027171",
    declarations: 10,
    edges: 13,
  },
} as const;
export type Role = keyof typeof profiles;
export type Mode = "esm" | "commonjs" | "direct";
export const modes = ["esm", "commonjs", "direct"] as const;
export const referencePath = fileURLToPath(new URL("./source-reference.json.txt", import.meta.url));
export function reference() {
  const raw = readFileSync(referencePath);
  assert.equal(sha256(raw), "d7c3bbef80e3d79522f13a387480e99736bf92d149e134e777c6deb3d709ea2e");
  const value = readJsonRecord(referencePath);
  assert.equal(
    value.sourceReceiptSha256,
    "e14555e1a9f5758d0a29582a35c25d3f9d0ca118c9c2beddde8d810b41b20da5",
  );
  assert.equal(asArray(value.sourceCells).length, 6);
  assert.equal(value.sourceModeRowsAndExportsWhollyEqual, true);
  return value;
}
export function expected(role: Role, mode: Mode) {
  const selected = asRecord(asRecord(reference().roles)[role]);
  return { ...selected, ...asRecord(asRecord(selected.modes)[mode]) };
}
export function assertGraph(role: Role, root: string, graph: DeclarationGraph) {
  const sdk = asArray(expected(role, "esm").sdk).map((value) => asRecord(value));
  assert.equal(graph.files.length, profiles[role].declarations);
  assert.equal(graph.edges.length, profiles[role].edges);
  assert.deepEqual(
    graph.files
      .map((file) => ({
        file: file.path.slice(root.length + 1),
        bytes: file.bytes,
        sha256: file.sha256,
      }))
      .sort((a, b) => a.file.localeCompare(b.file)),
    sdk
      .map((file) => ({ file: file.file, bytes: file.bytes, sha256: file.sha256 }))
      .sort((a, b) => String(a.file).localeCompare(String(b.file))),
    "whole installed declaration inventory belongs to this exact release role",
  );
  const graphPath = fileURLToPath(new URL("./declaration-edges.json.txt", import.meta.url));
  assert.equal(
    sha256(readFileSync(graphPath)),
    "22f282468418c4f0a98aacc332e7beaee7309db5bd29c0383a63a088c1c2aa8b",
  );
  assert.deepEqual(
    graph.edges.map((edge) => ({
      ...edge,
      from: edge.from.slice(root.length + 1),
      target: edge.target.slice(root.length + 1),
    })),
    readJsonRecord(graphPath)[role],
    "complete ordered edge kinds, specifiers, import attributes and physical targets",
  );
}
export function assertRows(
  role: Role,
  mode: Mode,
  query: { rows: unknown[]; exports: unknown[] },
  preservation: boolean,
) {
  const cell = expected(role, mode);
  assert.deepEqual(query.rows, preservation ? cell.preservationRows : cell.rows);
  if (!preservation) {
    const keyed = (items: unknown[]) =>
      Object.fromEntries(
        items.map((value) => {
          const row = asRecord(value);
          return [String(row.name), row];
        }),
      );
    assert.equal(query.exports.length, 75);
    assert.equal(new Set(query.exports.map((value) => asRecord(value).name)).size, 75);
    assert.deepEqual(
      keyed(query.exports),
      keyed(asArray(cell.exports)),
      "whole former implicit and explicit export surface",
    );
  }
}
export function assertProcess(
  actual: {
    status: number | null;
    signal: string | null;
    error: string | null;
    stdout: string;
    stderr: string;
  },
  stdout: string,
) {
  assert.deepEqual(
    {
      status: actual.status,
      signal: actual.signal,
      error: actual.error,
      stdout: actual.stdout,
      stderr: actual.stderr,
    },
    { status: stdout ? 2 : 0, signal: null, error: null, stdout, stderr: "" },
    "complete compiler outcome, without diagnostic or path normalization",
  );
}
