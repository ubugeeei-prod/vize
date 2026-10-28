import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gzipSync, gunzipSync } from "node:zlib";
import { readPinnedArtifact } from "../differential/harness.mjs";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { validateLspReport } from "../differential/lsp-report.ts";
import {
  LOCAL_RESOLVE_SESSION_ID,
  readLocalResolveSourceSession,
  sourceSessionUnsupportedLegacy,
  validateLocalResolveSourceSession,
} from "../differential/lsp-source-sessions.ts";
import { referenceObservation, referenceReport } from "./support/lsp/differential-reference.ts";
import {
  NATIVE_CAPACITY_ID,
  NATIVE_CAPACITY_UNSUPPORTED_REASON,
} from "../differential/lsp-native-capacity-contract.ts";

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
      const session = value.sourceSession;
      assert.ok(session?.kind === "local-completion-resolve-v1");
      session.plannedClientRpcCount = 22 as 23;
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
  const unsupportedIds = [LOCAL_RESOLVE_SESSION_ID, NATIVE_CAPACITY_ID];
  assert.deepEqual(
    report.rows.filter(({ legacy }) => legacy.state === "unsupported").map(({ id }) => id),
    unsupportedIds,
  );
  const capacity = report.rows.find(({ id }) => id === NATIVE_CAPACITY_ID);
  assert.ok(capacity);
  assert.deepEqual(capacity.legacy, {
    state: "unsupported",
    verdict: "unsupported-transition",
    comparator: "source-inspected-session",
    reason: NATIVE_CAPACITY_UNSUPPORTED_REASON,
  });
  assert.equal(summary.legacyUnsupported, unsupportedIds.length);
  assert.equal(summary.legacyMatches, loaded.cases.length - unsupportedIds.length);
  assert.equal(summary.nativeHandled, 0);
  for (const extra of [
    { state: "completed", verdict: "matched-reference" },
    { responses: [] },
    { observation: {} },
    { workspace: {} },
  ]) {
    for (const id of unsupportedIds) {
      const forged = structuredClone(report);
      const changed = forged.rows.find((row) => row.id === id);
      assert.ok(changed);
      Object.assign(changed.legacy, extra);
      assert.throws(() => validateLspReport(loaded, forged, build));
    }
  }
});

test("the handler compares complete CURRENT CSS documentation and preserves the public response", () => {
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
  const current = JSON.parse(
    readPinnedArtifact(
      path.join(root, "tests/_fixtures/differential/lsp/css-hover-stylesheet-examples-10572"),
      {
        path: "local-completion-resolve.expected-current.json",
        sha256: "33f35ddde74f1bc0221e47d07d891d594bac2c86fdaea0c21b3ae2539a667035",
      },
    ).toString("utf8"),
  );
  // This reference-only law permits exactly the authored Example delta.
  // Runtime comparisons use the complete fixed CURRENT object unchanged.
  assert.deepEqual({ ...current, documentation: response.documentation }, response);
  assert.equal(
    current.documentation.value,
    response.documentation.value.replace(
      "```css\ndisplay: block;\n```",
      "```css\n.example {\n  display: block;\n}\n```",
    ),
  );
  assert.deepEqual({ kind: "markdown", value: literal[1] }, current.documentation);
});
