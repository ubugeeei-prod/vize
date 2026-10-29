/** Bounded public-vs-source slot triage. Original carrier inputs and the failure oracle stay frozen. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadAdmissionAPI } from "./canon-helper-admission-api.ts";
import { prepareHelperArchitectureCarrier } from "./canon-template-helper-carrier.ts";
import { prepareHelperProvider, sourceMapperSdk } from "./canon-helper-baseline-provider.ts";
import { nativeDiagnostics } from "./canon-helper-baseline-observe.ts";
import { packet, isRecord, jsonValue } from "./canon-instance-directives-model.ts";
import { identify, hash, run, save } from "./canon-instance-directives-io.ts";

const tiny = `<script setup lang="ts">
import WitnessChild from '../component';
type Assert<T extends true> = T;
type IsAny<T> = 0 extends 1 & T ? true : false;
type IsNever<T> = [T] extends [never] ? true : false;
type IsUnknown<T> = IsAny<T> extends true ? false : unknown extends T ? keyof T extends never ? true : false : false;
type Exact<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
type SlotReport<T> = IsAny<T> extends true ? 'any' : IsNever<T> extends true ? 'never' : IsUnknown<T> extends true ? 'unknown' : Exact<T, number> extends true ? 'number' : 'other';
function slotReport<T>(_value: T): SlotReport<T> { throw new Error('type witness'); }
function expectSlotNumber(_tag: 'number'): void {}
type PublicSlotContract = Assert<Exact<Parameters<InstanceType<typeof WitnessChild>['$slots']['default']>[0], { item: number }>>;
</script>
<template>
  <WitnessChild :value="1">
    <template #default="{ item }">{{ expectSlotNumber(slotReport(item)) }}</template>
  </WitnessChild>
</template>
`;
const realComponent = `import { defineComponent, type SlotsType, type VNode } from 'vue';
const VueChild = defineComponent({
  props: { value: { type: Number, required: true } },
  slots: Object as SlotsType<{ default(props: { item: number }): VNode[] }>,
});
export declare const WitnessChild: { new(): InstanceType<typeof VueChild> };
export default WitnessChild;
`;
const config = {
  compilerOptions: {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    module: "ESNext",
    moduleResolution: "Bundler",
    target: "ES2024",
    lib: ["ES2024", "DOM"],
    types: [],
    incremental: false,
  },
  include: ["src/**/*.vue", "component.ts"],
};
async function main() {
  const [providerArg, binaryArg, regenerationArg, driverArg, outputArg] = process.argv.slice(2);
  assert.ok(providerArg && binaryArg && regenerationArg && driverArg && outputArg);
  const provider = path.resolve(providerArg),
    binary = path.resolve(binaryArg),
    regeneration = path.resolve(regenerationArg),
    output = path.resolve(outputArg);
  fs.mkdirSync(output);
  const { checker } = await prepareHelperProvider(provider, binary, regeneration, output);
  const { api } = await loadAdmissionAPI(path.resolve(driverArg));
  const frozen = path.join(output, "frozen");
  const corpus = prepareHelperArchitectureCarrier(
    frozen,
    provider,
    {
      sfcs: 1,
      sfcBytes: 2048,
      graphNodes: 8,
      shape: "diamond",
      typeWidth: 4,
      typeDepth: 4,
      stage: "valid",
      allowLarge: true,
    },
    api,
  );
  const component = fs.readFileSync(path.join(frozen, "component.ts"), "utf8");
  assert.equal(hash(component), "351ea130f8a93fbbeccc889e3315351f8f32c8e0f46a326ad2c272a7a6ffbddd");
  assert.equal(
    hash(fs.readFileSync(path.join(frozen, "src/Scale00000.vue"))),
    "4e24a7b527ff84a6417d358a17bf3a22c92fd3d749d309866b8ee2659eaea6b8",
  );
  save(path.join(output, "frozen-corpus.json"), corpus);
  const nativePackage = path.join(
    provider,
    "node_modules/@vizejs/native-linux-x64-gnu/package.json",
  );
  const metadata = jsonValue(fs.readFileSync(nativePackage, "utf8"));
  assert.ok(
    isRecord(metadata) && metadata.version === "0.602.0" && typeof metadata.main === "string",
  );
  const publicBinary = path.join(provider, "node_modules/vize/bin/vize");
  save(path.join(output, "published-image.json"), {
    role: "genuine public SDK default mapper exec; source override absent",
    publicEntrypoint: identify(publicBinary),
    nativePackage: identify(nativePackage),
    nativeImage: identify(path.resolve(path.dirname(nativePackage), metadata.main)),
  });
  const mapperArgs = [
    "--runExternalCode",
    "--checkers",
    "1",
    "--noEmit",
    "--pretty",
    "false",
    "--project",
    "tsconfig.mapper.json",
  ];
  const full = run(output, "frozen-public-mapper", checker, mapperArgs, frozen);
  const fullRows = nativeDiagnostics(full.stdout, full.status, frozen);
  save(path.join(output, "frozen-public-diagnostics.json"), fullRows);
  const fullFailed = fullRows.length > 0;
  const rows: unknown[] = [];
  const reportToken = "slotReport(item)";
  const reportLine = tiny.slice(0, tiny.indexOf(reportToken)).split("\n").length;
  const reportColumn = tiny.slice(0, tiny.indexOf(reportToken)).split("\n").at(-1)!.length + 1;
  for (const route of ["public-mapper", "source-mapper", "public-cli", "source-cli"] as const) {
    const project = path.join(output, route),
      src = path.join(project, "src");
    fs.mkdirSync(src, { recursive: true });
    fs.symlinkSync(path.join(provider, "node_modules"), path.join(project, "node_modules"), "dir");
    const authored =
      "import { h, type Component } from 'vue';\nimport WitnessChild from './component';\nconst allowed: Component = WitnessChild;\nh(WitnessChild, { value: 1 });\nvoid allowed;\n";
    fs.writeFileSync(path.join(project, "authored.ts"), authored);
    save(path.join(project, "tsconfig.authored.json"), {
      ...config,
      include: ["authored.ts", "component.ts"],
    });
    save(path.join(project, "tsconfig.json"), config);
    save(path.join(project, "tsconfig.mapper.json"), {
      ...config,
      contentMappers: [{ package: "vize", extensions: [".vue"] }],
    });
    fs.writeFileSync(path.join(src, "Tiny.vue"), tiny);
    fs.writeFileSync(path.join(project, "component.ts"), realComponent);
    let numberDiagnostics: unknown[] | null = null;
    if (route === "source-mapper")
      save(
        path.join(project, "source-sdk.json"),
        sourceMapperSdk(project, provider, [binary, "content-mapper"]),
      );
    if (route === "public-mapper") {
      const nativeLegal = run(
        output,
        "authored-native",
        checker,
        ["--checkers", "1", "--noEmit", "--pretty", "false", "--project", "tsconfig.authored.json"],
        project,
      );
      assert.equal(nativeLegal.status, 0);
      assert.equal(nativeLegal.stdout.length, 0);
      const stock = run(
        output,
        "authored-stock-ts6",
        process.execPath,
        [
          path.join(provider, "node_modules/typescript/lib/tsc.js"),
          "--pretty",
          "false",
          "--project",
          "tsconfig.authored.json",
        ],
        project,
      );
      assert.equal(stock.status, 0);
      assert.equal(stock.stdout.length, 0);
    }
    for (const item of ["number", "any", "unknown", "never", "string", "repaired"] as const) {
      const actualType = item === "repaired" ? "number" : item;
      const source = realComponent.replace(
        "props: { item: number }",
        "props: { item: " + actualType + " }",
      );
      assert.notEqual(source, "", "a missing mutation must not pass");
      if (actualType !== "number") assert.notEqual(source, realComponent);
      fs.writeFileSync(path.join(project, "component.ts"), source);
      const isCli = route.endsWith("-cli");
      const program =
        route === "source-cli" ? binary : route === "public-cli" ? publicBinary : checker;
      const args = isCli
        ? [
            "check",
            "--no-config",
            "--format",
            "json",
            "--show-virtual-ts",
            "--tsconfig",
            "tsconfig.json",
            "--corsa-path",
            checker,
          ]
        : mapperArgs;
      const result = run(output, route + "/" + item, program, args, project);
      const diagnostics = isCli
        ? packet(result.stdout, result.status, project).diagnostics
        : nativeDiagnostics(result.stdout, result.status, project);
      if (item === "number") numberDiagnostics = diagnostics;
      if (item === "repaired") {
        assert.ok(numberDiagnostics, route + " must retain its number-stage diagnostics");
        assert.deepEqual(
          diagnostics,
          numberDiagnostics,
          route + " repaired diagnostics must exactly restore the number-stage baseline",
        );
      }
      const precisionProbe = diagnostics.filter(
        (row) =>
          row.file === "src/Tiny.vue" &&
          row.line === reportLine &&
          row.column === reportColumn &&
          row.code === 2345,
      );
      const shouldPoison = actualType !== "number";
      assert.equal(
        precisionProbe.length,
        shouldPoison ? 1 : 0,
        route +
          ":" +
          item +
          " inferred slot precision must respond to an independent provider mutation",
      );
      const entry = {
        route,
        item,
        originalTiny: hash(tiny),
        component: hash(source),
        diagnostics,
        precisionProbe,
        timingUsable: false,
      };
      rows.push(entry);
      save(path.join(output, route, item, "acceptance.json"), entry);
    }
    assert.equal(fs.readFileSync(path.join(project, "component.ts"), "utf8"), realComponent);
  }
  save(path.join(output, "acceptance.json"), {
    frozenInputRetained: true,
    originalAccuracyOracle: [],
    publicOriginalDiagnostics: fullRows,
    publicOriginalPassed: !fullFailed,
    slotPrecisionPoisonAndRepair: true,
    rows,
    interpretation:
      "an observed public failure is a reproduction, not an accepted replacement accuracy oracle",
    timingUsable: false,
  });
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
