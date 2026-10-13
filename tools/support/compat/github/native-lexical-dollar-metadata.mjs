import assert from "node:assert/strict";
import { spawnSync, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  nativeLexicalSourcePaths,
  validateNativeLexicalSources,
} from "./native-lexical-javascript-source.mjs";

export const helperPaths = [
  "native-lexical-dollar.sh",
  "native-lexical-dollar-metadata.mjs",
  "native-lexical-dollar-receipt.mjs",
].map((name) => `tools/support/compat/github/${name}`);
export const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const cargoArgs = [
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
  "--config",
  'profile.ci-opt.inherits="release"',
  "--config",
  'profile.ci-opt.lto="thin"',
  "--config",
  "profile.ci-opt.codegen-units=16",
];
export const originalReceiptNames = [
  "receipts.json",
  "imported-runtime-props.runtime.json",
  "editor-runtime.json",
];
export const unobserved = [
  "diagnosing native PID",
  "native normalized options",
  "engine loaded source-file vector",
];
export const sourcePaths = (manifest) => nativeLexicalSourcePaths(manifest, helperPaths);
const git = (args) => execFileSync("git", ["--no-replace-objects", ...args]);
const identity = (file) => {
  const actual = fs.realpathSync(file);
  assert.ok(fs.lstatSync(actual).isFile());
  const bytes = fs.readFileSync(actual);
  return { path: actual, byteLength: bytes.length, sha256: digest(bytes) };
};

export function captureMetadata(output) {
  assert.equal(process.env.GIT_NO_REPLACE_OBJECTS, "1");
  assert.equal(git(["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  assert.equal(git(["status", "--porcelain", "--untracked-files=no"]).length, 0);
  const root = fs.realpathSync(process.cwd());
  const head = git(["rev-parse", "HEAD"]).toString().trim();
  const tree = git(["rev-parse", "HEAD^{tree}"]).toString().trim();
  assert.equal(fs.readFileSync(path.join(output, "source.sha"), "utf8").trim(), head);
  const compilerLines = fs
    .readFileSync(path.join(output, "cargo.stdout"), "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const executables = compilerLines.filter(
    (record) =>
      record.reason === "compiler-artifact" &&
      record.target.name === "vize_canon" &&
      record.target.kind.join() === "lib" &&
      record.profile.test &&
      record.executable,
  );
  assert.equal(executables.length, 1);
  const artifact = executables[0];
  assert.equal(
    fs.realpathSync(artifact.target.src_path),
    path.join(root, "crates/vize_canon/src/lib.rs"),
  );
  assert.equal(fs.readFileSync(path.join(output, "cargo.status"), "utf8").trim(), "0");
  assert.deepEqual(fs.readFileSync(path.join(output, "cargo.argv.nul"), "utf8").split("\0"), [
    ...cargoArgs,
    "",
  ]);
  const testBinary = identity(artifact.executable);
  const native = identity(process.env.CORSA_PATH);
  const packagePath = path.resolve(path.dirname(native.path), "../package.json");
  const packageBytes = fs.readFileSync(packagePath);
  const nativePackage = JSON.parse(packageBytes);
  assert.equal(nativePackage.name, `@typescript/typescript-${process.platform}-${process.arch}`);
  assert.equal(nativePackage.version, "7.0.2");
  assert.equal(native.path, path.join(path.dirname(packagePath), "lib/tsc"));
  const probe = spawnSync(native.path, ["--version"]);
  assert.equal(probe.status, 0);
  assert.equal(probe.signal, null);
  assert.deepEqual(probe.stderr, Buffer.alloc(0));
  assert.equal(probe.stdout.toString().trim(), "Version 7.0.2");
  const fixture = "tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879";
  const manifest = JSON.parse(
    fs.readFileSync(
      "tests/performance/support/warm-type-backed-lexical-dollar-manifest.json",
      "utf8",
    ),
  );
  const sources = sourcePaths(manifest);
  const copied = sources.map((file) => {
    const entry = git(["ls-tree", "-z", "HEAD", "--", file]);
    const match = /^100644 blob ([a-f0-9]{40})\t(.+)\0$/u.exec(entry.toString());
    assert.ok(match, file);
    assert.equal(match[2], file);
    const bytes = fs.readFileSync(file);
    assert.deepEqual(bytes, git(["cat-file", "blob", match[1]]));
    const target = path.join(output, "source", file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, bytes);
    if (file.startsWith(`${fixture}/`))
      assert.deepEqual(
        fs.readFileSync(path.join(output, "corpus", file.slice(fixture.length + 1))),
        bytes,
      );
    return {
      file,
      gitEntryHex: entry.toString("hex"),
      byteLength: bytes.length,
      sha256: digest(bytes),
    };
  });
  const originalReceipts = originalReceiptNames.map((name) => {
    const original = path.join(process.env.VIZE_DEFAULT_PROP_CAPTURE, name);
    const body = fs.readFileSync(original);
    fs.mkdirSync(path.join(output, "original-runtime"), { recursive: true });
    fs.writeFileSync(path.join(output, "original-runtime", name), body);
    return { name, ...identity(original) };
  });
  const buildTools = ["cargo", "rustc"].map((tool) => {
    const result = spawnSync(tool, ["--version"]);
    assert.equal(result.status, 0);
    assert.deepEqual(result.stderr, Buffer.alloc(0));
    return { tool, stdout: result.stdout.toString(), status: result.status, signal: result.signal };
  });
  const metadata = {
    version: 1,
    source: { root, head, tree, noReplaceObjects: true },
    copied,
    testBinary,
    compilerArtifact: artifact,
    originalReceipts,
    buildTools,
    vueManifest: identity("tests/node_modules/vue/package.json"),
    runtimeEnvironment: {
      requiredNative: process.env.VIZE_TEST_REQUIRE_TSGO,
      disableNative: process.env.VIZE_TEST_DISABLE_TSGO ?? null,
      tsgoPath: fs.realpathSync(process.env.VIZE_TEST_TSGO_PATH),
      corsaPath: native.path,
      cargoTarget: fs.realpathSync(process.env.CARGO_TARGET_DIR),
    },
    native: {
      ...native,
      packagePath,
      packageBytes: [...packageBytes],
      versionProbe: {
        stdoutBytes: [...probe.stdout],
        stderrBytes: [...probe.stderr],
        status: probe.status,
        signal: probe.signal,
      },
    },
    unobserved,
  };
  fs.writeFileSync(path.join(output, "metadata.json"), `${JSON.stringify(metadata, null, 2)}\n`);
  return metadata;
}

export function validateMetadata(output) {
  const read = (name) => JSON.parse(fs.readFileSync(path.join(output, name), "utf8"));
  const text = (name) => fs.readFileSync(path.join(output, name), "utf8");
  const metadata = read("metadata.json");
  const sourceGit = (args) =>
    execFileSync("git", ["--no-replace-objects", ...args], { cwd: metadata.source.root });
  assert.equal(metadata.version, 1);
  assert.equal(metadata.source.noReplaceObjects, true);
  assert.equal(sourceGit(["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  assert.equal(metadata.source.head, sourceGit(["rev-parse", "HEAD"]).toString().trim());
  assert.equal(metadata.source.tree, sourceGit(["rev-parse", "HEAD^{tree}"]).toString().trim());
  assert.equal(text("source.sha").trim(), metadata.source.head);
  assert.equal(new Set(metadata.copied.map((entry) => entry.file)).size, metadata.copied.length);
  for (const entry of metadata.copied) {
    assert.ok(fs.lstatSync(path.join(output, "source", entry.file)).isFile());
    const body = fs.readFileSync(path.join(output, "source", entry.file));
    assert.equal(body.length, entry.byteLength);
    assert.equal(digest(body), entry.sha256);
    const blob = createHash("sha1")
      .update(Buffer.from(`blob ${body.length}\0`))
      .update(body)
      .digest("hex");
    const gitEntry = Buffer.from(`100644 blob ${blob}\t${entry.file}\0`);
    assert.deepEqual(Buffer.from(entry.gitEntryHex, "hex"), gitEntry);
    assert.deepEqual(sourceGit(["ls-tree", "-z", "HEAD", "--", entry.file]), gitEntry);
  }
  const manifest = read(
    "source/tests/performance/support/warm-type-backed-lexical-dollar-manifest.json",
  );
  validateNativeLexicalSources(output, metadata, manifest, helperPaths);
  assert.equal(manifest.files.length, 9);
  assert.equal(manifest.corpusBodies.length, 21);
  for (const executable of [metadata.testBinary, metadata.native])
    assert.deepEqual(identity(executable.path), {
      path: executable.path,
      byteLength: executable.byteLength,
      sha256: executable.sha256,
    });
  assert.equal(text("cargo.status"), "0\n");
  assert.deepEqual(text("cargo.argv.nul").split("\0"), [...cargoArgs, ""]);
  const records = text("cargo.stdout")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const artifacts = records.filter(
    (record) =>
      record.reason === "compiler-artifact" &&
      record.target.name === "vize_canon" &&
      record.target.kind.join() === "lib" &&
      record.profile.test &&
      record.executable,
  );
  assert.deepEqual(artifacts, [metadata.compilerArtifact]);
  assert.deepEqual(
    records.filter((record) => record.reason === "build-finished"),
    [{ reason: "build-finished", success: true }],
  );
  const artifact = metadata.compilerArtifact;
  assert.equal(artifact.executable, metadata.testBinary.path);
  assert.equal(
    artifact.target.src_path,
    path.join(metadata.source.root, "crates/vize_canon/src/lib.rs"),
  );
  assert.equal(artifact.profile.opt_level, "3");
  assert.equal(artifact.profile.debug_assertions, false);
  assert.deepEqual(
    [...artifact.features].sort((a, b) => a.localeCompare(b)),
    ["default", "native"],
  );
  assert.deepEqual(
    metadata.buildTools.map((item) => item.tool),
    ["cargo", "rustc"],
  );
  for (const item of metadata.buildTools) {
    assert.equal(item.status, 0);
    assert.equal(item.signal, null);
    assert.match(item.stdout, new RegExp(`^${item.tool} 1\\.99\\.0(?: |\\n)`, "u"));
  }
  const environment = metadata.runtimeEnvironment;
  assert.equal(environment.requiredNative, "1");
  assert.equal(environment.disableNative, null);
  assert.equal(environment.tsgoPath, metadata.native.path);
  assert.equal(environment.corsaPath, metadata.native.path);
  assert.equal(fs.realpathSync(environment.cargoTarget), environment.cargoTarget);
  assert.equal(
    path.relative(environment.cargoTarget, metadata.testBinary.path).startsWith(".."),
    false,
  );
  const packageBytes = Buffer.from(metadata.native.packageBytes);
  assert.deepEqual([...packageBytes], metadata.native.packageBytes);
  const pkg = JSON.parse(packageBytes.toString());
  assert.equal(pkg.name, `@typescript/typescript-${process.platform}-${process.arch}`);
  assert.equal(pkg.version, "7.0.2");
  assert.deepEqual(fs.readFileSync(metadata.native.packagePath), packageBytes);
  assert.equal(
    metadata.native.path,
    path.join(path.dirname(metadata.native.packagePath), "lib/tsc"),
  );
  assert.equal(metadata.native.versionProbe.status, 0);
  assert.equal(metadata.native.versionProbe.signal, null);
  assert.deepEqual(metadata.native.versionProbe.stderrBytes, []);
  assert.deepEqual(
    [...Buffer.from(metadata.native.versionProbe.stdoutBytes)],
    metadata.native.versionProbe.stdoutBytes,
  );
  assert.equal(
    Buffer.from(metadata.native.versionProbe.stdoutBytes).toString().trim(),
    "Version 7.0.2",
  );
  assert.deepEqual(
    metadata.originalReceipts.map((receipt) => receipt.name),
    originalReceiptNames,
  );
  for (const receipt of metadata.originalReceipts) {
    const body = fs.readFileSync(path.join(output, "original-runtime", receipt.name));
    assert.equal(body.length, receipt.byteLength);
    assert.equal(digest(body), receipt.sha256);
    assert.deepEqual(body, fs.readFileSync(receipt.path));
  }
  const original = read("original-runtime/receipts.json");
  assert.equal(original.sourceSha, metadata.source.head);
  assert.equal(original.matchedCompleteExpectations, true);
  assert.deepEqual(
    original.cases.map((item) => item.name),
    [
      "imported_callable_default_remains_defined_in_all_template_prop_views",
      "undefined_callable_default_keeps_original_cli_conditions_optional",
      "undefined_callable_default_still_rejects_unguarded_template_calls",
    ],
  );
  for (const item of original.cases) {
    assert.equal(item.sourceSha, metadata.source.head);
    assert.equal(item.nativeBinary, metadata.native.path);
    assert.equal(item.nativeSha256, metadata.native.sha256);
    assert.equal(item.nativePackageSha256, digest(packageBytes));
    assert.equal(item.nativeVersion, "Version 7.0.2");
  }
  for (const name of originalReceiptNames.slice(1)) {
    const runtime = read(`original-runtime/${name}`);
    if (name === "imported-runtime-props.runtime.json")
      assert.equal(runtime.sourceSha, metadata.source.head);
    assert.equal(fs.realpathSync(runtime.nativeBinary), metadata.native.path);
  }
  assert.deepEqual(metadata.unobserved, unobserved);
  assert.deepEqual(identity(metadata.vueManifest.path), metadata.vueManifest);
  assert.equal(JSON.parse(fs.readFileSync(metadata.vueManifest.path, "utf8")).name, "vue");
  return { metadata, manifest };
}

if (
  process.argv[1] &&
  fs.existsSync(process.argv[1]) &&
  fs.realpathSync(process.argv[1]) === fs.realpathSync(new URL(import.meta.url))
)
  captureMetadata(process.argv[2]);
