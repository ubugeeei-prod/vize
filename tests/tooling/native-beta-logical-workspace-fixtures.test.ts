import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  LOGICAL_WORKSPACE_PROTOCOL_ID,
  newLogicalWorkspaceProtocolCases,
} from "../../tools/support/compat/github/native-beta-logical-workspace-fixtures.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";
import { summarizeDialectCoverage } from "../differential/harness.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
// Private integration preparation can use the actual consumer owner's checkout.
// Every source still has to match the same literal authority hashes.
const sourceRoot = process.env.VIZE_LOGICAL_PROTOCOL_SOURCE_ROOT ?? root;
const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const fixtureRoot =
  "tests/_fixtures/differential/typechecker/logical-workspace-overlay-revision-11145/";
const authority = JSON.parse(
  fs.readFileSync(path.join(root, fixtureRoot, "source-authority.json"), "utf8"),
);
const manifest = JSON.parse(fs.readFileSync(path.join(root, manifestPath), "utf8"));
const row = manifest.protocolCases.find(
  (value: { id: string }) => value.id === LOGICAL_WORKSPACE_PROTOCOL_ID,
);
const oldManifest = structuredClone(manifest);
oldManifest.protocolCases = oldManifest.protocolCases.filter(
  (value: { id: string }) => value.id !== LOGICAL_WORKSPACE_PROTOCOL_ID,
);
const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function read(side: "base" | "head", file: string): Buffer | null {
  if (file === manifestPath)
    return Buffer.from(JSON.stringify(side === "base" ? oldManifest : manifest));
  try {
    if (side === "base")
      return execFileSync("git", ["show", `${authority.production.beforeRevision}:${file}`], {
        cwd: root,
        stdio: ["ignore", "pipe", "ignore"],
      });
    return fs.readFileSync(path.join(file.startsWith(fixtureRoot) ? root : sourceRoot, file));
  } catch {
    return null;
  }
}
type Overrides = Record<string, Buffer | null>;
function validate(rows: unknown = [row], previous: unknown = [], overrides: Overrides = {}) {
  return newLogicalWorkspaceProtocolCases(previous, rows, (side, file) =>
    Object.hasOwn(overrides, `${side}:${file}`) ? overrides[`${side}:${file}`]! : read(side, file),
  );
}

void test("logical compiled input retains exact bytes with either Git autocrlf checkout mode", () => {
  const input = authority.consumer.input;
  const original = execFileSync("git", ["cat-file", "blob", `HEAD:${input.path}`], { cwd: root });
  assert.equal(original.length, 1108);
  assert.equal(sha(original), input.sha256);
  for (const mode of ["false", "true"]) {
    const filtered = execFileSync(
      "git",
      ["-c", `core.autocrlf=${mode}`, "cat-file", "--filters", `HEAD:${input.path}`],
      { cwd: root },
    );
    assert.deepEqual(filtered, original);
  }
});

void test("logical registration consumes the actual pinned input and all eight source pairs without native admission", () => {
  const result = validate();
  assert.deepEqual(result.ids, [LOGICAL_WORKSPACE_PROTOCOL_ID]);
  assert.equal(result.sources.size, 10);
  assert.equal(authority.nativeAdmission, false);
  assert.equal(authority.consumer.receipt, "attached.observerInputDependencies");
  assert.deepEqual(
    authority.nativeRoles.map((value: { role: string; sdk: string }) => [value.role, value.sdk]),
    [
      ["before", "7.0.2"],
      ["before", "7.1.0-beta"],
      ["after", "7.0.2"],
      ["after", "7.1.0-beta"],
    ],
  );
  assert.equal(sha(read("head", authority.consumer.input.path)!), authority.consumer.input.sha256);
  assert.equal(authority.consumer.corePrefix.bytes, 12150);
});

void test("protocol registration leaves authored cases, dialect rates and historical protocol rows unchanged", () => {
  const loaded = loadTypecheckerManifest(path.join(root, manifestPath));
  assert(!loaded.cases.some((value) => value.id === LOGICAL_WORKSPACE_PROTOCOL_ID));
  assert.deepEqual(manifest.cases, oldManifest.cases);
  assert.deepEqual(
    summarizeDialectCoverage(manifest.cases),
    summarizeDialectCoverage(oldManifest.cases),
  );
  const original = JSON.parse(
    execFileSync("git", ["show", `ae58ff2f6a5eb8ce6c752169a255268ac87a4971:${manifestPath}`], {
      cwd: root,
      encoding: "utf8",
    }),
  );
  for (const historical of original.protocolCases) {
    assert.deepEqual(
      oldManifest.protocolCases.find((value: { id: string }) => value.id === historical.id),
      historical,
    );
  }
  const oldPairs =
    "tests/_fixtures/differential/typechecker/native-beta-diagnostic-dto-10731/source-pairs.json";
  assert.equal(
    sha(fs.readFileSync(path.join(root, oldPairs))),
    "512e95f921e6ec064f281ec00d777cab0df66964d69087384cbb8615c22e21b1",
  );
});

void test("closed protocol rows refuse duplicate, unknown, dropped and modified history", () => {
  assert.deepEqual(validate([], []).ids, []);
  assert.deepEqual(validate([row], [row]).ids, []);
  for (const rows of [
    [row, row],
    [{ ...row, id: "typechecker/protocol/unknown" }],
    [{ ...row, nativeAdmission: true }],
    [{ ...row, targets: ["batch"] }],
    [{ ...row, tests: [] }],
  ])
    assert.throws(() => validate(rows));
  assert.throws(() => validate([], [row]));
  assert.throws(() => validate([{ ...row, state: "draft" }], [row]));
  for (const field of ["path", "sha256"]) {
    const changed = structuredClone(row);
    changed.inputs[0][field] = "forged";
    assert.throws(() => validate([changed]));
  }
});

void test("whole authored input, source authority, production preimages and compiled consumers reject mutation", () => {
  for (const file of [
    authority.consumer.input.path,
    fixtureRoot + "source-authority.json",
    authority.consumer.observer.path,
    authority.consumer.helper.path,
  ]) {
    assert.throws(() => validate([row], [], { [`head:${file}`]: null }));
    assert.throws(() =>
      validate([row], [], {
        [`head:${file}`]: Buffer.concat([read("head", file)!, Buffer.from(" ")]),
      }),
    );
  }
  for (const entry of authority.production.files)
    for (const side of ["base", "head"] as const) {
      assert.throws(() => validate([row], [], { [`${side}:${entry.path}`]: null }));
      assert.throws(() =>
        validate([row], [], {
          [`${side}:${entry.path}`]: Buffer.concat([read(side, entry.path)!, Buffer.from(" ")]),
        }),
      );
    }
  const forged = structuredClone(authority);
  forged.nativeAdmission = true;
  const bytes = Buffer.from(JSON.stringify(forged));
  const changed = structuredClone(row);
  changed.inputs[1].sha256 = sha(bytes);
  assert.throws(() =>
    validate([changed], [], { [`head:${fixtureRoot}source-authority.json`]: bytes }),
  );
});

void test("all four genuine workflow roles and the actual Rust selector remain mandatory", () => {
  const file = ".github/workflows/native-beta-diagnostic-dto.yml";
  const source = read("head", file)!.toString("utf8");
  for (const marker of [
    "role: [before, after]",
    "version: 7.1.0-beta",
    "version: 7.0.2",
    "13fa41356d075047a229f580014b637d3bbeb57f",
    row.tests[0].selector,
  ])
    assert.throws(() =>
      validate([row], [], { [`head:${file}`]: Buffer.from(source.replaceAll(marker, "removed")) }),
    );
});

void test("planner routes only this closed logical case and refuses unknown protocols and unrelated source", () => {
  const paths = [authority.production.files[5].path, authority.consumer.helper.path];
  const report = analyzeLegacyFix({
    title: "fix(lsp): preserve logical workspace revisions",
    paths,
    readFile: read,
  });
  assert.deepEqual(report.products, [
    {
      product: "typechecker",
      state: "new-inputs-registered",
      cases: [LOGICAL_WORKSPACE_PROTOCOL_ID],
    },
  ]);
  const unrelated = analyzeLegacyFix({
    title: "fix(lsp): preserve logical workspace revisions",
    paths: [...paths, "crates/vize_canon/src/type_checker.rs"],
    readFile: read,
  });
  assert.equal(unrelated.products[0]!.state, "new-input-missing");
  const noLogical = analyzeLegacyFix({
    title: "fix(lsp): preserve logical workspace revisions",
    paths,
    readFile: (side, file) =>
      file === manifestPath && side === "head"
        ? Buffer.from(JSON.stringify(oldManifest))
        : read(side, file),
  });
  assert.equal(noLogical.products[0]!.state, "new-input-missing");
  const unknown = structuredClone(manifest);
  unknown.protocolCases.push({ id: "typechecker/protocol/unknown", state: "active" });
  const refused = analyzeLegacyFix({
    title: "fix(lsp): preserve logical workspace revisions",
    paths,
    readFile: (side, file) =>
      file === manifestPath && side === "head"
        ? Buffer.from(JSON.stringify(unknown))
        : read(side, file),
  });
  assert.equal(refused.products[0]!.state, "invalid-manifest-or-input");
});
