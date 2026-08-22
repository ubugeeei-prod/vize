import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const completeCheckSnapshotTests = [
  "ant-design-vue",
  "compiler-macros",
  "ecosystem-products",
  "elk",
  "misskey",
  "npmx",
  "nuxt-ui",
  "reka-ui",
  "style-preprocessors",
  "typecheck-errors",
  "vue2-elm",
  "vuefes",
].sort();

const assertionOnlyCheckTests = {
  "class-component": "class-component vue-tsc parity has known upstream-noisy diagnostics",
  "class-component-lsp-oracle":
    "class-component LSP oracle asserts exact TS2551 member diagnostics across document versions",
  "create-vue-editor-range-oracle":
    "editor-range oracle asserts exact authored Vue ranges for live LSP editor features",
  "create-vue-generated-template-oracle":
    "generated-template patch oracle asserts exact batch typechecker and vue-tsc parity",
  "create-vue-patch-oracle":
    "patch oracle asserts exact live CLI and LSP behavior across document versions",
  "element-plus-slot-oracle":
    "UI library patch oracle asserts global component slot types across editor revisions",
  "javascript-sfc-checkjs-oracle":
    "checkJs oracle asserts exact per-case diagnostics and vue-tsc agreement on JavaScript SFCs",
  "jsx-intrinsic-globals-oracle":
    "JSX oracle builds a throwaway workspace and asserts exact intrinsic-global cleanliness plus strict component prop diagnostics",
  "nuxt-ui-ambient-oracle":
    "framework patch oracle asserts generated Nuxt ambient and #imports virtual-module types across editor revisions",
  "nuxt-no-tsconfig-oracle":
    "framework patch oracle asserts clean, broken, and repaired Nuxt checking without a root tsconfig",
  "nuxt-template-globals":
    "template-global oracle asserts exact vue-tsc agreement on declared, undeclared, and repaired Nuxt instance globals",
  "nuxt-vue-module-augmentations":
    'module-augmentation oracle asserts exact vue-tsc agreement on generated, project, and package `declare module "vue"` globals',
  "pinia-generic-store-oracle":
    "library patch oracle asserts generic store inference and dependency refresh behavior",
  "template-ref-unwrap-oracle":
    "ref-unwrap oracle builds throwaway workspaces and asserts exact vue-tsc parity plus identical diagnostics for imported and auto-imported composables",
  "typescript-go-module-resolution-determinism":
    "module-resolution determinism gate asserts the pinned tsgo build and byte-identical output across fresh processes",
  "typescript-project-references-oracle":
    "solution tsconfig oracle asserts referenced-project CLI and LSP diagnostic parity",
  "vue-benchmarks-correctness-plants":
    "upstream benchmark plants assert exact clean, broken, repaired, and vue-tsc parity",
  "vue-benchmarks-lsp-ref-unwrap-oracle":
    "LSP probe asserts exact backend-liveness diagnostics and rejects heuristic hover answers",
  "vue-benchmarks-scaled-corpus-plants":
    "scaled corpus plants assert every planted diagnostic survives full-corpus re-validation",
  "vue-router-patch-oracle":
    "library patch oracle asserts exact package-resolution behavior across document versions",
  "vue-router-dmts-oracle":
    "declaration patch oracle asserts .d.mts resolution and dependent diagnostic refresh",
  "vue-router-formatter-oracle":
    "formatter oracle asserts exact fmt convergence, repair, idempotence, and compiled output",
  "vitepress-theme-oracle":
    "framework patch oracle asserts package exports and theme declaration refresh behavior",
  "vue-element-admin-legacy-oracle":
    "legacy project patch oracle asserts Vue 2 slot-scope and filter typecheck behavior",
  "vue-element-admin-legacy-lsp-oracle":
    "legacy LSP oracle asserts exact Vue 2 template diagnostics and CLI agreement per version",
  "vue-element-admin-unmapped-diagnostic-oracle":
    "unmapped-diagnostic oracle asserts every published range stays addressable in the document",
  "vue2-class-component-oracle":
    "legacy class-component oracle asserts exact clean, broken, repeated, and repaired diagnostics",
  directus: "real-world smoke lane is too large for a deterministic complete baseline",
  "element-plus": "real-world smoke lane is too large for a deterministic complete baseline",
  "frontend-phpcon": "real-world smoke lane is too large for a deterministic complete baseline",
  "generic-build": "generic build fixture asserts targeted compiler behavior",
  hoppscotch: "real-world smoke lane is too large for a deterministic complete baseline",
  "naive-ui": "real-world smoke lane is too large for a deterministic complete baseline",
  "nuxt-parity": "parity lane asserts focused framework behavior",
  "options-api": "options-api fixture asserts focused parity behavior",
  "options-api-inherited-members":
    "inherited-members fixture asserts exact mixins/extends diagnostics and vue-tsc agreement",
  primevue: "covered by the complete ecosystem-products baseline",
  "toolchain-parity": "parity lane asserts focused vue-tsc agreement",
  "typecheck-vue-imports": "fixture asserts focused import-resolution behavior",
  voicevox: "real-world smoke lane is too large for a deterministic complete baseline",
  "vue-vben-admin": "real-world smoke lane is too large for a deterministic complete baseline",
  vuetify: "real-world smoke lane is too large for a deterministic complete baseline",
  "zz-intentional-errors-fixtures":
    "intentional-error aggregate asserts exact broken diagnostics and complete clean repair",
  "zz-intentional-errors-realworld":
    "intentional-error aggregate asserts exact broken diagnostics and complete clean repair",
} satisfies Record<string, string>;

function snapshotFiles(...segments: string[]): string[] {
  const directory = path.join(root, ...segments);
  return fs
    .readdirSync(directory)
    .filter((file) => file.endsWith(".snap"))
    .sort()
    .map((file) => path.join(directory, file));
}

function readJsonSnapshot(file: string): unknown {
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function checkSnapshotTestNames(): string[] {
  return fs
    .readdirSync(path.join(root, "tests", "snapshots", "check"))
    .filter((file) => file.endsWith(".ts"))
    .map((file) => file.replace(/\.ts$/, ""))
    .sort();
}

test("check snapshot tests declare whether they use complete baselines", () => {
  const complete = completeCheckSnapshotTests;
  const assertionOnly = Object.keys(assertionOnlyCheckTests).sort();
  const declared = [...complete, ...assertionOnly].sort();
  assert.deepEqual(declared, checkSnapshotTestNames());

  for (const [name, reason] of Object.entries(assertionOnlyCheckTests)) {
    assert.ok(reason.length >= 20, `${name}: assertion-only reason should be explicit`);
  }

  for (const name of complete) {
    const source = fs.readFileSync(
      path.join(root, "tests", "snapshots", "check", `${name}.ts`),
      "utf8",
    );
    assert.match(source, /assertSnapshot\(/, `${name}: expected a complete snapshot assertion`);
  }
});

test("check snapshots are complete JSON baselines", () => {
  for (const snapshot of snapshotFiles("tests", "snapshots", "check", "__snapshots__")) {
    const data = readJsonSnapshot(snapshot);

    assert.ok(data && typeof data === "object" && !Array.isArray(data), snapshot);
    const baseline = data as {
      files?: unknown[];
      fileCount?: unknown;
      errorCount?: unknown;
      warningCount?: unknown;
    };

    assert.ok(Array.isArray(baseline.files), snapshot);
    assert.equal(baseline.fileCount, baseline.files.length, snapshot);
    assert.equal(typeof baseline.errorCount, "number", snapshot);
    assert.equal(typeof baseline.warningCount, "number", snapshot);

    const severities = new Map<string, number>();
    for (const file of baseline.files) {
      assert.ok(file && typeof file === "object", snapshot);
      const entry = file as { file?: unknown; virtualTs?: unknown; diagnostics?: unknown };
      assert.equal(typeof entry.file, "string", snapshot);
      assert.ok(entry.virtualTs === undefined || typeof entry.virtualTs === "string", snapshot);
      assert.ok(Array.isArray(entry.diagnostics), snapshot);

      for (const diagnostic of entry.diagnostics as unknown[]) {
        assert.equal(typeof diagnostic, "string", snapshot);
        const severity = /^(error|warning|info|hint):/.exec(diagnostic as string)?.[1];
        assert.ok(severity, `${snapshot}: unrecognized severity in ${String(diagnostic)}`);
        severities.set(severity, (severities.get(severity) ?? 0) + 1);
      }
    }

    // The summary counters are derived from the rows, so a baseline edit that
    // drops a row must drop the count with it. Two PRs each removing one
    // diagnostic and each writing `30 -> 29` merged cleanly into a wrong `29`
    // (#4239 and #4241 both landed before the other's CI could observe it),
    // which only surfaced on the next PR to run against both. `info` and
    // `hint` rows are deliberately counted by neither field — `vize check`
    // reports only errors and warnings in its summary.
    assert.equal(
      severities.get("error") ?? 0,
      baseline.errorCount,
      `${snapshot}: errorCount must equal the number of error rows`,
    );
    assert.equal(
      severities.get("warning") ?? 0,
      baseline.warningCount,
      `${snapshot}: warningCount must equal the number of warning rows`,
    );
  }
});

test("lint snapshots include rule documentation and consistent message counts", () => {
  for (const snapshot of snapshotFiles("tests", "snapshots", "lint", "__snapshots__")) {
    const data = readJsonSnapshot(snapshot);

    assert.ok(Array.isArray(data), snapshot);
    assert.ok(data.length > 0, snapshot);

    for (const entry of data as Array<{
      file?: unknown;
      messages?: unknown[];
      errorCount?: unknown;
      warningCount?: unknown;
    }>) {
      assert.equal(typeof entry.file, "string", snapshot);
      assert.ok(Array.isArray(entry.messages), snapshot);
      assert.equal(typeof entry.errorCount, "number", snapshot);
      assert.equal(typeof entry.warningCount, "number", snapshot);

      let errors = 0;
      let warnings = 0;
      for (const message of entry.messages as Array<{
        ruleId?: unknown;
        ruleDocsPath?: unknown;
        message?: unknown;
        severity?: unknown;
      }>) {
        const ruleDocsPath = message.ruleDocsPath;
        assert.equal(typeof message.ruleId, "string", snapshot);
        if (typeof ruleDocsPath !== "string") {
          assert.fail(`${snapshot}: missing ruleDocsPath`);
        }
        assert.equal(typeof message.message, "string", snapshot);
        assert.ok(
          fs.existsSync(path.join(root, ruleDocsPath)),
          `${snapshot}: missing ${ruleDocsPath}`,
        );

        if (message.severity === 2) {
          errors++;
        } else if (message.severity === 1) {
          warnings++;
        } else {
          assert.fail(`${snapshot}: unexpected severity ${String(message.severity)}`);
        }
      }

      assert.equal(entry.errorCount, errors, snapshot);
      assert.equal(entry.warningCount, warnings, snapshot);
    }
  }
});
