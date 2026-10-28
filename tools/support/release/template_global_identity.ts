import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, isAbsolute, join, relative } from "node:path";
import { asArray, asRecord, asString, parseJson, readJsonRecord } from "../json-value.ts";
import type * as Native from "../../../npm/native/index.js";

const sha = (bytes: string | Buffer): string => createHash("sha256").update(bytes).digest("hex");
function api(path: string): Record<string, unknown> {
  const result = spawnSync("gh", ["api", "repos/ubugeeei-prod/vize/" + path], {
    encoding: "utf8",
    maxBuffer: 2 * 1024 * 1024,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stderr);
  return asRecord(parseJson(result.stdout));
}
function ancestor(base: string, head: string): void {
  assert.match(base, /^[a-f0-9]{40}$/);
  assert.match(head, /^[a-f0-9]{40}$/);
  assert.ok(
    ["ahead", "identical"].includes(asString(api("compare/" + base + "..." + head).status)),
  );
}

/** The candidate is exclusively a fresh public package, authenticated against its producer. */
export function loadPublicTemplateCompiler() {
  const packageFile = process.env.VIZE_TEMPLATE_GLOBAL_PUBLIC_PACKAGE;
  const version = process.env.VIZE_TEMPLATE_GLOBAL_PUBLIC_VERSION;
  const tagSource = process.env.VIZE_TEMPLATE_GLOBAL_PUBLIC_SOURCE;
  const authorityBlob = process.env.VIZE_TEMPLATE_GLOBAL_PUBLIC_AUTHORITY;
  assert.ok(packageFile && isAbsolute(packageFile));
  assert.ok(version && /^[0-9]+\.[0-9]+\.[0-9]+$/.test(version));
  assert.ok(tagSource && /^[a-f0-9]{40}$/.test(tagSource));
  assert.ok(authorityBlob && /^[a-f0-9]{40}$/.test(authorityBlob));
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  for (const key of ["NODE_PATH", "NAPI_RS_NATIVE_LIBRARY_PATH", "VIZE_NATIVE_PATH"])
    assert.equal(process.env[key], undefined, key);
  const producer = asRecord(api("git/blobs/" + authorityBlob));
  assert.equal(producer.encoding, "base64");
  const authorityRaw = Buffer.from(asString(producer.content), "base64");
  const authority = asRecord(parseJson(authorityRaw.toString("utf8")));
  assert.equal(authority.schema, "vize.publicTemplateGlobalReleaseAuthority.v1");
  assert.equal(authority.version, version);
  assert.equal(authority.tagSourceSha, tagSource);
  const producerSource = asString(authority.producerSource);
  const pr = api("pulls/10973");
  assert.equal(
    pr.merged,
    true,
    "the public compiler policy must include the actual protected merge",
  );
  const requiredMerge = asString(pr.merge_commit_sha);
  assert.equal(authority.required10973MergeSha, requiredMerge);
  ancestor(requiredMerge, producerSource);
  const evidence = asRecord(authority.producerEvidence);
  const runId = evidence.runId;
  assert.ok(typeof runId === "number" && Number.isSafeInteger(runId) && runId > 0);
  const workflowHead = asString(evidence.workflowHead);
  assert.match(workflowHead, /^[a-f0-9]{40}$/);
  ancestor(producerSource, workflowHead);
  const producerRun = api("actions/runs/" + runId);
  assert.equal(producerRun.id, runId);
  assert.equal(producerRun.head_sha, workflowHead);
  assert.equal(producerRun.path, ".github/workflows/release.yml");
  assert.equal(producerRun.event, "workflow_dispatch");
  const prepared = api("git/commits/" + tagSource);
  assert.deepEqual(
    asArray(prepared.parents).map((value) => asRecord(value).sha),
    [producerSource],
  );
  ancestor(producerSource, tagSource);
  const componentBlob = asString(evidence.componentBlob);
  assert.match(componentBlob, /^[a-f0-9]{40}$/);
  const componentObject = api("git/blobs/" + componentBlob);
  assert.equal(componentObject.encoding, "base64");
  const componentRaw = Buffer.from(asString(componentObject.content), "base64");
  const component = asRecord(parseJson(componentRaw.toString("utf8")));
  assert.equal(component.schema, "vize.originalActionsProviderReceipts.v1");
  assert.equal(component.version, version);
  assert.equal(component.producerSource, producerSource);
  assert.equal(component.tagSourceSha, tagSource);
  const componentEvidence = asRecord(component.producerEvidence);
  assert.equal(componentEvidence.runId, runId);
  assert.equal(componentEvidence.workflowHead, workflowHead);
  assert.deepEqual(asArray(authority.receipts), asArray(component.receipts));
  let tag = asRecord(api("git/ref/tags/v" + version).object);
  if (tag.type === "tag") tag = asRecord(api("git/tags/" + asString(tag.sha)).object);
  assert.equal(tag.type, "commit");
  assert.equal(tag.sha, tagSource);

  const fromPublic = createRequire(packageFile);
  const prefix = dirname(packageFile);
  const sourceRoot = process.env.GITHUB_WORKSPACE;
  assert.ok(sourceRoot && isAbsolute(sourceRoot));
  const outsideSource = relative(sourceRoot, prefix);
  assert.ok(outsideSource.startsWith("..") || isAbsolute(outsideSource));
  const lock = asRecord(readJsonRecord(join(prefix, "package-lock.json")).packages);
  const sri = asRecord(authority.npmSRI);
  assert.equal(Object.keys(sri).length, 10);
  const packages: Record<string, unknown> = {};
  for (const name of ["vize", "@vizejs/native", "@vizejs/native-linux-x64-gnu"]) {
    const manifest = readJsonRecord(join(prefix, "node_modules", name, "package.json"));
    assert.equal(manifest.version, version, name);
    const entry = asRecord(lock["node_modules/" + name]);
    const expected = asRecord(sri[name]);
    assert.equal(entry.version, version);
    assert.match(asString(expected.integrity), /^sha512-[A-Za-z0-9+/]+=*$/);
    assert.ok(asString(expected.tarball).startsWith("https://registry.npmjs.org/"));
    assert.equal(asString(entry.integrity), asString(expected.integrity));
    assert.equal(asString(entry.resolved), asString(expected.tarball));
    packages[name] = entry;
  }
  const candidate = fromPublic("@vizejs/native") as typeof Native;
  const filename = fromPublic.resolve("@vizejs/native-linux-x64-gnu");
  assert.ok(fromPublic.cache[filename], "the actual public image must be loaded");
  const loadedImages = Object.keys(fromPublic.cache).filter(
    (path) => path.endsWith(".node") && path.includes("vize-vitrine."),
  );
  assert.deepEqual(loadedImages, [filename]);
  const bytes = readFileSync(filename);
  const image = { filename, bytes: bytes.length, sha256: sha(bytes) };
  const receipts = asArray(authority.receipts).map((value) => {
    const record = asRecord(value);
    const origin = asRecord(record.origin);
    assert.equal(origin.runId, runId);
    assert.equal(origin.workflowHead, workflowHead);
    assert.ok(
      typeof origin.artifactId === "number" &&
        Number.isSafeInteger(origin.artifactId) &&
        origin.artifactId > 0,
    );
    assert.equal(
      origin.artifactApi,
      "https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/" + origin.artifactId,
    );
    assert.ok(asString(origin.member).endsWith(".vsix.receipt.json"));
    const raw = Buffer.from(asString(record.rawBase64), "base64");
    assert.equal(sha(raw), record.sha256);
    const receipt = asRecord(parseJson(raw.toString("utf8")));
    assert.equal(receipt.schema, "vize.platformVsix.v1");
    assert.equal(receipt.version, version);
    assert.equal(receipt.source, tagSource);
    return { record, receipt };
  });
  assert.equal(receipts.length, 8);
  const matching = receipts.filter(
    ({ receipt }) => asRecord(receipt.provider).nodeFile === "vize-vitrine.linux-x64-gnu.node",
  );
  assert.equal(matching.length, 1);
  const match = matching[0];
  assert.ok(match);
  const provider = asRecord(match.receipt.provider);
  assert.equal(provider.schema, "vize.vscodeNativeProvider.v1");
  assert.equal(provider.targetPlatform, "linux-x64");
  assert.equal(provider.rustTarget, "x86_64-unknown-linux-gnu");
  assert.equal(provider.source, tagSource);
  assert.equal(provider.releaseVersion, version);
  const files = asArray(provider.files).map((value) => asRecord(value));
  const native = files.filter((file) => file.path === provider.nodeFile);
  assert.equal(native.length, 1);
  assert.equal(native[0]?.bytes, image.bytes);
  assert.equal(native[0]?.sha256, image.sha256);
  const token = candidate.getNativeBuildToken();
  assert.ok(token && /^[a-f0-9]{32}$/.test(token));
  assert.equal(token, provider.nativeBuildToken);
  const binding = fromPublic("@vizejs/native/native-binding-identity.js") as {
    getBindingIdentity(value: unknown): unknown;
  };
  const identity = asRecord(binding.getBindingIdentity(candidate));
  assert.equal(identity.kind, "native");
  assert.equal(identity.token, token);
  return {
    candidate,
    token,
    identity,
    image,
    evidence: {
      mode: "fresh-public-sdk",
      version,
      tagSource,
      producerSource,
      producerEvidence: evidence,
      originalComponent: { blob: componentBlob, sha256: sha(componentRaw) },
      required10973Merge: requiredMerge,
      authorityBlob,
      authoritySha256: sha(authorityRaw),
      originalProducerReceipt: match.record,
      packages,
      lockSha256: sha(readFileSync(join(prefix, "package-lock.json"))),
      image,
      identity,
    },
  };
}
