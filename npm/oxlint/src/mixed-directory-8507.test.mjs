// Additive exact-source regression; historical public BEFORE runs are separate.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../native/scripts/formatter-history-build.mjs";
import { snapshot, plainHostEnvironment, readEvents } from "./test-support/html-cli-oracles.mjs";
import { presentationInputs } from "./cli/presentation-context.ts";
import {
  authorities,
  assertEvents,
  expectedStockVueRow,
  judgeCli,
  judgeNative,
} from "./test-support/mixed-directory-8507-assertions.mjs";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const repository = path.resolve(packageDir, "../..");
const corpus = path.join(
  repository,
  "tests/_fixtures/differential/lint/oxlint-mixed-directory-8507",
);
const plan = JSON.parse(fs.readFileSync(path.join(corpus, "controls.json")));
const engine = fs.realpathSync(process.env.VIZE_OXLINT_TEST_ENTRYPOINT);
const wrapper = fs.realpathSync(process.env.VIZE_OXLINT_SOURCE_BIN);
const capturePath = process.env.VIZE_OXLINT_MIXED_CAPTURE;
const workerIndex = Number(process.env.VIZE_OXLINT_MIXED_WORKER);
assert.ok(["0", "1"].includes(process.env.VIZE_OXLINT_MIXED_WORKER));
const custody = JSON.parse(fs.readFileSync(process.env.VIZE_OXLINT_NATIVE_CUSTODY));
assert.equal(custody.schema, "vize.oxlint.source-native");
assert.equal(custody.version, 1);
assert.equal(process.env.GITHUB_ACTIONS, "true");
assert.equal(custody.source.head, process.env.GITHUB_SHA);
assert.equal(wrapper, fs.realpathSync(path.join(packageDir, "dist/cli.mjs")));
assert.ok(capturePath && process.env.NODE_OPTIONS, "source observer and complete capture required");
const nativeDir = path.join(repository, "npm/native");
const buildReceiptBytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
const buildReceipt = JSON.parse(buildReceiptBytes);
validateNativeHistoryBuild(nativeDir, buildReceipt);
assert.deepEqual(custody.source, buildReceipt.source);
assert.deepEqual(custody.toolchain, buildReceipt.toolchain);
assert.equal(custody.binary.sha256, buildReceipt.frozen.sha256);
assert.ok(
  fs
    .readFileSync(path.join(path.dirname(custody.binary.path), "build-receipt.json"))
    .equals(buildReceiptBytes),
  "staged current-source build receipt must be byte exact",
);
const native = createRequire(import.meta.url)(path.join(nativeDir, "index.js"));
assert.equal(typeof native.lintOxlintHtml, "function");
const driverLoad = readEvents(custody.calls).filter(
  (event) => event.kind === "load" && event.pid === process.pid,
);
assert.ok(driverLoad.length, "driver must actually load the authenticated staged addon");
for (const event of driverLoad) {
  assert.deepEqual(event.source, custody.source);
  assert.equal(event.binary, custody.binary.path);
  assert.equal(event.binarySha256, custody.binary.sha256);
}
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
assert.equal(hash(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
assert.equal(fs.realpathSync(custody.binary.path), custody.binary.path);
const providerRoot = path.resolve(path.dirname(engine), "..");
const metadata = JSON.parse(fs.readFileSync(path.join(providerRoot, "package.json")));
assert.equal(metadata.name, "oxlint");
assert.equal(metadata.version, "1.81.0");
assert.equal(engine, fs.realpathSync(path.join(providerRoot, metadata.bin.oxlint)));
const providerFiles = () =>
  snapshot(providerRoot).map(([file, kind, bytes]) =>
    kind === "file" ? [file, kind, hash(Buffer.from(bytes, "base64"))] : [file, kind, bytes],
  );
const providerBefore = providerFiles();
const providerNative = [];
for (const name of Object.keys(metadata.optionalDependencies ?? {})) {
  let entrypoint;
  try {
    entrypoint = createRequire(engine).resolve(name);
  } catch (error) {
    if (error.code === "MODULE_NOT_FOUND") continue;
    throw error;
  }
  const file = fs.realpathSync(entrypoint),
    packageFile = path.join(path.dirname(file), "package.json");
  const packageBytes = fs.readFileSync(packageFile),
    dependency = JSON.parse(packageBytes);
  assert.equal(dependency.name, name);
  assert.equal(dependency.version, "1.81.0");
  providerNative.push({
    name,
    file,
    sha256: hash(fs.readFileSync(file)),
    packageFile,
    packageBytes: Array.from(packageBytes),
  });
}
assert.ok(providerNative.length, "actual pinned Oxlint native package required");
const sourcePackageJson = path.join(packageDir, "package.json");
const sourcePackageBytes = Array.from(fs.readFileSync(sourcePackageJson));
const sourceDistFiles = () =>
  snapshot(path.join(packageDir, "dist")).map(([file, kind, bytes]) =>
    kind === "file" ? [file, kind, hash(Buffer.from(bytes, "base64"))] : [file, kind, bytes],
  );
const sourceDistBefore = sourceDistFiles();
const wrapperSha256 = hash(fs.readFileSync(wrapper));
const environment = plainHostEnvironment(process.env);
const retainedEnvironment = Object.fromEntries(
  [
    ...presentationInputs,
    "NODE_OPTIONS",
    "VIZE_OXLINT_NATIVE_CUSTODY",
    "VIZE_PREFER_WORKSPACE_BINDING",
  ].map((key) => [key, environment[key] ?? null]),
);
const capture = {
  complete: false,
  workerIndex,
  source: custody.source,
  custody,
  engine,
  wrapper,
  wrapperSha256,
  sourcePlugin: {
    packageJson: sourcePackageJson,
    packageBytes: sourcePackageBytes,
    distFiles: sourceDistBefore,
  },
  providerFiles: providerBefore,
  resolvedProviderNativeCandidates: providerNative,
  environment: retainedEnvironment,
  inputs: snapshot(corpus),
  initialEvents: readEvents(custody.calls),
  observations: [],
  passed: { cli: 0, native: 0 },
};
fs.mkdirSync(path.dirname(capturePath), { recursive: true });
const save = () => fs.writeFileSync(capturePath, JSON.stringify(capture, null, 2) + "\n");
const wholeError = (error) =>
  error instanceof Error
    ? {
        name: error.name,
        ...Object.fromEntries(
          Object.getOwnPropertyNames(error).map((key) => [key, wholeError(error[key])]),
        ),
      }
    : error;
save();
function materialize(fixture) {
  const workspace = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-mixed-8507-")));
  const cwd = path.join(workspace, "repro");
  for (let ancestor = workspace; ; ancestor = path.dirname(ancestor)) {
    for (const marker of [".git", ".jj"]) assert.ok(!fs.existsSync(path.join(ancestor, marker)));
    if (path.dirname(ancestor) === ancestor) break;
  }
  const copy = (file, source) => {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.copyFileSync(path.join(corpus, source), file);
  };
  for (const file of plan.baseFiles) copy(path.join(cwd, file), file);
  for (const [file, source] of Object.entries(fixture.overlay ?? {}))
    copy(path.join(cwd, file), source);
  for (const [file, source] of Object.entries(fixture.parentFiles ?? {}))
    copy(path.join(workspace, file), source);
  for (const directory of fixture.directories ?? [])
    fs.mkdirSync(path.join(cwd, directory), { recursive: true });
  if (fixture.surface === "cli") {
    fs.mkdirSync(path.join(cwd, "node_modules"), { recursive: true });
    fs.symlinkSync(providerRoot, path.join(cwd, "node_modules/oxlint"), "dir");
    fs.symlinkSync(packageDir, path.join(cwd, "node_modules/oxlint-plugin-vize"), "dir");
  }
  return { workspace, cwd };
}

function observe(entrypoint, argv, cwd, workspace, fixture, format, surface) {
  const before = snapshot(workspace),
    index = readEvents(custody.calls).length;
  const sourceAuthorities = authorities(cwd);
  const result = spawnSync(process.execPath, [entrypoint, ...argv], {
    cwd,
    env: environment,
    timeout: 60_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const record = {
    fixture,
    format,
    surface,
    executable: process.execPath,
    entrypoint,
    argv,
    cwd,
    environment: retainedEnvironment,
    before,
    after: snapshot(workspace),
    sourceAuthorities,
    sourceAuthoritiesAfter: authorities(cwd),
    status: result.status,
    signal: result.signal,
    error: wholeError(result.error ?? null),
    stdoutBytes: Array.from(result.stdout ?? []),
    stderrBytes: Array.from(result.stderr ?? []),
    events: readEvents(custody.calls).slice(index),
  };
  capture.observations.push(record);
  save();
  assert.equal(record.signal, null);
  assert.equal(record.error, null);
  assert.deepEqual(record.after, record.before);
  assert.deepEqual(record.sourceAuthoritiesAfter, record.sourceAuthorities);
  assertEvents(custody);
  return record;
}

function observeNative(fixture, cwd, workspace) {
  const rootJson = path.join(cwd, ".oxlintrc.json"),
    before = snapshot(workspace);
  const options = {
    cwd,
    literalTarget: fixture.target,
    rootJson,
    rootBytes: Array.from(fs.readFileSync(rootJson)),
    hostProfile: "1.81.0",
    noIgnore: fixture.noIgnore ?? false,
    cliIgnorePatterns: [],
    customIgnoreFilename: ".eslintignore",
    format: "json",
    presentation: {
      cwd,
      graphicalTheme: "plain",
      links: false,
      width: 400,
      stylishNoColor: true,
      stylishRelative: true,
    },
  };
  const index = readEvents(custody.calls).length;
  const record = {
    fixture: fixture.name,
    surface: "native",
    cwd,
    options,
    before,
    sourceAuthorities: authorities(cwd),
    result: null,
    error: null,
  };
  capture.observations.push(record);
  save();
  try {
    record.result = native.lintOxlintHtml(options);
  } catch (error) {
    record.error = wholeError(error);
  }
  record.after = snapshot(workspace);
  record.events = readEvents(custody.calls).slice(index);
  save();
  record.sourceAuthoritiesAfter = authorities(cwd);
  save();
  judgeNative(fixture, record, cwd, workspace, custody);
}

try {
  let cellIndex = 0;
  for (const fixture of plan.cases) {
    const formats = fixture.surface === "cli" ? ["implicit-default", ...plan.formats] : ["json"];
    for (const format of formats) {
      if (cellIndex++ % 2 !== workerIndex) continue;
      const { cwd, workspace } = materialize(fixture);
      try {
        if (fixture.surface === "cli") {
          const argv = [
            ...fixture.argv,
            ...(format === "implicit-default" ? [] : ["--format", format]),
          ];
          const stock = observe(engine, argv, cwd, workspace, fixture.name, format, "stock");
          const vue = fixture.expectedDiagnostics.filter((finding) =>
            finding.path.endsWith(".vue"),
          );
          assert.equal(stock.status, vue.length ? 1 : 0);
          assert.deepEqual(stock.stderrBytes, []);
          if (format === "json")
            assert.deepEqual(
              JSON.parse(Buffer.from(stock.stdoutBytes).toString("utf8")).diagnostics,
              vue.map((finding) => expectedStockVueRow(finding, cwd)),
            );
          const record = observe(wrapper, argv, cwd, workspace, fixture.name, format, "wrapper");
          judgeCli(fixture, format, record, cwd, engine);
          capture.passed.cli++;
        } else {
          observeNative(fixture, cwd, workspace);
          capture.passed.native++;
        }
        save();
      } finally {
        fs.rmSync(workspace, { recursive: true, force: true });
      }
    }
  }
  assert.deepEqual(
    capture.passed,
    [
      { cli: 23, native: 4 },
      { cli: 22, native: 4 },
    ][workerIndex],
  );
  assert.deepEqual(providerFiles(), providerBefore);
  for (const file of providerNative) {
    assert.equal(hash(fs.readFileSync(file.file)), file.sha256);
    assert.deepEqual(Array.from(fs.readFileSync(file.packageFile)), file.packageBytes);
  }
  assert.deepEqual(snapshot(corpus), capture.inputs);
  assert.deepEqual(Array.from(fs.readFileSync(sourcePackageJson)), sourcePackageBytes);
  assert.deepEqual(sourceDistFiles(), sourceDistBefore);
  assert.equal(hash(fs.readFileSync(wrapper)), wrapperSha256);
  assert.equal(hash(fs.readFileSync(custody.binary.path)), custody.binary.sha256);
  capture.complete = true;
  save();
} catch (error) {
  capture.failure = wholeError(error);
  save();
  throw error;
}
