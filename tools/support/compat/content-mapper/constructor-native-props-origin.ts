/** Opt-in same-snapshot metadata observation; never constructor or diagnostic admission. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { readFileSync, realpathSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { asArray, asRecord, asString, parseJson, readJsonRecord } from "../../json-value.ts";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { fileImage, inside } from "../../release/computed_event_custody.ts";
import { loadOriginal27 } from "../../release/constructor_original27_public/fixtures.ts";
import { capture, completed } from "../../release/constructor_original27_public/capture.ts";

const controllerRoot = fileURLToPath(new URL("../../../../", import.meta.url));
const rustPath =
  "crates/vize_canon/src/batch/type_checker/tests/recent_issues/constructor_native_props_origin";
const sourceFiles = [`${rustPath}.rs`, `${rustPath}/query.rs`];
const git = (root: string, ...args: string[]) =>
  execFileSync("git", args, { cwd: root }).toString("utf8").trim();

function sourceIdentity(root: string, expected: string) {
  assert.match(expected, /^[a-f0-9]{40}$/u);
  assert.equal(git(root, "rev-parse", "HEAD"), expected);
  assert.equal(git(root, "status", "--porcelain"), "");
  for (const path of sourceFiles)
    assert.deepEqual(readFileSync(join(root, path)), readFileSync(join(controllerRoot, path)));
  return {
    source: expected,
    tree: git(root, "rev-parse", "HEAD^{tree}"),
    files: sourceFiles.map((path) => fileImage(join(root, path))),
  };
}

function build(root: string, expected: string, record: Recorder) {
  const source = sourceIdentity(root, expected);
  const rows = readFileSync(join(record.root, "origin-cargo.jsonl"), "utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => asRecord(parseJson(line)));
  assert.deepEqual(rows.at(-1), { reason: "build-finished", success: true });
  const artifacts = rows.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      asRecord(row.target).name === "vize_canon" &&
      asArray(asRecord(row.target).kind).includes("lib") &&
      asRecord(row.profile).test === true &&
      row.executable,
  );
  assert.equal(artifacts.length, 1);
  assert.equal(artifacts[0].fresh, false);
  record.json("origin-build.json", {
    ...source,
    executable: fileImage(asString(artifacts[0].executable)),
    cargoArtifact: artifacts[0],
    argv: [
      "cargo",
      "test",
      "--locked",
      "--profile",
      "ci-opt",
      "-p",
      "vize_canon",
      "--lib",
      "--no-run",
      "--message-format=json",
    ],
    controller: {
      source: git(controllerRoot, "rev-parse", "HEAD"),
      script: fileImage(fileURLToPath(import.meta.url)),
    },
    productionOverlay: false,
    nativeExecuted: false,
    timingUsable: false,
  });
}

async function run(
  root: string,
  expected: string,
  fixtureRoot: string,
  provider: string,
  record: Recorder,
) {
  const source = sourceIdentity(root, expected),
    original = await loadOriginal27(fixtureRoot);
  const row = original.inputs.rows.filter(
    (value) => value.id === "constructor-public-props-discrepancy",
  );
  assert.equal(row.length, 1);
  const rust = readFileSync(join(root, `${rustPath}.rs`), "utf8");
  for (const [name, body] of [
    ["COMPONENT", row[0].component],
    ["TINY", row[0].source],
    ["CONFIG", original.inputs.files.get("tsconfig.json")!],
  ] as const) {
    const match = new RegExp(`const ${name}: &str = r#"([\\s\\S]*?)"#;`, "u").exec(rust);
    assert.equal(match?.[1], body, `frozen ${name} literal`);
  }
  assert.deepEqual(readJsonRecord(join(provider, "package.json")), original.provider);
  const modules = realpathSync(join(provider, "node_modules"));
  const resolver = join(modules, "typescript-native/lib/getExePath.js");
  const resolveNative = asRecord(await import(pathToFileURL(resolver).href));
  assert.equal(typeof resolveNative.default, "function");
  const native = realpathSync(
    asString(Reflect.apply(resolveNative.default as () => unknown, undefined, [])),
  );
  assert.ok(inside(modules, native));
  const nativeImage = fileImage(native);
  assert.equal(nativeImage.bytes, original.inputs.historical.checker.bytes);
  assert.equal(nativeImage.sha256, original.inputs.historical.checker.sha256);
  const official = join(modules, "vue-tsc/bin/vue-tsc.js");
  assert.equal(
    realpathSync(createRequire(official).resolve("typescript/package.json")),
    realpathSync(join(modules, "typescript/package.json")),
  );
  const versions = Object.fromEntries(
    ["vue", "typescript", "typescript-native", "vue-tsc"].map((name) => [
      name,
      readJsonRecord(join(modules, name, "package.json")).version,
    ]),
  );
  assert.deepEqual(versions, {
    vue: "3.5.42",
    typescript: "6.0.3",
    "typescript-native": "7.1.0-beta",
    "vue-tsc": "3.3.13",
  });
  const providerImages = original.inputs.historical.identities
    .filter((value) =>
      /^node_modules\/(?:vue|typescript|typescript-native|vue-tsc)\//u.test(value.file),
    )
    .map((value) => {
      const image = fileImage(join(provider, value.file));
      assert.equal(image.bytes, value.bytes);
      assert.equal(image.sha256, value.sha256);
      return image;
    });
  const buildReceipt = readJsonRecord(join(record.root, "origin-build.json"));
  assert.equal(buildReceipt.source, source.source);
  assert.equal(buildReceipt.tree, source.tree);
  const executable = asRecord(buildReceipt.executable);
  assert.deepEqual(fileImage(asString(executable.path)), executable);
  record.json("origin-inputs-and-provider.json", {
    ...source,
    fixtureCustody: original.custody,
    frozenCase: row[0],
    originalConfig: original.inputs.files.get("tsconfig.json"),
    versions,
    native: nativeImage,
    providerImages,
    resolver: fileImage(resolver),
    lockRawBase64: readFileSync(join(provider, "package-lock.json")).toString("base64"),
    publishedVizeExecuted: false,
    responseScope: "complete SDK typed DTOs; not whole transport frames",
  });
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    CORSA_PATH: native,
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_NATIVE_PROPS_ORIGIN_MODULES: modules,
    VIZE_NATIVE_PROPS_ORIGIN_OUTPUT: join(record.root, "native-origin.json"),
    NATIVE_ORIGIN_SOURCE: source.source,
    NATIVE_ORIGIN_TREE: source.tree,
  };
  delete env.VIZE_TEST_DISABLE_TSGO;
  const result = capture(
    record,
    "native-origin-unit",
    [
      asString(executable.path),
      "batch::type_checker::tests::recent_issues::constructor_native_props_origin::",
      "--nocapture",
      "--test-threads=1",
    ],
    root,
    env,
  );
  completed(result, 0, false);
  assert.match(result.stdout.toString("utf8"), /^running 3 tests$/mu);
  assert.match(result.stdout.toString("utf8"), /test result: ok\. 3 passed; 0 failed; 0 ignored;/u);
  const receipt = readJsonRecord(join(record.root, "native-origin.json"));
  assert.equal(receipt.source, source.source);
  assert.equal(receipt.tree, source.tree);
  assert.equal(receipt.nativeAdmission, false);
  assert.equal(receipt.diagnosticsRequested, false);
  assert.deepEqual(fileImage(asString(executable.path)), executable);
  assert.deepEqual(fileImage(native), nativeImage);
  record.json("origin-complete.json", {
    source,
    native: nativeImage,
    receipt: fileImage(join(record.root, "native-origin.json")),
    metadataProbeCompleted: true,
    nativeAdmission: false,
    diagnosticRepair: false,
    publicAcceptance: false,
    flagInterpretation: "UNPROVED",
    testProcessPid: result.pid,
    nativeProcessPidClaim: false,
    quiescenceClaim: false,
  });
}

async function main() {
  const [mode, rawRoot, source, fixtureRoot, provider, evidence] = process.argv.slice(2);
  assert.ok(rawRoot && source && fixtureRoot && provider && evidence);
  const root = realpathSync(rawRoot),
    record = new Recorder(resolve(evidence));
  if (mode === "build") build(root, source, record);
  else {
    assert.equal(mode, "run");
    await run(root, source, fixtureRoot, provider, record);
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
