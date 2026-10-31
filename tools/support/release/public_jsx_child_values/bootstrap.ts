/** Hosted-only, independent public installation and generic release/image custody. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { boundedBytes, githubSnapshot } from "../release/quota/http.ts";
import { readProducedCandidate } from "../release/quota/producer.ts";
import { verifyPublicProvider } from "../public-provider.ts";
import {
  cachedArchive,
  checkSources,
  completedProducer,
  image,
  lockEntry,
  read,
  request,
  sha256,
  validateArchive,
  within,
  write,
  type Authority,
} from "./contract.ts";
import { qualificationTarget, targetCustody } from "./target.ts";

const [version, candidate, producerRun, output] = process.argv.slice(2);
assert(version && candidate && producerRun && output);
request(version, candidate, producerRun, process.env);
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
const report = process.report.getReport() as { header?: { glibcVersionRuntime?: string } };
assert(report.header?.glibcVersionRuntime, "actual GNU native target required");
const checkout = fs.realpathSync(process.cwd());
const root = path.resolve(output);
assert(root.startsWith(fs.realpathSync(process.env.RUNNER_TEMP!) + path.sep));
assert(!fs.existsSync(root), "a completely fresh public installation is required");
fs.mkdirSync(root);
const proof = path.join(root, "proof");
fs.mkdirSync(proof);
const target = qualificationTarget(process.env.PUBLIC_JSX_TARGET);
const custody = targetCustody(target);
checkSources(checkout, custody);
const git = (...args: string[]): string => {
  const result = spawnSync("git", ["--no-replace-objects", ...args], {
    cwd: checkout,
    encoding: "utf8",
    timeout: 60000,
    maxBuffer: 2 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
};
assert.equal(git("rev-parse", "HEAD").trim(), process.env.GITHUB_SHA);
const remote = git(
  "ls-remote",
  "https://github.com/ubugeeei-prod/vize.git",
  "refs/tags/v" + version,
);
assert.deepEqual(remote.trim().split(/\s+/), [candidate, "refs/tags/v" + version]);
git("fetch", "--no-tags", "origin", "refs/tags/v" + version);
assert.equal(git("rev-parse", "FETCH_HEAD^{commit}").trim(), candidate);
git("merge-base", "--is-ancestor", custody.merge, candidate);
const commit = git("cat-file", "commit", candidate);
const parents = commit
  .split("\n\n", 1)[0]!
  .split("\n")
  .filter((line) => line.startsWith("parent "))
  .map((line) => line.slice(7));
assert.equal(parents.length, 1);
const producerSource = parents[0]!;
git("merge-base", "--is-ancestor", custody.merge, producerSource);
write(path.join(proof, "tag-source.json"), {
  version,
  candidate,
  producerRun,
  requiredMerge: custody.merge,
  containsRequiredMerge: true,
  remote,
  commit,
});
const run = await githubSnapshot("actions/runs/" + producerRun);
write(path.join(proof, "producer-run.json"), run);
const facts = completedProducer(run.value, producerRun);
const produced = await readProducedCandidate({
  version,
  runId: producerRun,
  workflowHead: facts.head_sha,
  runAttempt: facts.run_attempt,
  candidateSha: candidate,
  producerSource,
  nativeManifestGitBlob: git("rev-parse", candidate + ":npm/native/package.json").trim(),
  inventoryGitBlob: git(
    "rev-parse",
    candidate + ":tools/support/release/publish_crates/main.ts",
  ).trim(),
});
write(path.join(proof, "producer-candidate.json"), produced);
// Authentication of an actual producer is separate from unrelated quota/product admissions.
const nativeManifest = JSON.parse(git("cat-file", "blob", candidate + ":npm/native/package.json"));
assert.equal(nativeManifest.name, "@vizejs/native");
assert.equal(nativeManifest.version, version);

async function install(name: string, dependencies: Record<string, string>): Promise<string> {
  const workspace = path.join(root, name);
  fs.mkdirSync(workspace);
  write(path.join(workspace, "package.json"), {
    name: "vize-public-jsx-" + name,
    private: true,
    type: "module",
    dependencies,
  });
  fs.writeFileSync(path.join(workspace, ".npmrc"), "registry=https://registry.npmjs.org/\n");
  const cache = path.join(workspace, ".npm-cache");
  const env = { ...process.env };
  for (const key of ["GH_TOKEN", "GITHUB_TOKEN", "NPM_TOKEN", "NODE_AUTH_TOKEN"]) delete env[key];
  const args = [
    "install",
    "--ignore-scripts",
    "--legacy-peer-deps",
    "--no-audit",
    "--no-fund",
    "--prefer-online",
    "--registry=https://registry.npmjs.org",
    "--userconfig",
    path.join(workspace, ".npmrc"),
    "--cache",
    cache,
  ];
  const result = spawnSync("npm", args, {
    cwd: workspace,
    env,
    encoding: "utf8",
    timeout: 480000,
    maxBuffer: 8 * 1024 * 1024,
  });
  write(path.join(proof, name + "-install.json"), {
    command: ["npm", ...args],
    dependencies,
    status: result.status,
    signal: result.signal,
    error: result.error?.message,
    stdout: result.stdout,
    stderr: result.stderr,
  });
  assert.equal(result.status, 0, "genuine independent public npm installation failed");
  return workspace;
}
const consumer = await install("consumer", { "@vizejs/native": version });
const oracle = await install("oracle", custody.referenceDependencies);
for (const name of Object.keys(custody.pins)) {
  const destination = path.join(oracle, name);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.copyFileSync(path.join(checkout, name), destination, fs.constants.COPYFILE_EXCL);
}
checkSources(oracle, custody);
const require = createRequire(path.join(consumer, "package.json"));
const entrypoint = within(path.join(consumer, "node_modules"), require.resolve("@vizejs/native"));
const packageJson = path.join(path.dirname(entrypoint), "package.json");
const native = require("@vizejs/native") as { getNativeBuildToken(): string };
const identity = (
  require("@vizejs/native/native-binding-identity.js") as {
    getBindingIdentity(binding: unknown): { kind: "native"; token: string };
  }
).getBindingIdentity(native);
assert.equal(identity.kind, "native");
assert.match(identity.token, /^[0-9a-f]{32}$/);
assert.equal(native.getNativeBuildToken(), identity.token);
const binaryPackage = "@vizejs/native-linux-x64-gnu";
const binary = within(path.join(consumer, "node_modules"), require.resolve(binaryPackage));
assert(binary.endsWith(".node"));
const loaded = Object.keys(require.cache).filter((file) => file.endsWith(".node"));
assert.equal(loaded.length, 1);
assert.equal(fs.realpathSync(loaded[0]!), binary);
const provider = await verifyPublicProvider(version, candidate, image(binary));
write(path.join(proof, "platform-provider.json"), provider);

async function verifyPackage(workspace: string, name: string, expected: string): Promise<void> {
  const lock = read<{ packages: Record<string, unknown> }>(
    path.join(workspace, "package-lock.json"),
  );
  const locked = lockEntry(lock.packages["node_modules/" + name], name, expected);
  const response = await fetch(
    "https://registry.npmjs.org/" + encodeURIComponent(name) + "/" + expected,
    { redirect: "error", signal: AbortSignal.timeout(30000) },
  );
  assert.equal(response.status, 200);
  const body = await boundedBytes(response, 2 * 1024 * 1024);
  const metadata = JSON.parse(body.toString()) as {
    name: string;
    version: string;
    dist: { integrity: string; tarball: string };
  };
  assert.equal(metadata.name, name);
  assert.equal(metadata.version, expected);
  assert.equal(metadata.dist.integrity, locked.integrity);
  assert.equal(metadata.dist.tarball, locked.resolved);
  const archive = cachedArchive(path.join(workspace, ".npm-cache"), locked.integrity);
  const installed = within(
    path.join(workspace, "node_modules"),
    path.join(workspace, "node_modules", name),
  );
  const manifest = read<{ name: string; version: string }>(path.join(installed, "package.json"));
  assert.equal(manifest.name, name);
  assert.equal(manifest.version, expected);
  const files = validateArchive(archive, installed);
  write(
    path.join(
      proof,
      (workspace === consumer ? "consumer-" : "oracle-") + name.replaceAll("/", "_") + ".json",
    ),
    {
      name,
      version: expected,
      locked,
      registry: {
        status: response.status,
        bytes: body.length,
        sha256: sha256(body),
        bodyBase64: body.toString("base64"),
      },
      archive: imageFromBytes(archive),
      files,
    },
  );
}
function imageFromBytes(bytes: Buffer) {
  return { bytes: bytes.length, sha256: sha256(bytes) };
}
for (const [name, expected] of [
  ["@vizejs/native", version],
  [binaryPackage, version],
])
  await verifyPackage(consumer, name!, expected!);
for (const [name, expected] of Object.entries(custody.referenceDependencies))
  await verifyPackage(oracle, name, expected);
const authority: Authority = {
  version,
  candidate,
  producerRun,
  nativeIdentity: identity,
  entrypoint: image(entrypoint),
  packageJson: image(packageJson),
  binary: { package: binaryPackage, ...image(binary) },
};
write(path.join(proof, "authority.json"), authority);
write(path.join(proof, "bootstrap-complete.json"), {
  version,
  candidate,
  producerRun,
  authority: image(path.join(proof, "authority.json")),
  custody,
  qualificationTarget: target,
  checkout,
  source: image(fileURLToPath(import.meta.url)),
  scope:
    "installed public native compiler; separate unchanged original prerelease Vapor reference/runtime environment",
});
