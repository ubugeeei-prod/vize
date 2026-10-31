import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import {
  listenerEventPack,
  selectListenerEventAuthority,
} from "../differential/typechecker-listener-events.ts";
import {
  loadTypecheckerManifest,
  typecheckerReport,
  workspaceTypecheckerReport,
  validateTypecheckerCapture,
  validateWorkspaceTypecheckerCapture,
  type Capture,
  type TypecheckerCase,
} from "../differential/typechecker.ts";

const root = fileURLToPath(new URL("../_fixtures/differential/typechecker/", import.meta.url));
const manifest = JSON.parse(fs.readFileSync(path.join(root, "manifest.json"), "utf8"));
const frozen = JSON.parse(
  fs.readFileSync(path.join(root, listenerEventPack, "cases.json"), "utf8"),
);
const currentFile = path.join(listenerEventPack, "native-event-current.json");
const current = JSON.parse(fs.readFileSync(path.join(root, currentFile), "utf8"));
function original(index = 0): TypecheckerCase {
  return {
    ...structuredClone(
      manifest.cases.filter((item: TypecheckerCase) => item.pack === listenerEventPack)[index],
    ),
    diagnostics: structuredClone(frozen.cases[index].diagnostics),
  };
}
function sandbox() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-art-display-"));
  fs.copyFileSync(path.join(root, "manifest.json"), path.join(directory, "manifest.json"));
  fs.cpSync(path.join(root, listenerEventPack), path.join(directory, listenerEventPack), {
    recursive: true,
  });
  return directory;
}

test("the real manifest selects the full listener successor with separate authority", () => {
  const before = original(),
    selected = selectListenerEventAuthority(root, before);
  assert.deepEqual(before, original(), "selection does not normalize its historical input");
  assert.deepEqual(selected.diagnostics, current.cases[0].diagnostics);
  assert.deepEqual(selected.packReference, before.packReference);
  const loaded = loadTypecheckerManifest(path.join(root, "manifest.json"));
  const rows = loaded.cases.filter((item) => item.pack === listenerEventPack);
  assert.deepEqual(
    rows.map((item) => item.diagnostics),
    current.cases.map((item: { diagnostics: unknown }) => item.diagnostics),
  );
  assert.deepEqual(
    rows.map((item) => item.diagnosticAuthority),
    [selected.diagnosticAuthority],
  );
  assert.equal(
    loaded.cases.filter((item) => item.pack === listenerEventPack && item.diagnosticAuthority)
      .length,
    1,
  );
});

test("the actual selector refuses mutations of every authenticated authority", async (t) => {
  const changes: [string, (root: string) => void][] = [
    [
      "historical pack",
      (dir) => fs.appendFileSync(path.join(dir, listenerEventPack, "cases.json"), " "),
    ],
    [
      "child contract",
      (dir) => fs.appendFileSync(path.join(dir, listenerEventPack, "src/Child.vue.txt"), " "),
    ],
    [
      "parent source",
      (dir) => fs.appendFileSync(path.join(dir, listenerEventPack, "src/App.vue.txt"), " "),
    ],
    [
      "compiler config",
      (dir) => fs.appendFileSync(path.join(dir, listenerEventPack, "tsconfig.json.txt"), " "),
    ],
    [
      "manifest context",
      (dir) => {
        const m = structuredClone(manifest);
        m.cases.find((item: TypecheckerCase) => item.pack === listenerEventPack).state =
          "unregistered";
        fs.writeFileSync(path.join(dir, "manifest.json"), JSON.stringify(m));
      },
    ],
  ];
  for (const [name, mutate] of changes)
    await t.test(name, () => {
      const dir = sandbox();
      try {
        mutate(dir);
        assert.throws(() => selectListenerEventAuthority(dir, original()));
      } finally {
        fs.rmSync(dir, { recursive: true });
      }
    });
  for (const field of [
    "file",
    "line",
    "column",
    "severity",
    "code",
    "message",
    "count",
    "order",
    "sourceRevision",
    "obsolete-event",
  ])
    await t.test(`current vector ${field}`, () => {
      const dir = sandbox(),
        changed = structuredClone(current);
      try {
        if (field === "sourceRevision") changed.sourceRevision = "0".repeat(40);
        else if (field === "count") changed.cases[0].diagnostics.pop();
        else if (field === "order") changed.cases[0].diagnostics.reverse();
        else if (field === "obsolete-event")
          changed.cases[0].diagnostics.push(frozen.cases[0].diagnostics[0]);
        else
          changed.cases[0].diagnostics[1][field] =
            typeof changed.cases[0].diagnostics[1][field] === "number" ? 0 : "unreviewed";
        fs.writeFileSync(path.join(dir, currentFile), JSON.stringify(changed));
        assert.throws(() => selectListenerEventAuthority(dir, original()));
      } finally {
        fs.rmSync(dir, { recursive: true });
      }
    });
  for (const field of ["pack", "case", "sourceRevision", "id", "test", "targets"])
    await t.test(`requested ${field}`, () => {
      const changed = original();
      Object.assign(changed, { [field]: "unknown" });
      assert.throws(() => selectListenerEventAuthority(root, changed));
    });
});

test("whole actual capture validation and report retain the selected authority", () => {
  const loaded = loadTypecheckerManifest(path.join(root, "manifest.json"));
  const receipt = Buffer.from("synthetic custody control; no native execution credit"),
    binarySha256 = sha256(receipt);
  const packs = [...new Set(loaded.cases.map((item) => item.pack))];
  const captures = packs.map((pack): Capture => {
    const fixtures = loaded.cases.filter((item) => item.pack === pack);
    return {
      schema: "vize.typechecker-fixture-observation",
      version: 1,
      pack,
      test: fixtures[0].test,
      fixturePackSha256: fixtures[0].packReference.sha256,
      archiveReceiptSha256: sha256(receipt),
      binarySha256,
      binaryPath: "/synthetic/control",
      missingFields: ["end", "relatedInformation", "raw backend diagnostics"],
      matchedContract: "batch-start-diagnostics-v1",
      unbaselinedFields: ["diagnostics[].blockType", "exitCode", "success"],
      cases: fixtures.map((fixture) => ({
        id: fixture.case,
        projectRoot: "/synthetic/project",
        inputs: fixture.inputs.map((input) => ({ file: input.file, sha256: input.sha256 })),
        diagnostics: structuredClone(fixture.diagnostics),
        publicResult: {
          exitCode: fixture.diagnostics.length ? 1 : 0,
          success: fixture.diagnostics.length === 0,
          diagnostics: fixture.diagnostics.map((d) => ({
            ...d,
            file: `/synthetic/project/${d.file}`,
            line: d.line - 1,
            column: d.column - 1,
            blockType: null,
          })),
        },
      })),
    };
  });
  for (const capture of captures)
    validateTypecheckerCapture(loaded, capture, { receipt, binarySha256 });
  const report = typecheckerReport(loaded, captures, "1".repeat(40));
  assert.deepEqual(
    report.rows
      .filter((row) => row.id.startsWith(`typechecker/${listenerEventPack}/`))
      .map((row) => row.legacy.diagnosticAuthority),
    loaded.cases
      .filter((item) => item.pack === listenerEventPack)
      .map((item) => item.diagnosticAuthority),
  );
  const workspace = captures.map(({ archiveReceiptSha256: _archive, ...capture }) => ({
    ...capture,
    version: 2 as const,
    kind: "full-workspace" as const,
    workspaceReceiptSha256: sha256(receipt),
  }));
  for (const capture of workspace)
    validateWorkspaceTypecheckerCapture(loaded, capture, { receipt, binarySha256 });
  const workspaceReport = workspaceTypecheckerReport(loaded, workspace, "1".repeat(40));
  assert.deepEqual(
    workspaceReport.rows.map((row) => row.legacy.diagnosticAuthority),
    report.rows.map((row) => row.legacy.diagnosticAuthority),
  );
  const originalCapture = captures.find((capture) => capture.pack === listenerEventPack)!;
  for (const mutate of [
    (capture: Capture) => {
      capture.cases[0].diagnostics = structuredClone(frozen.cases[0].diagnostics);
    },
    (capture: Capture) => {
      capture.cases[0].publicResult.diagnostics[1].message = "unreviewed";
    },
    (capture: Capture) => {
      capture.cases[0].diagnostics[1].column++;
    },
    (capture: Capture) => {
      capture.cases[0].diagnostics.pop();
    },
  ]) {
    const changed = structuredClone(originalCapture);
    mutate(changed);
    assert.throws(() => validateTypecheckerCapture(loaded, changed, { receipt, binarySha256 }));
  }
});
