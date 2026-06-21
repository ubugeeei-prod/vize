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
  "vuefes",
].sort();

const assertionOnlyCheckTests = {
  "class-component": "class-component vue-tsc parity has known upstream-noisy diagnostics",
  directus: "real-world smoke lane is too large for a deterministic complete baseline",
  "element-plus": "real-world smoke lane is too large for a deterministic complete baseline",
  "frontend-phpcon": "real-world smoke lane is too large for a deterministic complete baseline",
  "generic-build": "generic build fixture asserts targeted compiler behavior",
  hoppscotch: "real-world smoke lane is too large for a deterministic complete baseline",
  "naive-ui": "real-world smoke lane is too large for a deterministic complete baseline",
  "nuxt-parity": "parity lane asserts focused framework behavior",
  "options-api": "options-api fixture asserts focused parity behavior",
  primevue: "covered by the complete ecosystem-products baseline",
  "toolchain-parity": "parity lane asserts focused vue-tsc agreement",
  "typecheck-vue-imports": "fixture asserts focused import-resolution behavior",
  voicevox: "real-world smoke lane is too large for a deterministic complete baseline",
  "vue-vben-admin": "real-world smoke lane is too large for a deterministic complete baseline",
  vuetify: "real-world smoke lane is too large for a deterministic complete baseline",
  "zz-intentional-errors-fixtures": "intentional-error aggregate asserts diagnostic presence",
  "zz-intentional-errors-realworld": "intentional-error aggregate asserts diagnostic presence",
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

    for (const file of baseline.files) {
      assert.ok(file && typeof file === "object", snapshot);
      const entry = file as { file?: unknown; virtualTs?: unknown; diagnostics?: unknown };
      assert.equal(typeof entry.file, "string", snapshot);
      assert.ok(entry.virtualTs === undefined || typeof entry.virtualTs === "string", snapshot);
      assert.ok(Array.isArray(entry.diagnostics), snapshot);
    }
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
