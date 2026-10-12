import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const source = "4cc1ef9cbeb3ee33df3ba6086b174b611f8d5681";
export const originalTest =
  "ide::signature_help::tests::signature_help_tracks_imported_callable_edits";
export const providerRevision = "2bd066d87f5bafd315be9f40889d0a60b9e58e0b";
export const artifact = {
  id: 11720466915,
  run: 38187359884,
  name: `rust-test-archive-38187359884-${source}`,
  size: 4905262425,
  digest: "sha256:3a345f7b956970ebc0474f60d5ef9256463eedcaa46dd90cdd052b22c65c380f",
};
const custody = [
  ".config/nextest.toml",
  "Cargo.lock",
  "package.json",
  "pnpm-lock.yaml",
  "crates/vize_maestro/src/ide/signature_help.rs",
  "crates/vize_maestro/src/ide/signature_help/tests.rs",
  "crates/vize_maestro/src/ide/signature_help/tests/fixture.rs",
  "crates/vize_canon/src/lsp_client/workspace_project.rs",
  "crates/vize_canon/src/lsp_client/lifecycle.rs",
  "crates/vize_canon/src/lsp_client/session/replay.rs",
  "target/nextest/cargo-metadata.json",
  "target/nextest/binaries-metadata.json",
];
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const json = (path) => JSON.parse(readFileSync(path, "utf8"));
const save = (path, value) => writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
const git = (...args) =>
  execFileSync("git", args, {
    encoding: "utf8",
    env: { ...process.env, GIT_NO_REPLACE_OBJECTS: "1" },
  }).trim();

export function requireArtifact(value) {
  assert.equal(value.id, artifact.id);
  assert.equal(value.name, artifact.name);
  assert.equal(value.size_in_bytes, artifact.size);
  assert.equal(value.digest, artifact.digest);
  assert.equal(value.expired, false);
  assert.equal(value.workflow_run.id, artifact.run);
  assert.equal(value.workflow_run.head_sha, source);
}

export function selectedBinary(list) {
  const suites = Object.values(list["rust-suites"]);
  const selected = suites.filter(
    (suite) =>
      suite["package-name"] === "vize_maestro" &&
      suite["binary-name"] === "vize_maestro" &&
      Object.hasOwn(suite.testcases, originalTest) &&
      suite.testcases[originalTest]["filter-match"].status === "matches",
  );
  assert.equal(selected.length, 1, "exact original test must select one archive binary");
  const matches = suites.flatMap((suite) =>
    Object.entries(suite.testcases)
      .filter(([, value]) => value["filter-match"].status === "matches")
      .map(([name]) => name),
  );
  assert.deepEqual(matches, [originalTest], "no additional test may run under capture");
  return selected[0]["binary-path"];
}

function prepare(directory) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(resolve(process.cwd()), "/home/runner/_work/vize/vize");
  assert.equal(git("rev-parse", "HEAD"), source);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  assert.equal(process.env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(process.env.VIZE_TEST_DISABLE_TSGO, undefined);
  assert.equal(process.env.VIZE_NUXT_CONFIG_ITERATIONS, "100");
  assert.equal(process.env.RUST_TEST_THREADS, "4");
  requireArtifact(json(join(directory, "artifact.json")));
  const receipt = json(".artifacts/rust-test-archive/receipt.json");
  assert.equal(receipt.sha, source);
  assert.equal(receipt.tree, git("rev-parse", "HEAD^{tree}"));
  assert.equal(receipt.workspaceRoot, process.cwd());
  const binary = realpathSync(selectedBinary(json(join(directory, "selected-tests.json"))));
  assert.ok(binary.startsWith(`${process.cwd()}/target/ci/deps/`));
  const nativePackage = realpathSync("node_modules/@typescript/typescript-linux-x64");
  const provider = realpathSync(join(nativePackage, "lib/tsc"));
  const nativeManifest = json(join(nativePackage, "package.json"));
  assert.equal(nativeManifest.version, "7.0.2");
  assert.equal(nativeManifest.gitHead, providerRevision);
  // These earlier candidates would change the original fixture's resolver.
  for (const candidate of [
    "../corsa-bind/.cache/tsgo",
    "../corsa-bind/ref/corsa-upstream/.cache/tsgo",
  ]) {
    assert.equal(execFileSync("test", ["!", "-e", candidate], { encoding: "utf8" }), "");
  }
  mkdirSync(join(directory, "bodies"), { recursive: true });
  const files = [
    ...custody,
    binary,
    provider,
    join(nativePackage, "package.json"),
    ...[
      "derived-binaries-metadata.json",
      "selected-tests.json",
      "derived-selected-tests.json",
      "library-mapping.json",
      "library-runtime.json",
      "readelf.stdout",
      "elf-loader.stdout",
    ].map((name) => join(directory, name)),
    ...["nextest", "cargo", "rustup", "toolchainCargo"].map(
      (key) => json(join(directory, "library-runtime.json"))[key],
    ),
    ...json(join(directory, "library-mapping.json")).dependencies.map((entry) => entry.path),
  ];
  const bodies = files.map((path, index) => {
    const bytes = readFileSync(path);
    const retained = `bodies/${index}.bin`;
    copyFileSync(path, join(directory, retained));
    return { path: resolve(path), retained, bytes: bytes.length, sha256: sha256(bytes) };
  });
  copyFileSync(
    ".artifacts/rust-test-archive/receipt.json",
    join(directory, "archive-receipt.json"),
  );
  save(join(directory, "source.json"), {
    source,
    tree: receipt.tree,
    artifact,
    binary,
    provider,
    providerPackage: join(nativePackage, "package.json"),
    providerRevision,
    originalTest,
    bodies,
    timing: "instrumented; no performance or acceptance credit",
    environment: {
      requireTsgo: "1",
      disableTsgo: null,
      nuxtIterations: "100",
      rustTestThreads: "4",
    },
  });
}

function finish(directory) {
  const manifest = json(join(directory, "source.json"));
  assert.equal(git("rev-parse", "HEAD"), source);
  assert.equal(git("status", "--porcelain", "--untracked-files=no"), "");
  for (const body of manifest.bodies) {
    assert.equal(
      sha256(readFileSync(body.path)),
      body.sha256,
      `source/binary changed: ${body.path}`,
    );
    assert.equal(sha256(readFileSync(join(directory, body.retained))), body.sha256);
  }
  save(join(directory, "source-unchanged.json"), { source, tree: manifest.tree, unchanged: true });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [operation, directory] = process.argv.slice(2);
  assert.ok(directory && ["prepare", "finish"].includes(operation));
  if (operation === "prepare") prepare(directory);
  else finish(directory);
}
