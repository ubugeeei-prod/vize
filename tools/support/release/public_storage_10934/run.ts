import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { loadAuthority } from "./authority.ts";
import { bootstrap, NATIVE_PACKAGE } from "./provider.ts";
import { attestGoExecutions } from "./trace.ts";
import { validatePublicProvider } from "../public-provider.ts";
import {
  asArray, asNumber, asRecord, asString, blob, env, inside, read, run, save, sha256,
  HARNESS_BLOB, HARNESS_SHA, REQUIRED_MERGE,
} from "./io.ts";

const root = path.resolve(env("PUBLIC_STORAGE_ROOT"));
assert.equal(fs.existsSync(root), false, "acceptance attempt must own a fresh root");
fs.mkdirSync(root);
const evidence = path.join(root, "evidence");
fs.mkdirSync(evidence);
try {
  const version = env("PUBLIC_STORAGE_VERSION");
  const candidate = env("PUBLIC_STORAGE_CANDIDATE");
  const authorityBlob = env("PUBLIC_STORAGE_AUTHORITY_BLOB");
  assert.match(version, /^0\.\d+\.0$/);
  assert.match(candidate, /^[a-f0-9]{40}$/);
  assert.match(authorityBlob, /^[a-f0-9]{40}$/);
  const source = fs.realpathSync(env("PUBLIC_STORAGE_SOURCE"));
  const checkout = run("git", ["rev-parse", "HEAD"], source, path.join(evidence, "source-checkout"));
  assert.equal(checkout.stdout.trim(), candidate);
  const authority = await loadAuthority(version, candidate, authorityBlob, evidence);
  const { provider, backend, backendIdentity } = await bootstrap(root, evidence, authority);
  const scriptBytes = await blob(HARNESS_BLOB);
  assert.equal(sha256(scriptBytes), HARNESS_SHA);
  const script = path.join(root, "acceptance.py");
  fs.writeFileSync(script, scriptBytes);
  const fixture = path.join(source, "tests/_fixtures/differential/typechecker/relative-sibling-storage-10934");
  const harnessOutput = path.join(evidence, "harness");
  const cache = path.join(root, "cache");
  assert.equal(fs.existsSync(cache), false);
  fs.mkdirSync(cache);
  const environment = { ...process.env };
  for (const name of [
    "NAPI_RS_NATIVE_LIBRARY_PATH", "NAPI_RS_FORCE_WASI", "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
    "NODE_OPTIONS", "NODE_PATH",
  ]) delete environment[name];
  environment.XDG_CACHE_HOME = cache;
  environment.CORSA_PATH = backend;
  run("strace", [
    "-ff", "-qq", "-s", "65536", "-e", "trace=execve,execveat,chdir",
    "-e", "signal=none", "-o", path.join(evidence, "exec"), "--", "python3", script, "--provider", provider, "--corsa", backend, "--fixture", fixture,
    "--mode", "fixed", "--output", harnessOutput,
  ], source, path.join(evidence, "harness-process"), environment, 240000);
  const receiptFile = path.join(harnessOutput, "receipt.json");
  const receipt = asRecord(read(receiptFile));
  assert.equal(receipt.schema, "vize.issue-10934-public-acceptance");
  assert.equal(receipt.mode, "fixed");
  assert.equal(receipt.status, "fixed-contract-accepted");
  assert.equal(receipt.scriptSha256, HARNESS_SHA);
  assert.equal(receipt.publicVizeVersion, version);
  assert.equal(receipt.publicNativeVersion, version);
  const image = asRecord(receipt.loadedPublicNativeImage);
  inside(provider, asString(image.path));
  assert.equal(image.package, NATIVE_PACKAGE);
  assert.equal(image.version, version);
  const linux = authority.providers.find((item) => item.target === "linux-x64");
  assert.ok(linux);
  const publicImage = { bytes: asNumber(image.bytes), sha256: asString(image.sha256) };
  const providerAuthority = validatePublicProvider(linux.receipt, version, candidate, publicImage);
  assert.equal(image.buildToken, linux.buildToken);
  assert.equal(asRecord(receipt.nativeBackend).sha256, backendIdentity.sha256);
  assert.equal(asRecord(receipt.nativeBackend).path, backend);
  const rows = asArray(receipt.runs).map((value) => asRecord(value));
  assert.equal(rows.length, 3);
  const error = [["src/entry.ts", "error:2:31 [TS2322] Type 'string' is not assignable to type 'number'."]];
  for (let index = 0; index < rows.length; index++) {
    const row = rows[index];
    assert.ok(row);
    assert.equal(row.index, index + 1);
    assert.equal(row.freshProcess, true);
    assert.equal(row.programRoot, ".");
    assert.equal(row.exitCode, index === 2 ? 0 : 1);
    assert.deepEqual(row.diagnostics, index === 2 ? [] : error);
    const first = rows[0];
    assert.ok(first);
    assert.deepEqual(row.preservedSourceMtimesNs, first.preservedSourceMtimesNs);
    assert.deepEqual(row.buildInfoPaths, first.buildInfoPaths);
  }
  const workspace = fs.realpathSync(path.join(harnessOutput, "project"));
  const app = fs.realpathSync(path.join(workspace, "apps/app"));
  const originalConfig = fs.realpathSync(path.join(app, "tsconfig.json"));
  const configSha256 = sha256(fs.readFileSync(originalConfig));
  const original = asArray(receipt.inputs).map((value) => asRecord(value))
    .find((item) => item.file === "apps/app/tsconfig.json");
  assert.ok(original);
  assert.equal(configSha256, original.sha256);
  const ownerSha256 = createHash("sha256")
    .update(Buffer.from("vize-canon-project-key:v1\0")).update(Buffer.from(app)).digest("hex");
  const context = createHash("sha256").update(Buffer.from("vize-persistent-installed-batch:v1\0"));
  for (const file of [workspace, app, originalConfig]) {
    const bytes = Buffer.from(file);
    const length = Buffer.alloc(8);
    length.writeBigUInt64LE(BigInt(bytes.length));
    context.update(Buffer.from([1])).update(length).update(bytes);
  }
  const contextSha256 = context.digest("hex");
  const virtualRoot = path.join(cache, "vize/canon/private-batch-v1", ownerSha256, "canon/projects", contextSha256);
  assert.equal(receipt.virtualRoot, virtualRoot);
  inside(cache, virtualRoot);
  assert.equal(sha256(fs.readFileSync(backend)), backendIdentity.sha256);
  const actualGoExecutions = attestGoExecutions(evidence, backend, virtualRoot);
  const buildInfos = asArray(rows[0]?.buildInfoPaths).map((value) => asString(value));
  assert.ok(buildInfos.length > 0);
  const finalBuildInfo = buildInfos.map((file) => ({
    path: inside(virtualRoot, file), sha256: sha256(fs.readFileSync(file)),
  }));
  const result = {
    schema: "vize.publicStorage10934Acceptance.v1",
    version, candidate, requiredMerge: REQUIRED_MERGE,
    authorityBlob, authoritySha256: authority.authoritySha256,
    producerSource: authority.source, workflowHead: authority.workflowHead,
    producerRun: authority.producerRun, harnessBlob: HARNESS_BLOB, harnessSha256: HARNESS_SHA,
    checkout: candidate, sourceFixture: fixture, receiptSha256: sha256(fs.readFileSync(receiptFile)),
    providerAuthority, loadedPublicNativeImage: image, backend: backendIdentity,
    owner: { workspace, app, originalConfig, configSha256, ownerSha256, contextSha256, virtualRoot },
    finalBuildInfo, actualGoExecutions,
    cacheRegime: {
      freshCheckProcesses: 3, firstProjectCacheInitiallyEmpty: true,
      laterProcesses: "persistent build-info and unchanged mirror reuse",
      osPageCacheFlushed: false, coldPerformanceQualification: false,
      observedWithStrace: true,
    },
    observedExitCodes: rows.map((row) => row.exitCode),
    sourceFallback: false, passed: true,
  };
  save(path.join(evidence, "public-storage-acceptance.json"), result);
  console.log(JSON.stringify({
    passed: true, version, candidate, requiredMerge: REQUIRED_MERGE,
    freshCheckProcesses: 3, nativeSha256: publicImage.sha256,
    backendSha256: backendIdentity.sha256, ownerSha256, contextSha256,
  }));
} catch (error) {
  save(path.join(evidence, "failure.json"), {
    passed: false, error: error instanceof Error ? error.stack : String(error),
  });
  throw error;
}
