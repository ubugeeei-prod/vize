import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const { ROLE, BASE_SHA, HEAD_SHA, SDK_VERSION, RUNNER_TEMP, GITHUB_WORKSPACE, GITHUB_ENV } =
  process.env as NodeJS.ProcessEnv &
    Record<
      | "ROLE"
      | "BASE_SHA"
      | "HEAD_SHA"
      | "SDK_VERSION"
      | "RUNNER_TEMP"
      | "GITHUB_WORKSPACE"
      | "GITHUB_ENV",
      string
    >;
assert(["before", "after"].includes(ROLE));
assert(["7.0.2", "7.1.0-beta"].includes(SDK_VERSION));
for (const sha of [BASE_SHA, HEAD_SHA]) assert.match(sha, /^[0-9a-f]{40}$/);
const git = (args: string[], cwd = GITHUB_WORKSPACE) =>
  execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
assert.equal(git(["rev-parse", "HEAD"]), HEAD_SHA);
execFileSync("git", ["diff", "--exit-code"], { cwd: GITHUB_WORKSPACE });
const source = ROLE === "before" ? BASE_SHA : HEAD_SHA;
const root =
  ROLE === "before" ? path.join(RUNNER_TEMP, `native-dto-before-${source}`) : GITHUB_WORKSPACE;
const output = path.join(
  GITHUB_WORKSPACE,
  "target/vize-tests/native-beta-dto",
  `${ROLE}-${SDK_VERSION}`,
);
fs.mkdirSync(output, { recursive: true });
const observer = "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta.rs";
const observerDependencies = [
  {
    path: "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/storage_authority.rs",
    sha256: "e760076f8111d8d20c6e35574b79c4da01e2c6d96c7b068b35de82d9bb0c2804",
  },
];
const registration = "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests.rs";
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const harness = fs.readFileSync(path.join(GITHUB_WORKSPACE, observer));
for (const dependency of observerDependencies)
  assert.equal(
    hash(fs.readFileSync(path.join(GITHUB_WORKSPACE, dependency.path))),
    dependency.sha256,
  );
if (ROLE === "before") {
  assert(!fs.existsSync(root));
  execFileSync("git", ["worktree", "add", "--detach", root, source], { cwd: GITHUB_WORKSPACE });
  const original = fs.readFileSync(path.join(root, registration), "utf8");
  assert(!original.includes("mod native_beta;"));
  fs.writeFileSync(path.join(root, registration), `${original}\n#[cfg(unix)]\nmod native_beta;\n`);
  fs.copyFileSync(path.join(GITHUB_WORKSPACE, observer), path.join(root, observer));
  for (const dependency of observerDependencies) {
    const target = path.join(root, dependency.path);
    assert(
      !fs.existsSync(target),
      "before source must not already contain the current observer helper",
    );
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(GITHUB_WORKSPACE, dependency.path), target);
  }
  // The sole tracked delta only registers the separately hashed public-API test.
  assert.equal(git(["diff", "--name-only"], root), registration);
  assert.deepEqual(
    git(["ls-files", "--others", "--exclude-standard"], root).split("\n").sort(),
    [observer, ...observerDependencies.map((dependency) => dependency.path)].sort(),
  );
  execFileSync("git", ["diff", "--exit-code", "--", ".", `:(exclude)${registration}`], {
    cwd: root,
  });
  fs.writeFileSync(
    path.join(output, "test-only-registration.patch"),
    execFileSync("git", ["diff"], { cwd: root }),
  );
}
assert.equal(hash(fs.readFileSync(path.join(root, observer))), hash(harness));
for (const dependency of observerDependencies)
  assert.equal(hash(fs.readFileSync(path.join(root, dependency.path))), dependency.sha256);
const production = [
  "crates/vize_canon/src/lsp_client/initial_diagnostics/diagnostic_response.rs",
  "crates/vize_canon/src/lsp_client/initial_diagnostics/native_diagnostics.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/protocol.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp.rs",
  "crates/vize_canon/src/lsp_client/session/capabilities.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/component_types.rs",
  "crates/vize_canon/src/lsp_client/diagnostics/batch.rs",
];
for (const file of production) {
  assert.deepEqual(
    fs.readFileSync(path.join(root, file)),
    execFileSync("git", ["show", `${source}:${file}`], { cwd: root }),
  );
}
const manifest = {
  source,
  role: ROLE,
  sdkVersion: SDK_VERSION,
  head: HEAD_SHA,
  harness: { path: observer, sha256: hash(harness) },
  harnessDependencies: observerDependencies,
  production: Object.fromEntries(
    production.map((file) => [file, hash(fs.readFileSync(path.join(root, file)))]),
  ),
  scope:
    "Before production remains pinned; only the identical separately hashed observer, its one exact source-storage observation helper, and the test-only module registration are copied. Candidate decoders are never copied into the before role.",
};
fs.writeFileSync(path.join(output, "source-role.json"), JSON.stringify(manifest, null, 2) + "\n");
const target = path.join(RUNNER_TEMP, `native-dto-target-${ROLE}-${SDK_VERSION}-${source}`);
assert(!fs.existsSync(target));
for (const [name, value] of Object.entries({
  VIZE_NATIVE_DTO_SOURCE_SHA: source,
  VIZE_NATIVE_DTO_SOURCE_ROOT: root,
  VIZE_NATIVE_DTO_ROLE: ROLE,
  VIZE_NATIVE_DTO_CAPTURE: output,
  CARGO_TARGET_DIR: target,
}))
  fs.appendFileSync(GITHUB_ENV, `${name}=${value}\n`);
