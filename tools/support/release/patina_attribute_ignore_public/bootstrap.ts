/** Hosted-only generic producer, registry installation and native-image custody. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { asArray, asRecord, asString, parseJson, type JsonRecord } from "../../json-value.ts";
import { npmArgv, Recorder, success } from "../../config/windows_consumer/common.ts";
import { boundedBytes, githubSnapshot } from "../release/quota/http.ts";
import { readProducedCandidate } from "../release/quota/producer.ts";
import { verifyPublicProvider } from "../public-provider.ts";
import { archiveComparison, recheckFiles, type ArchiveReceipt } from "../public_install/archive.ts";
import { sriSha512 } from "../public_install/registry.ts";
import {
  ancestry,
  completedProducer,
  custody,
  digest,
  image,
  inside,
  requiredMerges,
  write,
  type Native,
} from "./contract.ts";

export type Context = {
  version: string;
  root: string;
  proof: string;
  record: Recorder;
  native: Native;
  binary: string;
  identity: JsonRecord;
  cli: string;
  plugin: string;
  eslint: string;
  parser: string;
  oxlint: string;
  receipts: ArchiveReceipt[];
};
const gitBlob = (bytes: Buffer) =>
  createHash("sha1")
    .update("blob " + bytes.length + "\0")
    .update(bytes)
    .digest("hex");
function cached(cache: string, integrity: string): Buffer {
  const bytes = sriSha512(integrity),
    key = bytes.toString("hex");
  const file = join(
    cache,
    "_cacache/content-v2/sha512",
    key.slice(0, 2),
    key.slice(2, 4),
    key.slice(4),
  );
  const result = readFileSync(file);
  assert(result.length > 0 && result.length <= 128 * 1024 * 1024);
  assert(createHash("sha512").update(result).digest().equals(bytes));
  return result;
}
export function preserve(context: Context): void {
  recheckFiles(context.receipts);
}

export async function bootstrap(
  version: string,
  candidate: string,
  producerRun: string,
  output: string,
): Promise<Context> {
  const source = custody();
  assert(!existsSync(output), "a fresh output and installation are required");
  mkdirSync(output);
  const proof = join(output, "proof");
  mkdirSync(proof);
  const record = new Recorder(proof);
  const observations: unknown[] = [];
  const read = async (endpoint: string) => {
    const snapshot = await githubSnapshot(endpoint);
    observations.push({ endpoint, ...snapshot });
    record.json("github-observations.json", observations);
    return asRecord(snapshot.value);
  };
  const tag = await read("git/ref/tags/v" + version);
  assert.equal(tag.ref, "refs/tags/v" + version);
  assert.equal(asRecord(tag.object).type, "commit");
  assert.equal(asRecord(tag.object).sha, candidate);
  const release = await read("releases/tags/v" + version);
  assert.equal(release.tag_name, "v" + version);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  assert(Number.isFinite(Date.parse(asString(release.published_at))));
  const commit = await read("git/commits/" + candidate);
  assert.equal(commit.sha, candidate);
  const parents = asArray(commit.parents);
  assert.equal(parents.length, 1);
  const producerSource = asString(asRecord(parents[0]).sha);
  assert.match(producerSource, /^[0-9a-f]{40}$/u);
  for (const merge of requiredMerges)
    ancestry(await read("compare/" + merge + "..." + producerSource), merge);
  ancestry(await read("compare/" + producerSource + "..." + candidate), producerSource);
  const producer = completedProducer(await read("actions/runs/" + producerRun), producerRun);
  const nativeManifest = await read("contents/npm/native/package.json?ref=" + candidate);
  assert.equal(nativeManifest.encoding, "base64");
  const nativeBytes = Buffer.from(asString(nativeManifest.content), "base64");
  assert.equal(gitBlob(nativeBytes), nativeManifest.sha);
  const packageJson = asRecord(parseJson(nativeBytes.toString()));
  assert.equal(packageJson.name, "@vizejs/native");
  assert.equal(packageJson.version, version);
  const inventory = await read(
    "contents/tools/support/release/publish_crates/main.ts?ref=" + candidate,
  );
  const produced = await readProducedCandidate({
    version,
    candidateSha: candidate,
    producerSource,
    runId: producerRun,
    workflowHead: asString(producer.head_sha),
    runAttempt: Number(producer.run_attempt),
    nativeManifestGitBlob: asString(nativeManifest.sha),
    inventoryGitBlob: asString(inventory.sha),
  });
  write(join(proof, "producer-candidate.json"), produced);
  for (const raw of asArray(source.files)) {
    const row = asRecord(raw),
      path = asString(row.path);
    const response = await read("contents/" + path + "?ref=" + asString(source.source));
    assert.equal(response.type, "file");
    assert.equal(response.path, path);
    assert.equal(response.encoding, "base64");
    const bytes = Buffer.from(asString(response.content), "base64");
    assert.equal(response.sha, row.gitBlob);
    assert.equal(gitBlob(bytes), row.gitBlob);
    assert.equal(bytes.length, row.bytes);
    assert.equal(digest(bytes), row.sha256);
  }
  write(join(proof, "source-custody.json"), source);
  write(join(proof, "release-authority.json"), {
    version,
    candidate,
    producerRun,
    producerSource,
    producer,
    requiredMerges,
    controllerSource: process.env.GITHUB_SHA,
    consumerRun: process.env.GITHUB_RUN_ID,
    consumerAttempt: process.env.GITHUB_RUN_ATTEMPT,
  });
  const root = join(output, "consumer");
  mkdirSync(root);
  const dependencies: Record<string, string> = {
    vize: version,
    "@vizejs/native": version,
    "@vizejs/native-linux-x64-gnu": version,
    "oxlint-plugin-vize": version,
    ...(asRecord(source.targetProviders) as Record<string, string>),
  };
  write(join(root, "package.json"), {
    name: "vize-public-attribute-ignore",
    private: true,
    type: "module",
    dependencies,
  });
  writeFileSync(join(root, ".npmrc"), "registry=https://registry.npmjs.org/\n", { flag: "wx" });
  const cache = join(root, ".npm-cache");
  const env = { ...process.env };
  for (const key of ["GH_TOKEN", "GITHUB_TOKEN", "NPM_TOKEN", "NODE_AUTH_TOKEN"]) delete env[key];
  const installed = record.run(
    "npm-install",
    npmArgv(
      "install",
      "--ignore-scripts",
      "--legacy-peer-deps",
      "--no-audit",
      "--no-fund",
      "--prefer-online",
      "--registry=https://registry.npmjs.org",
      "--userconfig",
      join(root, ".npmrc"),
      "--cache",
      cache,
    ),
    root,
    env,
    480000,
  );
  success(installed);
  const lock = asRecord(
    asRecord(parseJson(readFileSync(join(root, "package-lock.json"), "utf8"))).packages,
  );
  const receipts: ArchiveReceipt[] = [];
  for (const [name, expected] of Object.entries(dependencies)) {
    const locked = asRecord(lock["node_modules/" + name]);
    assert.equal(locked.version, expected, name);
    const url = new URL(asString(locked.resolved));
    assert.equal(url.origin, "https://registry.npmjs.org");
    assert(!url.username && !url.password && !url.search && !url.hash);
    assert.equal(
      url.pathname,
      `/${name}/-/${name.slice(name.lastIndexOf("/") + 1)}-${expected}.tgz`,
    );
    const response = await fetch(
      "https://registry.npmjs.org/" + encodeURIComponent(name) + "/" + expected,
      { redirect: "error", signal: AbortSignal.timeout(30000) },
    );
    const raw = await boundedBytes(response, 2 * 1024 * 1024);
    record.json("registry-" + name.replaceAll("/", "_") + ".json", {
      status: response.status,
      bytes: raw.length,
      sha256: digest(raw),
      rawBase64: raw.toString("base64"),
    });
    assert.equal(response.status, 200);
    const metadata = asRecord(parseJson(raw.toString()));
    assert.equal(metadata.name, name);
    assert.equal(metadata.version, expected);
    assert.equal(asRecord(metadata.dist).integrity, locked.integrity);
    assert.equal(asRecord(metadata.dist).tarball, locked.resolved);
    const directory = realpathSync(join(root, "node_modules", name));
    assert(directory.startsWith(realpathSync(root) + "/node_modules/"));
    const archive = archiveComparison(
      directory,
      cached(cache, asString(locked.integrity)),
      locked.integrity,
    );
    record.json("package-" + name.replaceAll("/", "_") + ".json", {
      name,
      version: expected,
      locked,
      archive,
    });
    receipts.push({ packageDirectory: directory, files: archive.files });
  }
  const require = createRequire(join(root, "package.json"));
  const nativeEntry = inside(root, require.resolve("@vizejs/native"));
  const binary = inside(root, require.resolve("@vizejs/native-linux-x64-gnu"));
  const binding: unknown = require("@vizejs/native");
  const native = asRecord(binding) as unknown as Native;
  assert.equal(typeof native.lintPatinaSfc, "function");
  assert.equal(typeof native.getNativeBuildToken, "function");
  const helper = asRecord(require("@vizejs/native/native-binding-identity.js"));
  assert.equal(typeof helper.getBindingIdentity, "function");
  const identity = asRecord(
    Reflect.apply(helper.getBindingIdentity as (...args: unknown[]) => unknown, helper, [native]),
  );
  assert.equal(identity.kind, "native");
  assert.match(asString(identity.token), /^[0-9a-f]{32}$/u);
  assert.equal(native.getNativeBuildToken(), identity.token);
  assert.equal(require.cache[binary]?.loaded, true);
  assert.equal(asRecord(require.cache[binary]?.exports).lintPatinaSfc, native.lintPatinaSfc);
  assert.deepEqual(
    Object.keys(require.cache).filter((file) => file.endsWith(".node")),
    [binary],
  );
  const provider = await verifyPublicProvider(version, candidate, image(binary));
  record.json("platform-provider.json", provider);
  const cliEntry = inside(root, require.resolve("vize"));
  const cliManifest = asRecord(
    parseJson(readFileSync(join(dirname(cliEntry), "..", "package.json"), "utf8")),
  );
  assert.equal(cliManifest.name, "vize");
  assert.equal(cliManifest.version, version);
  const cli = inside(root, join(dirname(cliEntry), "..", asString(asRecord(cliManifest.bin).vize)));
  const plugin = inside(root, require.resolve("oxlint-plugin-vize"));
  const eslint = inside(root, require.resolve("oxlint-plugin-vize/eslint"));
  const parser = inside(root, require.resolve("vue-eslint-parser"));
  const oxlintPackage = asRecord(
    parseJson(readFileSync(require.resolve("oxlint/package.json"), "utf8")),
  );
  const oxlint = inside(
    root,
    join(
      dirname(require.resolve("oxlint/package.json")),
      asString(asRecord(oxlintPackage.bin).oxlint),
    ),
  );
  const observedIdentity = {
    ...identity,
    binary: image(binary),
    nativeEntry: image(nativeEntry),
    cli: image(cli),
    plugin: image(plugin),
    eslint: image(eslint),
  };
  record.json("installed-identity.json", observedIdentity);
  const context = {
    version,
    root,
    proof,
    record,
    native,
    binary,
    identity: observedIdentity,
    cli,
    plugin,
    eslint,
    parser,
    oxlint,
    receipts,
  };
  preserve(context);
  return context;
}
