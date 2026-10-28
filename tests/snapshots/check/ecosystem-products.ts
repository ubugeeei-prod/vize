import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { ecosystemProductsApp, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";
import {
  runVizeCheckJson,
  stringifyDiagnosticSnapshot,
  type VizeCheckFileJson,
  type VizeCheckJson,
} from "../../_helpers/vize-check.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = ecosystemProductsApp;
const currentReceipt = {
  diagnosticSha256: "a9f9ae0f2bbee8294ebd92d300895cf772c49b7f62c20d9018a3d3435d2125dc",
  errorCount: 2,
  fileCount: 9,
  files: [
    "src/ComposablesAndRouting.vue",
    "src/GraphAnd3D.vue",
    "src/MediaAndDataViz.vue",
    "src/NaiveUiOptionsApi.vue",
    "src/PrimeVueOptionsApi.vue",
    "src/UiLibraries.vue",
    "src/types.ts",
    "src/vendors/nuxt-ui-component.ts",
    "src/vendors/vue-select.ts",
  ],
  provenance:
    "#11034 / 1ba0c04fbcc3df3b4b561b6c2fa95dbf49f17c55 and " +
    "#11050 / 3ecd00704e57fdc3bddb06aeb627eda2058aa8ac",
  slotFallbackProvenance: "#11034 / 1ba0c04fbcc3df3b4b561b6c2fa95dbf49f17c55",
  slotFallbackFiles: [
    "src/ComposablesAndRouting.vue",
    "src/MediaAndDataViz.vue",
    "src/UiLibraries.vue",
  ],
  snapshotSha256: "d9f554ddad89c5e88c22b7baebb2152ef964668930af0700a0a8844eb5b97e69",
  warningCount: 0,
} as const;
const unknownSlotFallback =
  "C extends (props: any, context: infer __Ctx, ...args: any[]) => any ? " +
  "NonNullable<__Ctx> extends { slots: infer __S } ? __S : any : unknown;";
const componentAriaHelper =
  "function __vizeComponentAriaPropValue<const T>(value: T): " +
  "__VizeNotUndefined<T> { return value as __VizeNotUndefined<T>; }";
const ariaPropReceipts = [
  {
    file: "src/PrimeVueOptionsApi.vue",
    priorProperty: '"aria-label": `${selectedAction.label} search`,',
    property: '"ariaLabel": __vizeComponentAriaPropValue(`${selectedAction.label} search`),',
  },
  {
    file: "src/UiLibraries.vue",
    priorProperty: '"aria-label": "PrimeVue input",',
    property: '"ariaLabel": __vizeComponentAriaPropValue("PrimeVue input"),',
  },
] as const;

function normalizeEnvironmentDependentDiagnostics(parsed: VizeCheckJson): VizeCheckJson {
  const files = parsed.files.map((file: VizeCheckFileJson) => ({
    ...file,
    diagnostics: file.diagnostics.filter((diagnostic: string) => {
      if (/^hint:.* \[TS638[57]\] /.test(diagnostic)) {
        return false;
      }
      return !/^error:.* \[TS2882\] Cannot find module or type declarations for side-effect import/.test(
        diagnostic,
      );
    }),
  }));

  return {
    ...parsed,
    files,
    errorCount: files.reduce((count: number, file: any) => {
      return (
        count +
        file.diagnostics.filter((diagnostic: string) => diagnostic.startsWith("error:")).length
      );
    }, 0),
    warningCount: files.reduce((count: number, file: any) => {
      return (
        count +
        file.diagnostics.filter((diagnostic: string) => diagnostic.startsWith("warning:")).length
      );
    }, 0),
  };
}

describe(`${app.name} check (type checker)`, () => {
  before(requireVizeAndCorsaBins);

  it("vize check does not crash and snapshot matches", () => {
    const checkConfig = app.check!;
    const parsed = normalizeEnvironmentDependentDiagnostics(
      runVizeCheckJson(checkConfig.cwd, checkConfig.patterns, {
        showVirtualTs: true,
        timeoutMs: 300_000,
      }),
    );
    console.log(`fileCount=${parsed.fileCount}, errorCount=${parsed.errorCount}`);
    assert.ok(parsed.fileCount > 0, "fileCount should be > 0");
    assert.deepEqual(
      [parsed.fileCount, parsed.errorCount, parsed.warningCount],
      [currentReceipt.fileCount, currentReceipt.errorCount, currentReceipt.warningCount],
      `diagnostic counts changed after ${currentReceipt.provenance}`,
    );
    assert.deepEqual(
      parsed.files.map(({ file }) => file),
      currentReceipt.files,
      `diagnostic files changed after ${currentReceipt.provenance}`,
    );
    assert.equal(
      sha256(diagnosticReceipt(parsed)),
      currentReceipt.diagnosticSha256,
      `diagnostics changed after ${currentReceipt.provenance}`,
    );
    assert.deepEqual(
      parsed.files
        .filter(({ virtualTs }) => virtualTs?.includes(unknownSlotFallback) === true)
        .map(({ file }) => file),
      currentReceipt.slotFallbackFiles,
      `the exact three any-to-unknown slot fallbacks from ${currentReceipt.slotFallbackProvenance} changed`,
    );
    for (const file of currentReceipt.slotFallbackFiles) {
      const virtualTs = parsed.files.find((candidate) => candidate.file === file)?.virtualTs ?? "";
      assert.equal(
        count(virtualTs, unknownSlotFallback),
        1,
        `${file} must own one unknown fallback`,
      );
    }
    assert.deepEqual(
      parsed.files
        .filter(({ virtualTs }) => virtualTs?.includes(componentAriaHelper) === true)
        .map(({ file }) => file),
      ariaPropReceipts.map(({ file }) => file),
      "the exact two #11050 ARIA prop projections changed",
    );
    for (const receipt of ariaPropReceipts) {
      const virtualTs =
        parsed.files.find((candidate) => candidate.file === receipt.file)?.virtualTs ?? "";
      assert.equal(count(virtualTs, componentAriaHelper), 1, `${receipt.file} must own one helper`);
      assert.equal(
        count(virtualTs, receipt.property),
        1,
        `${receipt.file} must use the camel prop`,
      );
      assert.equal(
        count(virtualTs, receipt.priorProperty),
        0,
        `${receipt.file} must not restore the kebab prop in the check object`,
      );
    }

    const prettyOutput = stringifyDiagnosticSnapshot(parsed, checkConfig.cwd);
    assert.equal(
      sha256(prettyOutput),
      currentReceipt.snapshotSha256,
      `current snapshot needs an intentional successor after ${currentReceipt.provenance}`,
    );
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});

function count(haystack: string, needle: string): number {
  return haystack.split(needle).length - 1;
}

function diagnosticReceipt(parsed: VizeCheckJson): string {
  return JSON.stringify(parsed.files.map(({ diagnostics, file }) => ({ file, diagnostics })));
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}
