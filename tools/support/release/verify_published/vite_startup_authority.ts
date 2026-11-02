import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, lstatSync, readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { readTarGz } from "../../archive/tar.ts";
import { provenancePayloadBinding, publicUrl, sriSha512 } from "../public_install/registry.ts";
export type ObjectValue = Record<string, unknown>;
export const hash = (raw: string | Buffer) => createHash("sha256").update(raw).digest("hex");
export const object = (v: unknown): ObjectValue => {
  assert(v && typeof v === "object" && !Array.isArray(v));
  return v as ObjectValue;
};
export const json = (raw: Buffer) =>
  object(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
export const contract = json(readFileSync(new URL("./vite-startup-reports.json", import.meta.url)));
export const target = object(contract.target);
const packages = ["vize", "@vizejs/vite-plugin", "@vizejs/native", "@vizejs/native-linux-x64-gnu"];
export function authenticateCapsule(value: unknown) {
  const capsule = object(value);
  assert.equal(capsule.version, target.version);
  assert.equal(capsule.tagSourceSha, target.candidate);
  assert.equal(capsule.producerSource, target.producerSource);
  const sri = object(capsule.npmSRI);
  for (const name of packages) {
    const row = object(sri[name]);
    sriSha512(row.integrity);
    publicUrl(String(row.tarball));
  }
  assert(Array.isArray(capsule.receipts));
  assert.equal(capsule.receipts.length, 4);
  const seen = new Set<string>();
  let linux: ObjectValue | undefined;
  for (const entry of capsule.receipts) {
    const row = object(entry),
      raw = Buffer.from(String(row.rawBase64), "base64");
    assert.equal(raw.toString("base64"), row.rawBase64);
    assert.equal(hash(raw), row.sha256);
    const receipt = json(raw),
      platform = String(receipt.target),
      origin = object(row.origin);
    assert(!seen.has(platform));
    seen.add(platform);
    assert.equal(receipt.schema, "vize.platformVsix.v1");
    assert.equal(receipt.source, target.candidate);
    assert.equal(receipt.version, target.version);
    assert.equal(String(origin.runId), target.runId);
    assert(Number.isSafeInteger(origin.artifactId));
    assert.equal(origin.member, "vize-" + platform + ".vsix.receipt.json");
    const provider = object(receipt.provider);
    assert.equal(provider.source, target.candidate);
    assert.equal(provider.releaseVersion, target.version);
    if (platform === "linux-x64") linux = provider;
  }
  assert.deepEqual([...seen].sort(), ["darwin-arm64", "darwin-x64", "linux-x64", "win32-x64"]);
  assert(linux);
  assert.equal(linux.schema, "vize.vscodeNativeProvider.v1");
  assert.equal(linux.rustTarget, "x86_64-unknown-linux-gnu");
  assert.equal(linux.targetPlatform, "linux-x64");
  return { capsule, sri, provider: linux };
}
export function safePath(root: string, path: string) {
  assert(path && !isAbsolute(path) && !path.split("/").some((p) => p === ".." || p === ""));
  const file = resolve(root, path);
  assert(!relative(root, file).startsWith(".."));
  return file;
}
export function image(path: string) {
  const raw = readFileSync(path);
  return { path, bytes: raw.length, sha256: hash(raw) };
}
export function packageRoot(
  project: string,
  name: string,
  importer = join(project, "package.json"),
) {
  const req = createRequire(importer);
  let path: string;
  try {
    path = req.resolve(name + "/package.json");
  } catch {
    path = req.resolve(name);
  }
  for (let root = dirname(realpathSync(path)); ;) {
    const manifest = join(root, "package.json");
    if (existsSync(manifest) && json(readFileSync(manifest)).name === name) {
      assert(root.startsWith(realpathSync(join(project, "node_modules")) + "/"));
      return root;
    }
    const parent = dirname(root);
    assert.notEqual(parent, root);
    root = parent;
  }
}
export function packageAuthenticator(
  sri: ObjectValue,
  observed: typeof fetch,
  save: (name: string, value: unknown) => void,
) {
  const payloadCache = new Map<
    string,
    { files: Array<{ path: string; bytes: number; sha256: string }>; manifest: ObjectValue }
  >();
  return async (project: string, name: string, expectedVersion: string, root: string) => {
    const key = name + "@" + expectedVersion;
    let payload = payloadCache.get(key);
    if (!payload) {
      const response = await observed(
        "https://registry.npmjs.org/" + encodeURIComponent(name) + "/" + expectedVersion,
        { redirect: "error", signal: AbortSignal.timeout(30_000) },
      );
      assert.equal(response.status, 200);
      const metadata = json(Buffer.from(await response.arrayBuffer()));
      assert.equal(metadata.name, name);
      assert.equal(metadata.version, expectedVersion);
      const dist = object(metadata.dist),
        integrity = String(dist.integrity);
      sriSha512(integrity);
      if (packages.includes(name)) {
        const expected = object(sri[name]);
        assert.equal(integrity, expected.integrity);
        assert.equal(dist.tarball, expected.tarball);
        const provenance = await provenancePayloadBinding(
          dist,
          { H: String(target.candidate), tag: "v" + target.version, R: String(target.runId) },
          async (url) => {
            const r = await observed(publicUrl(url), {
              redirect: "error",
              signal: AbortSignal.timeout(30_000),
            });
            assert.equal(r.status, 200);
            return json(Buffer.from(await r.arrayBuffer()));
          },
        );
        save("provenance-" + name.replaceAll("/", "-"), provenance);
      }
      const archive = await observed(publicUrl(String(dist.tarball)), {
        redirect: "error",
        signal: AbortSignal.timeout(30_000),
      });
      assert.equal(archive.status, 200);
      const tar = Buffer.from(await archive.arrayBuffer());
      assert.deepEqual(createHash("sha512").update(tar).digest(), sriSha512(integrity));
      const members = readTarGz(tar),
        seen = new Set<string>(),
        files = [];
      for (const member of members) {
        assert(member.name.startsWith("package/"));
        const path = member.name.slice(8);
        if (member.kind === "directory") continue;
        assert.equal(member.kind, "file");
        safePath(root, path);
        assert(!seen.has(path));
        seen.add(path);
        files.push({ path, bytes: member.size, sha256: hash(member.data) });
      }
      const manifest = members.find((v) => v.name === "package/package.json");
      assert(manifest);
      payload = { files, manifest: json(manifest.data) };
      payloadCache.set(key, payload);
      save("archive-" + payloadCache.size, {
        name,
        version: expectedVersion,
        integrity,
        tarball: dist.tarball,
        archiveBytes: tar.length,
        archiveSha256: hash(tar),
        ...payload,
        claim: "every published payload file; extra installed files are not classified",
      });
    }
    assert.equal(payload.manifest.name, name);
    assert.equal(payload.manifest.version, expectedVersion);
    for (const file of payload.files) {
      const path = safePath(root, file.path);
      assert(lstatSync(path).isFile());
      const actual = image(path);
      assert.equal(actual.bytes, file.bytes);
      assert.equal(actual.sha256, file.sha256);
    }
    assert(root.startsWith(realpathSync(join(project, "node_modules")) + "/"));
    return { name, version: expectedVersion, root, payload };
  };
}
