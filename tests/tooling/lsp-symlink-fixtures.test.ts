import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  initializeCapabilities,
  INITIALIZE_CAPABILITIES,
  loadLspManifest,
} from "../differential/lsp-manifest.ts";
import {
  lspWorkspace,
  observeLspWorkspace,
  validateLspWorkspace,
  type LspWorkspaceEvidence,
} from "../differential/lsp-workspace.ts";
import {
  lspSymlinks,
  materializeLspSymlinks,
  observeLspSymlinks,
} from "../differential/lsp-symlinks.ts";
import { validateCapturedLspReport, validateLspReport } from "../differential/lsp-report.ts";
import { referenceReport } from "./support/lsp/differential-reference.ts";

const manifest = fileURLToPath(
  new URL("../_fixtures/differential/lsp/manifest.json", import.meta.url),
);
const loaded = loadLspManifest(manifest);
const candidate = loaded.cases.find(
  (row) => row.id === "lsp/regression/auto-import-quoted-symlink-3952",
);
assert(candidate);
const fixture = candidate;
const target = "owner's-components/MyButton.vue";
const targetSha = "b0ca633eb9dd0a5fbf4ea5598d08ad9f0b49af75169670d2c949df3104267dbb";

void test("explicit symlink metadata keeps every earlier workspace and initialize profile unchanged", () => {
  const before = structuredClone(fixture.data);
  const active = lspWorkspace(fixture);
  assert.deepEqual(active.mapping.symlinks, [
    { path: "app/MyButton.vue", target, sha256: targetSha },
  ]);
  assert.deepEqual(initializeCapabilities(fixture), INITIALIZE_CAPABILITIES);
  assert.deepEqual(fixture.data, before);
  let originals = 0;
  for (const row of loaded.cases) {
    if (row.id === fixture.id) continue;
    originals += 1;
    assert.equal(row.data.workspaceSymlinks, undefined);
    assert.deepEqual(Object.keys(lspWorkspace(row).mapping), [
      "kind",
      "originalFiles",
      "files",
      "configurations",
    ]);
  }
  assert.equal(originals, 78);
});

void test("link layout rejects unknown, unpinned, cyclic, colliding and escaping source metadata", () => {
  const files = lspWorkspace(fixture).mapping.files;
  const valid = { path: "app/MyButton.vue", target };
  assert.deepEqual(lspSymlinks([valid], files), [{ ...valid, sha256: targetSha }]);
  for (const value of [
    null,
    false,
    [],
    [null],
    [{ ...valid, extra: true }],
    [{ path: valid.path }],
    [{ ...valid, path: "../outside.vue" }],
    [{ ...valid, path: "/outside.vue" }],
    [{ ...valid, path: "app\\MyButton.vue" }],
    [{ ...valid, target: "../outside.vue" }],
    [{ ...valid, target: "unknown.vue" }],
    [{ ...valid, path: "app/Page.vue" }],
    [{ ...valid, path: "app" }],
    [{ ...valid, path: "app/Page.vue/nested.vue" }],
    [{ ...valid, path: "unowned/alias.vue" }],
    [valid, valid],
    [valid, { path: "app/MyButton.vue/nested.vue", target }],
    [
      { path: "link-a.vue", target: "link-b.vue" },
      { path: "link-b.vue", target: "link-a.vue" },
    ],
  ]) {
    assert.throws(() => lspSymlinks(value, files), { name: "AssertionError" });
  }
});

function withWorkspace(check: (root: string, evidence: LspWorkspaceEvidence) => void) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-lsp-link-law-")));
  try {
    const active = lspWorkspace(fixture);
    for (const file of active.files) {
      const destination = path.join(root, file.runtimePath);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.writeFileSync(destination, file.bytes);
    }
    materializeLspSymlinks(root, active.mapping.symlinks);
    const evidence: LspWorkspaceEvidence = {
      mapping: active.mapping,
      before: observeLspWorkspace(root, active.mapping),
      after: observeLspWorkspace(root, active.mapping),
      symlinkWorkspaceUri: pathToFileURL(root).href,
      symlinksBefore: observeLspSymlinks(root, active.mapping.symlinks),
      symlinksAfter: observeLspSymlinks(root, active.mapping.symlinks),
    };
    check(root, evidence);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

void test("real link observations bind the pinned regular target before and after to the actual wire URI", () => {
  withWorkspace((root, evidence) => {
    const expected = [
      {
        path: "app/MyButton.vue",
        isSymbolicLink: true,
        targetIsRegularFile: true,
        linkTarget: path.join(root, target),
        canonicalTarget: path.join(root, target),
        sha256: targetSha,
      },
    ];
    assert.deepEqual(evidence.symlinksBefore, expected);
    assert.deepEqual(evidence.symlinksAfter, expected);
    validateLspWorkspace(fixture, evidence, true, pathToFileURL(root).href);
    assert.throws(() =>
      validateLspWorkspace(fixture, evidence, true, pathToFileURL(root + "-foreign").href),
    );
    const wrong = structuredClone(evidence);
    if (wrong.symlinksAfter) wrong.symlinksAfter[0].sha256 = "0".repeat(64);
    assert.throws(() => validateLspWorkspace(fixture, wrong, true, pathToFileURL(root).href));
  });
});

void test("redirected links and equal-byte regular replacements remain actual refusals", () => {
  withWorkspace((root, evidence) => {
    const external = fs.mkdtempSync(path.join(os.tmpdir(), "vize-link-foreign-"));
    try {
      const outside = path.join(external, "MyButton.vue");
      fs.writeFileSync(outside, "external owned control\n");
      const link = path.join(root, "app/MyButton.vue");
      fs.unlinkSync(link);
      fs.symlinkSync(outside, link, "file");
      evidence.symlinksAfter = observeLspSymlinks(root, evidence.mapping.symlinks);
      assert.deepEqual(evidence.symlinksAfter, [
        {
          path: "app/MyButton.vue",
          isSymbolicLink: true,
          targetIsRegularFile: true,
          linkTarget: outside,
          canonicalTarget: fs.realpathSync(outside),
          sha256: null,
        },
      ]);
      assert.throws(() => validateLspWorkspace(fixture, evidence, true, pathToFileURL(root).href));
      fs.unlinkSync(link);
      fs.copyFileSync(path.join(root, target), link);
      evidence.symlinksAfter = observeLspSymlinks(root, evidence.mapping.symlinks);
      assert.equal(evidence.symlinksAfter?.[0].isSymbolicLink, false);
      assert.throws(() => validateLspWorkspace(fixture, evidence, true, pathToFileURL(root).href));
    } finally {
      fs.rmSync(external, { recursive: true, force: true });
    }
  });
});

void test("materialization refuses an existing destination including a dangling link", () => {
  withWorkspace((root, evidence) => {
    assert.throws(
      () => materializeLspSymlinks(root, evidence.mapping.symlinks),
      /destination must be absent/,
    );
    const link = path.join(root, "app/MyButton.vue");
    fs.unlinkSync(link);
    fs.symlinkSync(path.join(root, "missing.vue"), link, "file");
    assert.throws(
      () => materializeLspSymlinks(root, evidence.mapping.symlinks),
      /destination must be absent/,
    );
  });
});

void test("materialization refuses missing, directory and symlink targets instead of following them", () => {
  withWorkspace((root, evidence) => {
    const targetPath = path.join(root, target);
    fs.unlinkSync(path.join(root, "app/MyButton.vue"));
    fs.unlinkSync(targetPath);
    assert.throws(() => materializeLspSymlinks(root, evidence.mapping.symlinks), {
      code: "ENOENT",
    });
    fs.mkdirSync(targetPath);
    assert.throws(
      () => materializeLspSymlinks(root, evidence.mapping.symlinks),
      /target must be a regular file/,
    );
    fs.rmdirSync(targetPath);
    fs.symlinkSync(path.join(root, "app/Page.vue"), targetPath, "file");
    assert.throws(
      () => materializeLspSymlinks(root, evidence.mapping.symlinks),
      /target must be a regular file/,
    );
  });
});

const syntheticBuild = {
  sourceRevision: "4".repeat(40),
  binaryPath: "target/ci/vize",
  binarySha256: "5".repeat(64),
  cliVersion: "vize synthetic validator fixture",
};

void test("optional capture receives a detached whole copy and absent capture never clones", () => {
  const report = referenceReport(loaded, syntheticBuild);
  const original = structuredClone(report);
  let observed = false;
  validateCapturedLspReport(loaded, report, syntheticBuild, (copy) => {
    observed = true;
    assert.deepEqual(copy, original);
    Reflect.set(copy, "sourceRevision", "0".repeat(40));
    Reflect.set(copy.summary, "legacyFailures", 900);
  });
  assert.equal(observed, true);
  assert.deepEqual(report, original);
  // A pure sentinel detects a clone attempt; it is not a product/wire packet.
  Object.defineProperty(report, "cloneSentinel", { value: () => undefined, enumerable: true });
  validateCapturedLspReport(loaded, report, syntheticBuild);
});

void test("capture failure fails otherwise valid evidence but never replaces validator or product first cause", () => {
  const captureError = new Error("owned capture write failed");
  let calls = 0;
  const failCapture = () => {
    calls += 1;
    throw captureError;
  };
  assert.throws(
    () =>
      validateCapturedLspReport(
        loaded,
        referenceReport(loaded, syntheticBuild),
        syntheticBuild,
        failCapture,
      ),
    (error) => error === captureError,
  );
  assert.equal(calls, 1);
  const invalid = referenceReport(loaded, syntheticBuild);
  invalid.sourceRevision = "0".repeat(40);
  let primary: assert.AssertionError | undefined;
  try {
    validateLspReport(loaded, invalid, syntheticBuild);
  } catch (error) {
    assert(error instanceof assert.AssertionError);
    primary = error;
  }
  assert(primary);
  const originalPrimary = primary;
  assert.throws(
    () => validateCapturedLspReport(loaded, invalid, syntheticBuild, failCapture),
    (error) => {
      assert(error instanceof assert.AssertionError);
      assert.equal(error.message, originalPrimary.message);
      assert.deepEqual(error.actual, originalPrimary.actual);
      assert.deepEqual(error.expected, originalPrimary.expected);
      assert.equal(error.operator, originalPrimary.operator);
      return true;
    },
  );
  assert.equal(calls, 2);
  const failed = referenceReport(loaded, syntheticBuild);
  const row = failed.rows.find((row) => row.id === fixture.id);
  assert(row);
  row.legacy.state = "failed";
  row.legacy.verdict = "failed";
  row.legacy.error = "original product failure remains first";
  failed.summary.legacyMatches -= 1;
  failed.summary.legacyFailures += 1;
  const summary = validateCapturedLspReport(loaded, failed, syntheticBuild, failCapture);
  assert.equal(summary.legacyFailures, 1);
  assert.equal(row.legacy.error, "original product failure remains first");
  assert.equal(calls, 3);
});
