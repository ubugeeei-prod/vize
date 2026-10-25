// CI-only attribution: execute both immutable original sources without changing their tests.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

assert.equal(process.env.GITHUB_ACTIONS, "true", "Use the isolated Actions source observer");
const root = process.cwd();
const capture = process.env.VIZE_PACKAGE_REFRESH_CAPTURE;
assert(capture);
fs.mkdirSync(capture, { recursive: true });
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const save = (name, value) =>
  fs.writeFileSync(path.join(capture, name), JSON.stringify(value, null, 2) + "\n");
const state = (result) => ({
  status: result.status,
  signal: result.signal,
  error: result.error?.message ?? null,
  stdout: result.stdout ?? "",
  stderr: result.stderr ?? "",
});
const observer = git("rev-parse", "HEAD");
const producerFile =
  "crates/vize_canon/src/corsa_bridge/vue_document_package_config_tests/extensionless_ts_alias.rs";
const producerParent = "crates/vize_canon/src/corsa_bridge/vue_document_package_config_tests.rs";
const producerSource = fs.readFileSync(path.join(root, producerFile));
const test =
  "corsa_bridge::vue_document_package_config_tests::native_editor_resolves_package_alias_from_monorepo_root";
const testFiles = [
  "crates/vize_canon/src/corsa_bridge/vue_document_package_config_tests.rs",
  "crates/vize_canon/src/corsa_bridge/outside_import_editor_tests.rs",
];
const sourceFiles = [
  ...testFiles,
  "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt",
  "crates/vize_canon/src/batch/virtual_project/tsconfig_gen.rs",
  "crates/vize_canon/src/batch/virtual_project/editor_projection.rs",
  "crates/vize_canon/src/batch/import_rewriter/project_mirror.rs",
  "crates/vize_canon/src/batch/import_rewriter/virtual_alias.rs",
  "crates/vize_canon/src/batch/import_rewriter_alias.rs",
  "crates/vize_canon/src/corsa_bridge/vue_dependencies_alias/context.rs",
];
const refs = [
  ["before-fix", "62652a4d4029fa11016dfa1ac8c24669b42c8d7a"],
  ["after-fix", observer],
];
const receipt = {
  schema: "vize.corsa-alias-10753-source-discovery",
  version: 1,
  observer,
  run: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  attributionOnly: true,
  sourceQualification: false,
  note: "The original strict assertion remains unchanged. Observed assertion failures give no source acceptance credit.",
  results: [],
};
save("attempt.json", receipt);
const sdk = path.join(capture, "sdk");
fs.mkdirSync(sdk, { recursive: true });
fs.writeFileSync(path.join(sdk, "package.json"), '{"private":true}\n');
const npm = path.resolve(path.dirname(process.execPath), "../lib/node_modules/npm/bin/npm-cli.js");
const installArgs = [
  npm,
  "install",
  "--ignore-scripts",
  "--no-audit",
  "--no-fund",
  "--save-exact",
  "@typescript/typescript-linux-x64@7.0.2",
];
const install = spawnSync(process.execPath, installArgs, {
  cwd: sdk,
  encoding: "utf8",
  timeout: 120_000,
  maxBuffer: 8_000_000,
});
save("sdk-install.json", {
  command: process.execPath,
  args: installArgs,
  cwd: sdk,
  ...state(install),
});
assert.equal(install.error, undefined);
assert.equal(install.status, 0, install.stderr);
const executable = fs.realpathSync(
  path.join(sdk, "node_modules/@typescript/typescript-linux-x64/lib/tsc"),
);
const image = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
const version = () => {
  assert.equal(hash(executable), image, "Both sources must use one unchanged physical SDK image");
  const result = spawnSync(executable, ["--version"], { encoding: "utf8", timeout: 10_000 });
  assert.deepEqual(state(result), {
    status: 0,
    signal: null,
    error: null,
    stdout: "Version 7.0.2\n",
    stderr: "",
  });
  return state(result);
};
receipt.sdk = {
  executable,
  sha256: image,
  package: JSON.parse(
    fs.readFileSync(
      path.join(sdk, "node_modules/@typescript/typescript-linux-x64/package.json"),
      "utf8",
    ),
  ),
  lockSha256: hash(path.join(sdk, "package-lock.json")),
  version: version(),
};
const strace = fs.realpathSync(execFileSync("which", ["strace"], { encoding: "utf8" }).trim());
receipt.traceObserver = {
  executable: strace,
  sha256: hash(strace),
  version: execFileSync(strace, ["--version"], { encoding: "utf8" }),
};
save("receipt.json", receipt);
for (const [label, source] of refs) {
  git("checkout", "--detach", source);
  assert.equal(git("rev-parse", "HEAD"), source);
  assert.equal(git("diff", "--name-only"), "", "Observe untouched source and original assertions");
  const dir = path.join(capture, label);
  fs.mkdirSync(dir);
  const sourceInventory = sourceFiles.map((file) => {
    const copy = path.join(dir, "source", file);
    fs.mkdirSync(path.dirname(copy), { recursive: true });
    fs.copyFileSync(path.join(root, file), copy);
    return {
      path: file,
      blob: git("rev-parse", source + ":" + file),
      bytes: fs.statSync(copy).size,
      sha256: hash(copy),
    };
  });
  const row = {
    label,
    source,
    originalSourceFiles: sourceInventory,
    sdkExecutable: executable,
    sdkSha256Before: hash(executable),
    sdkVersion: version(),
  };
  receipt.results.push(row);
  save("receipt.json", receipt);
  const buildArgs = [
    "test",
    "--lib",
    "--no-run",
    "--message-format=json",
    "--locked",
    "--profile",
    "ci",
    "-p",
    "vize_canon",
  ];
  const build = spawnSync("cargo", buildArgs, {
    cwd: root,
    env: process.env,
    encoding: "utf8",
    timeout: 900_000,
    maxBuffer: 64_000_000,
  });
  fs.writeFileSync(path.join(dir, "build.stdout"), build.stdout ?? "");
  fs.writeFileSync(path.join(dir, "build.stderr"), build.stderr ?? "");
  row.build = {
    command: "cargo",
    args: buildArgs,
    status: build.status,
    signal: build.signal,
    error: build.error?.message ?? null,
    stdoutSha256: hash(path.join(dir, "build.stdout")),
    stderrSha256: hash(path.join(dir, "build.stderr")),
  };
  save("receipt.json", receipt);
  assert.equal(build.error, undefined);
  assert.equal(build.status, 0, build.stderr);
  const artifacts = build.stdout
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter(
      (item) =>
        item.reason === "compiler-artifact" &&
        item.target.name === "vize_canon" &&
        item.profile.test &&
        item.executable,
    );
  assert.equal(artifacts.length, 1);
  const binary = fs.realpathSync(artifacts[0].executable);
  row.binary = {
    path: binary,
    sha256: hash(binary),
    bytes: fs.statSync(binary).size,
    cargoArtifact: artifacts[0],
  };
  assert.equal(git("diff", "--name-only"), "");
  const sourceCapture = path.join(dir, "existing-capture");
  const args = [
    "-c",
    'ulimit -f 524288; exec "$@"',
    "vize-alias-trace",
    strace,
    "-ff",
    "-q",
    "-ttt",
    "-T",
    "-xx",
    "-s",
    "16777216",
    "-e",
    "trace=process,read,write,readv,writev,openat,close,connect,socket,dup,dup2,dup3",
    "-o",
    path.join(dir, "native.trace"),
    binary,
    "--exact",
    test,
    "--nocapture",
  ];
  const env = {
    ...process.env,
    CORSA_PATH: executable,
    VIZE_TEST_TSGO_PATH: executable,
    VIZE_TYPECHECK_REGRESSION_CAPTURE: sourceCapture,
  };
  delete env.VIZE_TEST_DISABLE_TSGO;
  const result = spawnSync("bash", args, {
    cwd: root,
    env,
    encoding: "utf8",
    timeout: 180_000,
    maxBuffer: 32_000_000,
  });
  fs.writeFileSync(path.join(dir, "test.stdout"), result.stdout ?? "");
  fs.writeFileSync(path.join(dir, "test.stderr"), result.stderr ?? "");
  row.execution = {
    command: "bash",
    args,
    cwd: root,
    selectedEnvironment: {
      CORSA_PATH: executable,
      VIZE_TEST_TSGO_PATH: executable,
      VIZE_TYPECHECK_REGRESSION_CAPTURE: sourceCapture,
      VIZE_TEST_DISABLE_TSGO: null,
    },
    ...state(result),
    originalAssertionPassed: result.status === 0,
  };
  row.sdkSha256After = hash(executable);
  row.originalSourceUnchanged =
    sourceInventory.every((file) => hash(path.join(root, file.path)) === file.sha256) &&
    git("diff", "--name-only") === "";
  row.traces = fs
    .readdirSync(dir)
    .filter((file) => file.startsWith("native.trace."))
    .sort()
    .map((file) => ({
      path: file,
      bytes: fs.statSync(path.join(dir, file)).size,
      sha256: hash(path.join(dir, file)),
    }));
  save("receipt.json", receipt);
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(row.sdkSha256After, image);
  assert.equal(row.originalSourceUnchanged, true);
  assert.match(result.stdout, /running 1 test/);
  assert(
    row.traces.length > 0 && row.traces.every((file) => file.bytes < 512 * 1024 * 1024),
    "No truncated native stream is complete evidence",
  );
  const packetRoot = path.join(sourceCapture, "editor-monorepo-alias");
  const inventory = (directory) =>
    fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) =>
      entry.isDirectory()
        ? inventory(path.join(directory, entry.name))
        : [
            {
              path: path.relative(dir, path.join(directory, entry.name)),
              bytes: fs.statSync(path.join(directory, entry.name)).size,
              sha256: hash(path.join(directory, entry.name)),
            },
          ],
    );
  for (const file of [
    "diagnostics.json",
    "generated.vue.ts",
    "generated-tsconfig.json",
    "runtime.json",
  ])
    assert(
      fs.statSync(path.join(packetRoot, file)).size > 0,
      "Missing pre-assert capture: " + file,
    );
  const runtime = JSON.parse(fs.readFileSync(path.join(packetRoot, "runtime.json"), "utf8"));
  assert.equal(runtime.nativeBinary, executable);
  row.completeDiagnostics = JSON.parse(
    fs.readFileSync(path.join(packetRoot, "diagnostics.json"), "utf8"),
  );
  row.capturedFiles = inventory(sourceCapture);
  assert(
    row.capturedFiles.some(
      (file) => file.path.includes("/inputs/") && file.path.endsWith("App.vue"),
    ),
  );
  if (label === "before-fix") {
    // Original native observation above ran against an entirely untouched tree.
    // Add only this closed test observer to exercise the same producer on stock source.
    const parentPath = path.join(root, producerParent);
    const childPath = path.join(root, producerFile);
    const original = fs.readFileSync(parentPath);
    assert(!fs.existsSync(childPath));
    assert(!original.toString().includes("mod extensionless_ts_alias;"));
    const overlay = Buffer.concat([original, Buffer.from("\nmod extensionless_ts_alias;\n")]);
    const overlayDir = path.join(dir, "test-only-edge-overlay");
    fs.mkdirSync(overlayDir);
    fs.writeFileSync(path.join(overlayDir, "parent-before.rs"), original);
    fs.writeFileSync(path.join(overlayDir, "parent-observed.rs"), overlay);
    fs.writeFileSync(path.join(overlayDir, "extensionless_ts_alias.rs"), producerSource);
    row.beforeEdgeOverlay = {
      parent: producerParent,
      child: producerFile,
      productionSource: source,
      testOnly: true,
    };
    try {
      fs.mkdirSync(path.dirname(childPath), { recursive: true });
      fs.writeFileSync(parentPath, overlay);
      fs.writeFileSync(childPath, producerSource);
      row.beforeEdgeOverlay.parentSha256 = hash(parentPath);
      row.beforeEdgeOverlay.childSha256 = hash(childPath);
      const built = spawnSync("cargo", buildArgs, {
        cwd: root,
        env: process.env,
        encoding: "utf8",
        timeout: 900_000,
        maxBuffer: 64_000_000,
      });
      fs.writeFileSync(path.join(overlayDir, "build.stdout"), built.stdout ?? "");
      fs.writeFileSync(path.join(overlayDir, "build.stderr"), built.stderr ?? "");
      row.beforeEdgeOverlay.build = {
        command: "cargo",
        args: buildArgs,
        status: built.status,
        signal: built.signal,
        error: built.error?.message ?? null,
      };
      save("receipt.json", receipt);
      assert.equal(built.error, undefined);
      assert.equal(built.status, 0, built.stderr);
      const products = built.stdout
        .split("\n")
        .filter(Boolean)
        .map((line) => JSON.parse(line))
        .filter(
          (item) =>
            item.reason === "compiler-artifact" &&
            item.target.name === "vize_canon" &&
            item.profile.test &&
            item.executable,
        );
      assert.equal(products.length, 1);
      const overlayBinary = fs.realpathSync(products[0].executable);
      row.beforeEdgeOverlay.binary = {
        path: overlayBinary,
        sha256: hash(overlayBinary),
        cargoArtifact: products[0],
      };
      const args = [
        "corsa_bridge::vue_document_package_config_tests::extensionless_ts_alias::",
        "--nocapture",
      ];
      const checked = spawnSync(overlayBinary, args, {
        cwd: root,
        env: { ...env, VIZE_EDITOR_ALIAS_CAPTURE: path.join(dir, "producer-capture") },
        encoding: "utf8",
        timeout: 180_000,
        maxBuffer: 32_000_000,
      });
      fs.writeFileSync(path.join(overlayDir, "test.stdout"), checked.stdout ?? "");
      fs.writeFileSync(path.join(overlayDir, "test.stderr"), checked.stderr ?? "");
      row.beforeEdgeOverlay.execution = { command: overlayBinary, args, ...state(checked) };
      assert.equal(checked.error, undefined);
      assert.equal(checked.signal, null);
      assert.match(checked.stdout, /running 3 tests/);
    } finally {
      fs.writeFileSync(parentPath, original);
      if (fs.existsSync(childPath)) fs.unlinkSync(childPath);
      row.beforeEdgeOverlay.restoredOriginalSource =
        hash(parentPath) === sourceInventory.find((file) => file.path === producerParent).sha256 &&
        git("diff", "--name-only") === "";
      save("receipt.json", receipt);
      assert.equal(row.beforeEdgeOverlay.restoredOriginalSource, true);
    }
  }
  if (label === "after-fix") {
    row.unitChecks = [];
    for (const [name, filter] of [
      ["spelling", "batch::import_rewriter_alias::module_spelling_tests::"],
      [
        "admission",
        "batch::import_rewriter::virtual_alias::tests::typescript_alias_admission_requires_explicit_bundler_and_original_alias_identity",
      ],
      ["producer", "corsa_bridge::vue_document_package_config_tests::extensionless_ts_alias::"],
    ]) {
      const args = [filter, "--nocapture"];
      const checked = spawnSync(binary, args, {
        cwd: root,
        env: { ...env, VIZE_EDITOR_ALIAS_CAPTURE: path.join(dir, "producer-capture") },
        encoding: "utf8",
        timeout: 180_000,
        maxBuffer: 32_000_000,
      });
      fs.writeFileSync(path.join(dir, name + "-unit.stdout"), checked.stdout ?? "");
      fs.writeFileSync(path.join(dir, name + "-unit.stderr"), checked.stderr ?? "");
      row.unitChecks.push({ name, command: binary, args, ...state(checked) });
      save("receipt.json", receipt);
    }
    row.allUnitChecksPass = row.unitChecks.every(
      (test) => test.status === 0 && test.error === null && test.signal === null,
    );
  }
  row.observationComplete = true;
  save("receipt.json", receipt);
  process.stdout.write(
    JSON.stringify({
      label,
      source,
      originalAssertionPassed: row.execution.originalAssertionPassed,
      diagnosticCount: row.completeDiagnostics.length,
      observationComplete: true,
      sourceQualification: false,
    }) + "\n",
  );
}
receipt.complete =
  receipt.results.length === 2 && receipt.results.every((row) => row.observationComplete);
receipt.rawDiagnosticVectorsEqual =
  JSON.stringify(receipt.results[0].completeDiagnostics) ===
  JSON.stringify(receipt.results[1].completeDiagnostics);
assert.equal(receipt.complete, true);
save("receipt.json", receipt);
