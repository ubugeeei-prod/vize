import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { projectViteConfigSource } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import { sha256 } from "../differential/harness.mjs";
import { loadLspManifest } from "../differential/lsp-manifest.ts";
import { validateLspReport } from "../differential/lsp-report.ts";
import type { LspFixture } from "../differential/lsp-types.ts";
import {
  lspWorkspace,
  observeLspWorkspace,
  validateLspWorkspace,
} from "../differential/lsp-workspace.ts";
import { referenceReport } from "./support/lsp/differential-reference.ts";

const manifest = new URL("../_fixtures/differential/lsp/manifest.json", import.meta.url);
const loaded = loadLspManifest(manifest.pathname);
const configured = loaded.cases.filter((fixture) =>
  fixture.files.some((file) => file.runtimePath === "vize.config.json"),
);
const build = {
  sourceRevision: loaded.manifest.baseRevision,
  binaryPath: "target/ci/vize",
  binarySha256: "0".repeat(64),
  cliVersion: "vize 0.429.0-dev",
};
const clone = <T>(value: T): T => structuredClone(value);

function appendDeclared(fixture: LspFixture, runtimePath: string, source: string) {
  const bytes = Buffer.from(source);
  fixture.files.push({ runtimePath, bytes });
  fixture.data.files!.push({ runtimePath, source: runtimePath, sha256: sha256(bytes) });
  fixture.inputs.files.push({ path: runtimePath, sha256: sha256(bytes) });
}

test("all24 pinned LSP cases retain their inputs and only the two configured outline cases gain a carrier", () => {
  assert.equal(loaded.cases.length, 24);
  assert.deepEqual(
    configured.map((fixture) => fixture.id),
    [
      "lsp/regression/document-symbol-outline-parent",
      "lsp/regression/document-symbol-outline-myswitch",
    ],
  );
  for (const fixture of loaded.cases) {
    const before = fixture.files.map((file) => ({
      runtimePath: file.runtimePath,
      base64: file.bytes.toString("base64"),
    }));
    const plan = lspWorkspace(fixture);
    if (!configured.includes(fixture)) {
      assert.deepEqual(plan.files, fixture.files);
      assert.deepEqual(plan.mapping.configurations, []);
    } else {
      assert.equal(plan.files.length, fixture.files.length + 1);
      assert.equal(plan.mapping.configurations.length, 1);
      assert.deepEqual(
        plan.files.map((file) => file.runtimePath),
        [
          "src/MySwitch.vue",
          "src/Parent.vue",
          "tsconfig.json",
          "vize.config.json.fixture",
          "vite.config.mjs",
        ],
      );
    }
    assert.deepEqual(
      fixture.files.map((file) => ({
        runtimePath: file.runtimePath,
        base64: file.bytes.toString("base64"),
      })),
      before,
    );
  }
  const report = referenceReport(loaded, build);
  assert.deepEqual(report.argv, ["lsp"]);
  assert.equal(validateLspReport(loaded, report, build).legacyMatches, 24);
  assert.equal(report.summary.nativeHandled, 0);
  assert.equal(report.summary.nativeEquivalent, 0);
});

test("actual current carriers import and project the complete archived policy without changing raw corpus bytes", async (t) => {
  for (const fixture of configured) {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-carrier-law-"));
    t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
    const plan = lspWorkspace(fixture);
    for (const file of plan.files) {
      const destination = path.join(directory, file.runtimePath);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.writeFileSync(destination, file.bytes);
    }
    const mapping = plan.mapping.configurations[0];
    const raw = fixture.files.find((file) => file.runtimePath === mapping.originalPath)!.bytes;
    assert.deepEqual(fs.readFileSync(path.join(directory, mapping.evidencePath)), raw);
    assert.equal(mapping.originalBase64, raw.toString("base64"));
    assert.equal(mapping.originalSha256, sha256(raw));
    assert.equal(fs.existsSync(path.join(directory, "vize.config.json")), false);
    const { default: exported } = await import(
      pathToFileURL(path.join(directory, mapping.carrierPath)).href
    );
    assert.deepEqual(await projectViteConfigSource(exported), {
      ...JSON.parse(raw.toString()),
      typeChecker: { jsxTypecheck: false },
    });
    const evidence = {
      mapping: plan.mapping,
      before: observeLspWorkspace(directory, plan.mapping),
      after: observeLspWorkspace(directory, plan.mapping),
    };
    validateLspWorkspace(fixture, evidence, true);
    fs.appendFileSync(path.join(directory, mapping.carrierPath), "\n");
    evidence.after = observeLspWorkspace(directory, plan.mapping);
    assert.throws(
      () => validateLspWorkspace(fixture, evidence, true),
      /actual after workspace inputs changed/,
    );
  }
});

test("mapping refuses missing, extra and broken original inputs and declared references", () => {
  for (const mutate of [
    (fixture: LspFixture) => {
      fixture.files.pop();
    },
    (fixture: LspFixture) => {
      fixture.files.push({ runtimePath: "extra.vue", bytes: Buffer.from("<template/>") });
    },
    (fixture: LspFixture) => {
      fixture.files[3].bytes = Buffer.from("{}");
    },
    (fixture: LspFixture) => {
      fixture.data.files![3].sha256 = "0".repeat(64);
    },
    (fixture: LspFixture) => {
      fixture.inputs.files.pop();
    },
  ]) {
    const fixture = clone(configured[0]);
    mutate(fixture);
    assert.throws(() => lspWorkspace(fixture));
  }
  for (const contents of [
    "{broken",
    '{"extends":"./policy.json"}',
    '{"lsp":{},"languageServer":{}}',
  ]) {
    const fixture = clone(configured[0]);
    fixture.files.pop();
    fixture.data.files!.pop();
    fixture.inputs.files.pop();
    appendDeclared(fixture, "vize.config.json", contents);
    assert.throws(() => lspWorkspace(fixture), undefined, contents);
  }
  for (const collision of ["vite.config.mjs", "vize.config.json.fixture"]) {
    const fixture = clone(configured[0]);
    appendDeclared(fixture, collision, "{}");
    assert.throws(() => lspWorkspace(fixture));
  }
});

test("whole report validation rejects transport, source, carrier and observed-file drift without relaxing the wire oracle", () => {
  for (const mutation of [
    (workspace: ReturnType<typeof lspWorkspace>["mapping"]) => {
      workspace.originalFiles.pop();
    },
    (workspace: ReturnType<typeof lspWorkspace>["mapping"]) => {
      workspace.originalFiles.push({ path: "extra", sha256: "0".repeat(64) });
    },
    (workspace: ReturnType<typeof lspWorkspace>["mapping"]) => {
      workspace.configurations[0].originalBase64 = Buffer.from("{}").toString("base64");
    },
    (workspace: ReturnType<typeof lspWorkspace>["mapping"]) => {
      workspace.configurations[0].originalSha256 = "0".repeat(64);
    },
    (workspace: ReturnType<typeof lspWorkspace>["mapping"]) => {
      workspace.configurations[0].carrierBase64 += "AA==";
    },
    (workspace: ReturnType<typeof lspWorkspace>["mapping"]) => {
      workspace.configurations[0].carrierSha256 = "0".repeat(64);
    },
    (workspace: ReturnType<typeof lspWorkspace>["mapping"]) => {
      workspace.configurations[0].evidencePath = "vize.config.json";
    },
    (workspace: ReturnType<typeof lspWorkspace>["mapping"]) => {
      workspace.files[0].sha256 = "0".repeat(64);
    },
  ]) {
    const report = referenceReport(loaded, build);
    const row = report.rows.find((row) => row.id === configured[0].id)!;
    mutation(row.legacy.workspace!.mapping);
    assert.throws(() => validateLspReport(loaded, report, build));
  }
  for (const phase of ["before", "after"] as const) {
    for (const change of ["missing", "extra", "changed", "absent"] as const) {
      const report = referenceReport(loaded, build);
      const workspace = report.rows.find((row) => row.id === configured[0].id)!.legacy.workspace!;
      if (change === "missing") workspace[phase]!.pop();
      if (change === "extra") workspace[phase]!.push({ path: "extra", sha256: "0".repeat(64) });
      if (change === "changed") workspace[phase]![0].sha256 = "0".repeat(64);
      if (change === "absent") delete workspace[phase];
      assert.throws(
        () => validateLspReport(loaded, report, build),
        undefined,
        `${phase}/${change}`,
      );
    }
  }
});
