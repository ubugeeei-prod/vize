import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gzipSync, gunzipSync } from "node:zlib";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { validateLspReport } from "../differential/lsp-report.ts";
import {
  LOCAL_RESOLVE_SESSION_ID,
  readLocalResolveSourceSession,
  sourceSessionUnsupportedLegacy,
  validateLocalResolveSourceSession,
} from "../differential/lsp-source-sessions.ts";
import { referenceObservation, referenceReport } from "./support/lsp/differential-reference.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const fixture = readLocalResolveSourceSession(root);
const plan = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/lsp/local-completion-resolve-10592/session-plan.json",
    ),
    "utf8",
  ),
);
const loaded = loadLspManifest(path.join(root, "tests/_fixtures/differential/lsp/manifest.json"));
const build = {
  sourceRevision: loaded.manifest.baseRevision,
  binaryPath: "target/ci/vize",
  binarySha256: "0".repeat(64),
  cliVersion: "vize 0.429.0-dev",
};

test("the registered transition pins the actual public failure prefix and all source executor inputs", () => {
  const registered = loaded.cases.find(({ id }) => id === LOCAL_RESOLVE_SESSION_ID)!;
  assert.ok(registered);
  assert.equal(registered.requestCount, 0);
  assert.throws(() => referenceObservation(registered), /cannot use synthetic/);
  assert.deepEqual(registered.requests, []);
  assert.deepEqual(registered.data, fixture.data);
  assert.equal(registered.files.length, 15);
  assert.deepEqual(
    registered.files.map(({ runtimePath, bytes }) => ({ runtimePath, bytes })),
    [...fixture.files].map(([runtimePath, bytes]) => ({ runtimePath, bytes })),
  );
  assert.equal(fixture.initialFiles.length, 8);
  assert.equal(fixture.spec.plannedClientRpcCount, 23);
  assert.equal(fixture.spec.directWrapperRpcCount, 22);
});

test("only the closed transition profile can have an empty generic request list", () => {
  for (const mutate of [
    (value: typeof fixture.data) => {
      value.id += "-other";
    },
    (value: typeof fixture.data) => {
      value.method = "textDocument/hover";
    },
    (value: typeof fixture.data) => {
      delete value.sourceSession;
    },
    (value: typeof fixture.data) => {
      value.sourceSession!.plannedClientRpcCount = 22 as 23;
    },
    (value: typeof fixture.data) => {
      value.provenance.witness!.path = "unrelated.rs";
    },
  ]) {
    const data = structuredClone(fixture.data);
    mutate(data);
    assert.throws(() => validateLocalResolveSourceSession(data, fixture.files, plan));
  }
  for (const name of [
    fixture.spec.documentPaths[0],
    fixture.spec.transition.input,
    fixture.spec.localItem,
    "custody/source-law.rs",
  ]) {
    const files = new Map(fixture.files);
    files.set(name, Buffer.concat([files.get(name)!, Buffer.from("changed")]));
    assert.throws(() => validateLocalResolveSourceSession(fixture.data, files, plan), name);
  }
});

test("before evidence cannot claim an unobserved complete session or a different failure item", () => {
  for (const change of ["count", "item", "error"]) {
    const files = new Map(fixture.files);
    const name = "custody/public-before-wire.jsonl.gz";
    const records = gunzipSync(files.get(name)!)
      .toString("utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    // Retain chunk custody and alter the actual framed payload, not a derived counter.
    const chunk = records.find(
      (record) =>
        record.kind === "chunk" &&
        record.direction === (change === "error" ? "server" : "client") &&
        Buffer.from(record.chunkBase64, "base64").includes(Buffer.from('"id":12')),
    );
    assert.ok(chunk);
    let raw = Buffer.from(chunk.chunkBase64, "base64").toString("utf8");
    if (change === "count") raw = raw.replace('"id":12', '"id":99');
    if (change === "item") raw = raw.replace('"label":"id"', '"label":"xx"');
    if (change === "error") raw = raw.replace("-32801", "-32802");
    chunk.chunkBase64 = Buffer.from(raw).toString("base64");
    files.set(name, gzipSync(records.map((record) => JSON.stringify(record)).join("\n") + "\n"));
    assert.throws(() => validateLocalResolveSourceSession(fixture.data, files, plan), change);
  }
});

test("unsupported transitions get no synthetic wire, reference match or native credit", () => {
  const report = referenceReport(loaded, build);
  const row = report.rows.find(({ id }) => id === LOCAL_RESOLVE_SESSION_ID)!;
  assert.deepEqual(row.legacy, sourceSessionUnsupportedLegacy());
  const summary = validateLspReport(loaded, report, build);
  assert.equal(summary.legacyUnsupported, 1);
  assert.equal(summary.legacyMatches, loaded.cases.length - 1);
  assert.equal(summary.nativeHandled, 0);
  for (const extra of [
    { state: "completed", verdict: "matched-reference" },
    { responses: [] },
    { observation: {} },
    { workspace: {} },
  ]) {
    const forged = structuredClone(report);
    Object.assign(forged.rows.find(({ id }) => id === LOCAL_RESOLVE_SESSION_ID)!.legacy, extra);
    assert.throws(() => validateLspReport(loaded, forged, build));
  }
});

test("the handler compares complete CSS documentation from the retained public response", () => {
  const before = JSON.parse(
    gunzipSync(fixture.files.get("custody/public-before-observation.json.gz")!).toString("utf8"),
  );
  const response = before.find(
    (entry: { kind: string; method: string; params?: { label?: string } }) =>
      entry.kind === "response" &&
      entry.method === "completionItem/resolve" &&
      entry.params?.label === "display",
  ).response;
  const source = fs.readFileSync(
    path.join(root, "crates/vize_maestro/src/server/handlers/tests/completion_resolve.rs"),
    "utf8",
  );
  const literal = source.match(/documentation\.value,\s*r#"([\s\S]*?)"#/u);
  assert.ok(literal);
  assert.deepEqual({ kind: "markdown", value: literal[1] }, response.documentation);
});
