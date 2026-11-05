import assert from "node:assert/strict";
import { test } from "node:test";
import { compilerArgv, COMPILER_ARGV } from "../differential/compiler.mjs";

interface MutableCase {
  id: string;
  targets: string[];
  inputs: { root: string; files: Array<{ path: string; role: string; sha256: string }> };
  adapters: { legacy: string; legacyRecipe: string; native: null; reasons: { native: string } };
  reference: { path: string; sha256: string; source: string };
  comparison: { contract: string; requiredFacets: string[]; scriptLang: string };
  dialectCoverage: { state: string; registryRef: string | null; evidence: unknown[] };
  argv?: string[];
}

const id = "compiler/dom/typed-handler-runtime-spelling-11815";
const domArgv = ["build", "Layout.vue", "--format", "json", "--output", "out", "--no-config"];

void test("the finite DOM case uses the original CLI with only SSR mode removed", () => {
  assert.deepEqual(compilerArgv({ id, targets: ["dom"] }), domArgv);
  assert.deepEqual(compilerArgv({ targets: ["ssr"] }), COMPILER_ARGV);
  assert.deepEqual(compilerArgv({}), COMPILER_ARGV);
});

void test("unknown targets and foreign DOM selectors cannot select a compiler command", () => {
  for (const fixture of [
    { id, targets: [] },
    { id, targets: ["native"] },
    { id, targets: ["dom", "ssr"] },
    { id: "compiler/dom/foreign", targets: ["dom"] },
    { id, targets: ["dom"], argv: ["build", "--unsafe"] },
    { id, targets: ["dom"], inputs: { dialectInput: {} } },
  ]) {
    assert.throws(() => compilerArgv(fixture));
  }
});

void test("the new DOM manifest cannot repin its input, oracle, recipe or source authority", async () => {
  const { readFileSync } = await import("node:fs");
  const { validateCompilerDomCase } = await import("../differential/compiler-dom.ts");
  const manifest = JSON.parse(
    readFileSync(
      new URL("../_fixtures/differential/compiler/manifest.json", import.meta.url),
      "utf8",
    ),
  );
  const original = (manifest as { cases: MutableCase[] }).cases.find(
    (entry: { id: string }) => entry.id === id,
  );
  assert.ok(original, "the finite DOM case must be registered");
  validateCompilerDomCase(original);
  const changes: Array<(value: MutableCase) => void> = [
    (value) => {
      value.targets = ["ssr"];
    },
    (value) => {
      value.adapters.legacyRecipe += " --features legacy";
    },
    (value) => {
      value.adapters.legacy = "compiler-cli-ssr-v1";
    },
    (value) => {
      value.inputs.files[0].sha256 = "0".repeat(64);
    },
    (value) => {
      value.inputs.files.push(structuredClone(value.inputs.files[0]));
    },
    (value) => {
      value.inputs.files[0].path = "../Layout.vue.txt";
    },
    (value) => {
      value.reference.sha256 = "0".repeat(64);
    },
    (value) => {
      value.reference.source = "legacy source with unrelated provenance";
    },
    (value) => {
      value.comparison.requiredFacets.pop();
    },
    (value) => {
      value.comparison.scriptLang = "js";
    },
    (value) => {
      value.argv = ["build", "--unsafe"];
    },
    (value) => {
      value.dialectCoverage.state = "verified";
    },
  ];
  for (const change of changes) {
    const value = structuredClone(original);
    change(value);
    assert.throws(() => validateCompilerDomCase(value));
  }
});

void test("DOM input and oracle bytes remain pinned before materialization", async () => {
  const fs = await import("node:fs");
  const os = await import("node:os");
  const path = await import("node:path");
  const { loadCompilerManifest } = await import("../differential/compiler.mjs");
  const source = new URL("../_fixtures/differential/compiler/", import.meta.url);
  const { fileURLToPath } = await import("node:url");
  const original = JSON.parse(fs.readFileSync(new URL("manifest.json", source), "utf8"));
  const fixture = (original as { cases: MutableCase[] }).cases.find(
    (entry: { id: string }) => entry.id === id,
  );
  assert.ok(fixture);
  for (const name of ["Layout.vue.txt", "module.expected.txt"]) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-dom-pins-"));
    try {
      const manifest = { ...original, cases: [fixture], cssApiCases: undefined };
      fs.mkdirSync(path.join(root, fixture.inputs.root));
      for (const entry of ["Layout.vue.txt", "module.expected.txt"]) {
        fs.copyFileSync(
          fileURLToPath(new URL(`${fixture.inputs.root}/${entry}`, source)),
          path.join(root, fixture.inputs.root, entry),
        );
      }
      fs.writeFileSync(path.join(root, "manifest.json"), JSON.stringify(manifest));
      loadCompilerManifest(path.join(root, "manifest.json"));
      fs.appendFileSync(path.join(root, fixture.inputs.root, name), "\n");
      assert.throws(
        () => loadCompilerManifest(path.join(root, "manifest.json")),
        /SHA256 mismatch/,
      );
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  }
});

void test("a legacy source recipe cannot execute or claim the new DOM case", async () => {
  const { runCompilerPack } = await import("../differential/compiler.mjs");
  const { BUILD_RECIPE, LEGACY_BUILD_RECIPE } = await import("../differential/build-receipt.ts");
  const { fileURLToPath } = await import("node:url");
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const report = runCompilerPack({
    manifestPath: fileURLToPath(
      new URL("../_fixtures/differential/compiler/manifest.json", import.meta.url),
    ),
    binaryPath: `${root}/target/ci/intentionally-missing-dom-compiler`,
    sourceRevision: "0".repeat(40),
    repoRoot: root,
    buildRecipe: LEGACY_BUILD_RECIPE,
  });
  const rows: Array<{
    id: string;
    target: string;
    legacy: { state: string; observation: unknown; error: string };
    native: { state: string; reason: string };
  }> = report.rows;
  const row = rows.find((entry) => entry.id === id);
  assert.ok(row);
  assert.equal(row.target, "dom");
  assert.equal(row.legacy.state, "failed");
  assert.equal(row.legacy.observation, null);
  assert.equal(
    row.legacy.error,
    `${id} requires source-build recipe ${BUILD_RECIPE}; observed ${LEGACY_BUILD_RECIPE}`,
  );
  assert.equal(row.native.state, "unsupported");
  assert.ok(report.summary);
  assert.equal(report.summary.nativeHandled, 0);
  assert.equal(report.summary.nativeEquivalent, 0);
});
