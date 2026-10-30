import { describe, it, before } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
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
const qualifiedEmitProps = 'import("vize:virtual-ts").__EmitProps<';
const qualifiedWidenTemplateRef = 'import("vize:virtual-ts").$WidenTemplateRef';
const currentReceipt = {
  diagnosticSha256: "a9f9ae0f2bbee8294ebd92d300895cf772c49b7f62c20d9018a3d3435d2125dc",
  emitHelperFiles: [],
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
  helperHoistProvenance: "#11049 / ac442c78c8dda5dd05ee3c30049fa2d970bd9bc1",
  provenance:
    "#11034 / 1ba0c04fbcc3df3b4b561b6c2fa95dbf49f17c55, " +
    "#11050 / 3ecd00704e57fdc3bddb06aeb627eda2058aa8ac, " +
    "#11064 / c2ee0878fec5d53edb7c30e10f374e37ab256f02, and " +
    "#11049 / ac442c78c8dda5dd05ee3c30049fa2d970bd9bc1",
  slotFallbackProvenance: "#11064 / c2ee0878fec5d53edb7c30e10f374e37ab256f02",
  slotFallbackFiles: [],
  // Whole packet observed in #11153 / 8b86ad667fc, Actions 41117365474.
  // Prior packet ffd237083004fd1a5eb9b5c8b517ce16c05de9a211310f6aafdc379f1be97dc3
  // differs only in the two ARIA helpers' hygienic key constraints.
  snapshotSha256: "e0d7fd37fead28615939040a3e13dc11b8ecd4209252f99abeb29f8e9657a5f4",
  warningCount: 0,
  widenHelperFiles: [
    "src/ComposablesAndRouting.vue",
    "src/GraphAnd3D.vue",
    "src/MediaAndDataViz.vue",
    "src/NaiveUiOptionsApi.vue",
    "src/UiLibraries.vue",
  ],
} as const;
const unknownSlotFallback =
  "C extends (props: any, context: infer __Ctx, ...args: any[]) => any ? " +
  "NonNullable<__Ctx> extends { slots: infer __S } ? __S : any : unknown;";
const retiredSlotFallbackFiles = [
  "src/ComposablesAndRouting.vue",
  "src/MediaAndDataViz.vue",
  "src/UiLibraries.vue",
] as const;
const componentAriaHelper =
  "function __vizeComponentAriaPropEntries<const R extends keyof any, const C extends keyof any, const T>" +
  "(raw: R, camel: C, value: T): { readonly [K in R | C]: T extends undefined ? never : T } " +
  "{ return { [raw]: value, [camel]: value } as unknown as " +
  "{ readonly [K in R | C]: T extends undefined ? never : T }; }";
const retiredComponentAriaHelpers = [
  "function __vizeComponentAriaPropValue<const T>(value: T): " +
    "T extends undefined ? never : T { return value as (T extends undefined ? never : T); }",
  "function __vizeComponentAriaPropValue<const T>(value: T): " +
    "__VizeNotUndefined<T> { return value as __VizeNotUndefined<T>; }",
] as const;
const ariaPropReceipts = [
  {
    file: "src/PrimeVueOptionsApi.vue",
    priorProperty: '"aria-label": `${selectedAction.label} search`,',
    property:
      '...__vizeComponentAriaPropEntries("aria-label", "ariaLabel", `${selectedAction.label} search`),',
  },
  {
    file: "src/UiLibraries.vue",
    priorProperty: '"aria-label": "PrimeVue input",',
    property: '...__vizeComponentAriaPropEntries("aria-label", "ariaLabel", "PrimeVue input"),',
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
    // Keep the complete source output even if an earlier contract guard fails.
    // The existing Vue-parity artifact retains this exact packet for review.
    const prettyOutput = stringifyDiagnosticSnapshot(parsed, checkConfig.cwd);
    fs.mkdirSync(SNAPSHOT_DIR, { recursive: true });
    fs.writeFileSync(path.join(SNAPSHOT_DIR, `${app.name}-check.snap.actual`), prettyOutput);
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
      `slot fallbacks changed after ${currentReceipt.slotFallbackProvenance}`,
    );
    for (const file of retiredSlotFallbackFiles) {
      const virtualTs = parsed.files.find((candidate) => candidate.file === file)?.virtualTs ?? "";
      assert.equal(
        count(virtualTs, unknownSlotFallback),
        0,
        `${file} must not restore its retired unknown fallback`,
      );
    }
    assert.deepEqual(
      parsed.files
        .filter(({ virtualTs }) => virtualTs?.includes(qualifiedWidenTemplateRef) === true)
        .map(({ file }) => file),
      currentReceipt.widenHelperFiles,
      `qualified widening helpers changed after ${currentReceipt.helperHoistProvenance}`,
    );
    assert.deepEqual(
      parsed.files
        .filter(({ virtualTs }) => virtualTs?.includes(qualifiedEmitProps) === true)
        .map(({ file }) => file),
      currentReceipt.emitHelperFiles,
      `qualified emit helpers changed after ${currentReceipt.helperHoistProvenance}`,
    );
    for (const { file, virtualTs = "" } of parsed.files) {
      assert.equal(
        count(virtualTs, "$WidenTemplateRef"),
        count(virtualTs, qualifiedWidenTemplateRef),
        `${file} must not restore a file-local widening helper`,
      );
      assert.equal(
        count(virtualTs, "__EmitProps<"),
        count(virtualTs, qualifiedEmitProps),
        `${file} must not restore a file-local emit helper`,
      );
      assert.equal(
        count(virtualTs, "type $SlotOutletTarget"),
        0,
        `${file} must not restore the unused slot target alias`,
      );
      assert.equal(
        count(virtualTs, "type $SlotOutletArgs"),
        0,
        `${file} must not restore the unused slot argument alias`,
      );
    }
    for (const file of currentReceipt.widenHelperFiles) {
      const virtualTs = parsed.files.find((candidate) => candidate.file === file)?.virtualTs ?? "";
      assert.equal(
        count(virtualTs, qualifiedWidenTemplateRef),
        1,
        `${file} must use one shared widening helper`,
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
      for (const retired of retiredComponentAriaHelpers) {
        assert.equal(
          count(virtualTs, retired),
          0,
          `${receipt.file} must not restore a retired ARIA value helper`,
        );
      }
      assert.equal(
        count(virtualTs, "T extends undefined ? never : T"),
        2,
        `${receipt.file} must retain the collision-proof narrowing contract`,
      );
      assert.equal(
        count(virtualTs, receipt.property),
        1,
        `${receipt.file} must retain the raw and camel prop witness`,
      );
      assert.equal(
        count(virtualTs, receipt.priorProperty),
        0,
        `${receipt.file} must not restore the kebab prop in the check object`,
      );
    }

    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
    assert.equal(
      sha256(prettyOutput),
      currentReceipt.snapshotSha256,
      `current snapshot needs an intentional successor after ${currentReceipt.provenance}`,
    );
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
