/** Bind every original #7949 control to genuine current-source CLI/native/editor bytes. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { spawnSync } from "node:child_process";

const root = process.env.VIZE_VUE_HELPER_CAPTURE;
assert(root, "the required native helper capture is absent");
const cases = [
  "editor-root-vue-absent",
  "editor-root-vue-present",
  "invalid-model",
  "original-root-vue-absent",
  "original-root-vue-present",
  "valid-disabled",
];
assert.deepEqual(
  readdirSync(root, { withFileTypes: true })
    .filter((e) => e.isDirectory())
    .map((e) => e.name)
    .sort(),
  cases,
);
const hash = (file) => createHash("sha256").update(readFileSync(file)).digest("hex");
const json = (file) => JSON.parse(readFileSync(file, "utf8"));
const fixture = "tests/_fixtures/differential/typechecker/config-scoped-vue-helpers";
const contract = json(join(fixture, "contract.json"));
assert.equal(contract.issue, 7949);
assert.equal(contract.vueVersion, "3.6.0-rc.10");
assert.equal(contract.nativeVersion, "7.0.2");
const captureFiles = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? captureFiles(join(dir, e.name)) : [join(dir, e.name)],
  );
let nativeBinary;
let cliBinary;
let cliControls = 0;
let nativeControls = 0;
let editorControls = 0;
for (const name of cases) {
  const directory = join(root, name);
  const runtime = json(join(directory, "runtime.json"));
  assert.equal(runtime.sourceSha, process.env.SOURCE_SHA);
  const actualNative = realpathSync(runtime.nativeBinary);
  nativeBinary ??= actualNative;
  assert.equal(actualNative, nativeBinary);
  for (const file of contract.originalFiles) {
    const expected = readFileSync(join(fixture, file + ".txt"));
    const actual = readFileSync(join(directory, "inputs", file));
    if (file === "package.json" && name.endsWith("present")) {
      assert.equal(
        actual.toString(),
        '{ "name": "root", "private": true, "dependencies": { "vue": "3.6.0-rc.10" } }\n',
      );
    } else if (
      file === "apps/web/Toggle.vue" &&
      ["valid-disabled", "invalid-model"].includes(name)
    ) {
      assert.equal(actual.toString(), expected.toString().replace('"42"', '"true"'));
    } else if (file === "apps/web/Counter.vue" && name === "invalid-model") {
      assert.equal(actual.toString(), expected.toString().replace("count++", "count = 'x'"));
    } else {
      assert.deepEqual(actual, expected, name + "/" + file);
    }
  }
  for (const pkg of [
    "vue",
    "@vue/runtime-dom",
    "@vue/runtime-core",
    "@vue/reactivity",
    "@vue/shared",
    "@vue/compiler-dom",
    "@vue/compiler-core",
    "@vue/compiler-sfc",
    "@vue/runtime-vapor",
  ]) {
    const manifest = json(join(directory, "packages", pkg, "package.json"));
    assert.equal(manifest.name, pkg);
    assert.equal(manifest.version, contract.vueVersion);
    assert(readFileSync(join(directory, "packages", pkg, manifest.types)).length > 0);
  }
  if (name.startsWith("editor-")) {
    const configured = json(join(directory, "vize.config.json"));
    assert.equal(realpathSync(configured.typeChecker.corsaPath), nativeBinary);
    for (const file of ["App", "Counter", "Toggle", "Toggle-valid"]) {
      const result = json(join(directory, file + ".diagnostics.json"));
      if (file === "Toggle") {
        assert.deepEqual(result, [
          {
            range: { start: { line: 1, character: 11 }, end: { line: 1, character: 19 } },
            severity: 1,
            code: 2322,
            source: "vize/types",
            message: "Type '42' is not assignable to type 'Booleanish | undefined'.",
          },
        ]);
      } else {
        assert.deepEqual(result, []);
      }
      editorControls++;
    }
    continue;
  }
  assert.equal(
    json(join(directory, "native.runtime.json")).exitCode,
    name === "valid-disabled" ? 0 : 2,
  );
  assert.equal(readFileSync(join(directory, "native.stderr.txt"), "utf8"), "");
  const nativeExpected =
    name === "valid-disabled"
      ? ""
      : name === "invalid-model"
        ? "oracle.ts(3,1): error TS2322: Type 'string' is not assignable to type 'number'.\n"
        : "oracle.ts(4,58): error TS2322: Type 'number' is not assignable to type 'Booleanish | undefined'.\n";
  assert.equal(
    readFileSync(join(directory, "native.stdout.txt"), "utf8").replaceAll("\r\n", "\n"),
    nativeExpected,
  );
  nativeControls++;
  const modes = name.startsWith("original-")
    ? ["explicit", "default", "sharded"]
    : ["explicit", "default"];
  for (const mode of modes) {
    const cli = json(join(directory, mode + ".runtime.json"));
    const actualCli = realpathSync(cli.cliBinary);
    cliBinary ??= actualCli;
    assert.equal(actualCli, cliBinary);
    assert.equal(cli.exitCode, name === "valid-disabled" ? 0 : 1);
    const result = json(join(directory, mode + ".stdout.txt"));
    assert.equal(result.fileCount, 5);
    assert.equal(result.errorCount, name === "valid-disabled" ? 0 : 1);
    assert.equal(result.warningCount, 0);
    assert.equal(result.files.length, 5);
    const generated = join(directory, mode, "generated");
    const config = json(join(generated, "apps/web/tsconfig.json"));
    assert(config.include.includes("../../apps/web/__vize_helpers.d.ts"));
    assert(!config.include.includes("../../__vize_helpers.d.ts"));
    const helpers = readFileSync(join(generated, "apps/web/__vize_helpers.d.ts"), "utf8");
    assert(helpers.includes("typeof import('vue')") || helpers.includes('typeof import("vue")'));
    cliControls++;
  }
}
assert.equal(cliControls, 10);
assert.equal(nativeControls, 4);
assert.equal(editorControls, 8);
let packageRoot = dirname(nativeBinary);
while (!existsSync(join(packageRoot, "package.json")) && dirname(packageRoot) !== packageRoot)
  packageRoot = dirname(packageRoot);
const nativePackagePath = join(packageRoot, "package.json");
const nativePackage = json(nativePackagePath);
assert.equal(nativePackage.name, `@typescript/typescript-${process.platform}-${process.arch}`);
assert.equal(nativePackage.version, "7.0.2");
const version = spawnSync(nativeBinary, ["--version"], { encoding: "utf8" });
assert.equal(version.status, 0);
assert.equal(version.stdout, "Version 7.0.2\n");
assert.equal(version.stderr, "");
writeFileSync(
  join(root, "receipt.json"),
  JSON.stringify(
    {
      sourceSha: process.env.SOURCE_SHA,
      cliBinary,
      cliSha256: hash(cliBinary),
      nativeBinary,
      nativeSha256: hash(nativeBinary),
      nativePackageSha256: hash(nativePackagePath),
      nativePackage: { name: nativePackage.name, version: nativePackage.version },
      vueVersion: contract.vueVersion,
      cliControls,
      nativeControls,
      editorControls,
      originalFixtureFiles: contract.originalFiles.map((file) => ({
        file,
        sha256: hash(join(fixture, file + ".txt")),
      })),
      completeInputsAndOutputs: captureFiles(root).map((file) => ({
        path: file.slice(root.length + 1),
        sha256: hash(file),
      })),
    },
    null,
    2,
  ) + "\n",
);
