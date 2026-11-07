import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { sha256 } from "../differential/sha256.ts";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { sourceInspectedDialectCoverage } from "../differential/source-dialect-coverage.ts";
import { sourceSessionUnsupportedLegacy } from "../differential/lsp-source-sessions.ts";
import {
  MODEL_BINDING_UNIT_ID,
  MODEL_BINDING_INPUT_SHA256,
  MODEL_BINDING_UNIT_REASON,
  MODEL_BINDING_UNIT_SESSION,
  MODEL_BINDING_UNIT_PLAN,
  validateModelBindingSourceUnit,
  modelBindingSourceUnitDialectInputs,
} from "../differential/lsp-model-binding-source-unit.ts";
import type { FixtureData } from "../differential/lsp-types.ts";

const fixtureRoot = fileURLToPath(
  new URL("../_fixtures/differential/lsp/reported-model-binding-rename-11383/", import.meta.url),
);
const input = fs.readFileSync(path.join(fixtureRoot, "input.json"));
const data = JSON.parse(
  fs.readFileSync(path.join(fixtureRoot, "case.json"), "utf8"),
) as FixtureData;
const plan: unknown = JSON.parse(
  fs.readFileSync(path.join(fixtureRoot, "source-unit.expected.json"), "utf8"),
);
const files = () => new Map([["input.json", input]]);

function manifest(root: string) {
  fs.cpSync(fixtureRoot, path.join(root, "unit"), { recursive: true });
  const entry = {
    id: MODEL_BINDING_UNIT_ID,
    state: "active",
    targets: ["stdio"],
    case: {
      path: "unit/case.json",
      sha256: sha256(fs.readFileSync(path.join(root, "unit/case.json"))),
    },
    inputs: { root: "unit", files: [{ path: "input.json", sha256: MODEL_BINDING_INPUT_SHA256 }] },
    requestCount: 0,
    initializationOptions: data.initializationOptions,
    dialectCoverage: sourceInspectedDialectCoverage(
      modelBindingSourceUnitDialectInputs(files()),
      "LSP manifest pinned authored sources",
    ),
  };
  const target = path.join(root, "manifest.json");
  fs.writeFileSync(
    target,
    JSON.stringify({
      schema: "vize.differential.manifest",
      version: 1,
      product: "lsp",
      baseRevision: data.provenance.fixCommit,
      cases: [entry],
    }),
  );
  return target;
}

test("physical model-binding input loads with zero generic requests and explicit unsupported disposition", () => {
  assert.equal(input.length, 394);
  assert.equal(sha256(input), MODEL_BINDING_INPUT_SHA256);
  assert.deepEqual(
    sourceInspectedDialectCoverage(modelBindingSourceUnitDialectInputs(files()), "actual source")
      .evidence[0]?.dialects,
    ["vue3-sfc", "html-template", "ts"],
  );
  validateModelBindingSourceUnit(data, files(), plan);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-model-binding-unit-"));
  try {
    const loaded = loadLspManifest(manifest(root));
    assert.equal(loaded.cases.length, 1);
    assert.deepEqual(loaded.cases[0]?.requests, []);
    assert.deepEqual(loaded.cases[0]?.data.sourceSession, MODEL_BINDING_UNIT_SESSION);
    assert.deepEqual(sourceSessionUnsupportedLegacy(MODEL_BINDING_UNIT_SESSION), {
      state: "unsupported",
      verdict: "unsupported-source-unit",
      comparator: "source-inspected-unit",
      reason: MODEL_BINDING_UNIT_REASON,
    });
    assert.deepEqual(plan, MODEL_BINDING_UNIT_PLAN);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("source-unit refuses foreign executor, test, requests and fabricated native/capture credit", () => {
  for (const [key, value] of Object.entries({
    executor: "foreign.rs",
    test: "foreign::test",
    plannedClientRpcCount: 1,
    nativeParityChecked: 1,
    capturedSessions: 1,
    qualified: true,
  })) {
    const changed = structuredClone(data);
    assert.ok(changed.sourceSession);
    Object.assign(changed.sourceSession, { [key]: value });
    assert.throws(() => validateModelBindingSourceUnit(changed, files(), plan));
  }
});

test("source-unit refuses missing, altered and extra physical inputs", () => {
  assert.throws(() => validateModelBindingSourceUnit(data, new Map(), plan));
  assert.throws(() =>
    validateModelBindingSourceUnit(
      data,
      new Map([["input.json", Buffer.concat([input, Buffer.from(" ")])]]),
      plan,
    ),
  );
  assert.throws(() =>
    validateModelBindingSourceUnit(
      data,
      new Map([...files(), ["extra.vue", Buffer.from("<template />")]]),
      plan,
    ),
  );
});

test("source-unit refuses ordinary-RPC reinterpretation, foreign case and forged expected plan", () => {
  for (const patch of [
    { id: "lsp/regression/foreign" },
    { openDocument: true },
    { method: "textDocument/rename" },
    { extra: true },
  ]) {
    assert.throws(() =>
      validateModelBindingSourceUnit(Object.assign(structuredClone(data), patch), files(), plan),
    );
  }
  for (const patch of [
    { genericClientRpcCount: 1 },
    { qualified: true },
    { capturedSessions: 1 },
    { extra: true },
  ]) {
    assert.throws(() =>
      validateModelBindingSourceUnit(data, files(), { ...MODEL_BINDING_UNIT_PLAN, ...patch }),
    );
  }
  const changed = structuredClone(data);
  changed.expected.sha256 = "0".repeat(64);
  assert.throws(() => validateModelBindingSourceUnit(changed, files(), plan));
});

test("the existing two source-session unsupported dispositions remain unchanged", () => {
  assert.deepEqual(sourceSessionUnsupportedLegacy(), {
    state: "unsupported",
    verdict: "unsupported-transition",
    comparator: "source-inspected-session",
    reason:
      "The initial-document adapter cannot execute this pinned multi-root configuration transition; the dedicated source executor supplies its own full wire and source custody.",
  });
});
