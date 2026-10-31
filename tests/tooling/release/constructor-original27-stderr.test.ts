import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import test from "node:test";
import type { CliResult } from "../../../tools/benchmarks/scripts/canon-instance-directives-model.ts";
import { asRecord, asString, readJsonRecord } from "../../../tools/support/json-value.ts";
import { sha256 } from "../../../tools/support/config/windows_consumer/common.ts";
import { hook } from "../../../tools/support/release/editor_vite_stub/custody.ts";
import {
  sharedPreamble,
  verifyCliDebug,
  verifyNegativeCliJournal,
} from "../../../tools/support/release/constructor_original27_public/cli-output.ts";

const fixture = readJsonRecord(
  new URL(
    "../../../tools/support/release/constructor_original27_public/cli-shared-preamble-0614.json",
    import.meta.url,
  ),
);
const helper = Buffer.from(asString(fixture.rawBase64), "base64");
const project = "/public/original27";
// Per-file payloads below are synthetic refusal controls, not product evidence.
const files = () =>
  Array.from({ length: 54 }, (_, index) => ({
    file: `case-${String(index).padStart(2, "0")}.ts`,
    diagnostics: [],
    virtualTs: `// exact ${index}\n`,
  }));
const result = (): CliResult => ({ files: files(), fileCount: 54, errorCount: 14 });
function render(value = result(), body = helper) {
  return Buffer.concat([
    Buffer.from(
      `Building Corsa virtual project for 54 files under ${project}...\nRunning Corsa diagnostics for 54 files...\n\n=== __vize_helpers.d.ts ===\n`,
    ),
    body,
    Buffer.from("\n"),
    ...value.files.map((row) =>
      Buffer.from(`\n=== ${resolve(project, row.file)} ===\n${asString(row.virtualTs)}\n`),
    ),
  ]);
}
test("the exact source-pinned shared helper and all54 declared debug bodies are consumed", () => {
  assert.equal(fixture.selectedSource, "815972e433217e45672735bdc38d7e737c055e41");
  assert.equal(fixture.definitionGitBlob, "3bc47842bd494fb9ee3f98ef0e1e8c318bea251a");
  assert.equal(helper.length, sharedPreamble.bytes);
  assert.equal(sha256(helper), sharedPreamble.sha256);
  assert.equal(verifyCliDebug(render(), result(), project).exactNoRemainder, true);
});
test("extra, missing, changed-body, helper, order and duplicate debug variations refuse", () => {
  const value = result(),
    raw = render(value);
  for (const changed of [
    Buffer.concat([Buffer.from("unknown\n"), raw]),
    Buffer.concat([raw, Buffer.from("unknown\n")]),
    raw.subarray(0, -1),
    Buffer.from(raw.toString("utf8").replace("// exact 0\n", "// tampered\n")),
    Buffer.from(
      raw
        .toString("utf8")
        .replace("Running Corsa diagnostics for 54", "Running Corsa diagnostics for 53"),
    ),
  ]) {
    assert.ok(!changed.equals(raw), "each refusal control must change real bytes");
    assert.throws(() => verifyCliDebug(changed, value, project));
  }
  const changedHelper = Buffer.from(helper);
  changedHelper[10] ^= 1;
  assert.throws(() => verifyCliDebug(render(value, changedHelper), value, project));
  const changed = result();
  changed.files[0].virtualTs = asString(changed.files[0].virtualTs) + " ";
  assert.throws(() => verifyCliDebug(raw, changed, project));
  assert.throws(() => verifyCliDebug(raw, { ...value, files: value.files.slice(1) }, project));
  assert.throws(() =>
    verifyCliDebug(raw, { ...value, files: [...value.files.slice(1), value.files[1]] }, project),
  );
  const reordered = result();
  [reordered.files[0], reordered.files[1]] = [reordered.files[1], reordered.files[0]];
  assert.throws(() => verifyCliDebug(raw, reordered, project));
});

function journalFixture(body: (file: string) => void) {
  const root = mkdtempSync(join(tmpdir(), "original27-native-exit-"));
  try {
    body(join(root, "native.ndjson"));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
const expected = {
  root: "/registry",
  cli: "/registry/node_modules/vize/bin/vize",
  binary: { path: "/registry/native.node", bytes: 7, sha256: "a".repeat(64) },
  token: "b".repeat(32),
};
const child = {
  pid: 4321,
  argv: [process.execPath, "--require", hook, expected.cli, "check", "--show-virtual-ts"],
  status: 1,
  signal: null,
  error: null,
};
function events() {
  const base = { schema: "vize-public-native-custody-event-v1", pid: child.pid };
  return [
    {
      ...base,
      event: "initialized",
      installRoot: expected.root,
      nativePath: expected.binary.path,
      sha256: expected.binary.sha256,
      node: process.execPath,
      argv: [child.argv[0], ...child.argv.slice(3)],
    },
    ...["attempt", "returned"].map((event) => ({
      ...base,
      event,
      actualPath: expected.binary.path,
      sha256: expected.binary.sha256,
      expectedNative: true,
      corsaPath: "",
    })),
  ];
}
const save = (file: string, rows: unknown[]) =>
  writeFileSync(file, rows.map((row) => JSON.stringify(row)).join("\n") + "\n");
// Synthetic records exercise the real journal schema and independent PID join.
test("native Rust exit1 requires exact3 journal events and the captured childPID join", () =>
  journalFixture((file) => {
    save(file, events());
    const proof = verifyNegativeCliJournal(file, expected, child);
    assert.equal(proof.capturedChildExitStatus, 1);
    assert.equal(proof.NodeExitCallbackObserved, false);
    assert.ok(!readFileSync(file, "utf8").includes("token"));
  }));
test("negative native exit rejects PID, status, signal, error, argv and event/schema drift", () =>
  journalFixture((file) => {
    const original = events();
    save(file, original);
    for (const changed of [
      { ...child, pid: 4322 },
      { ...child, status: 0 },
      { ...child, signal: "SIGTERM" as const },
      { ...child, error: "failed" },
      { ...child, argv: [...child.argv, "unexpected"] },
    ])
      assert.throws(() => verifyNegativeCliJournal(file, expected, changed));
    const wrongHash = events();
    asRecord(wrongHash[1]).sha256 = "c".repeat(64);
    const reordered = events();
    [reordered[1], reordered[2]] = [reordered[2], reordered[1]];
    const tokenAdded = events();
    asRecord(tokenAdded[0]).token = expected.token;
    for (const changed of [
      wrongHash,
      reordered,
      tokenAdded,
      original.slice(0, -1),
      [
        ...original,
        { schema: "vize-public-native-custody-event-v1", pid: child.pid, event: "exit", code: 1 },
      ],
    ]) {
      save(file, changed);
      assert.throws(() => verifyNegativeCliJournal(file, expected, child));
    }
  }));
