import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
const tag = requiredEnv("RELEASE_TAG"),
  required = process.env.REQUIRED_SOURCE,
  target = requiredEnv("TARGET_PLATFORM");
const channel = requiredEnv("PUBLIC_CHANNEL");
assert(/^v\d+\.\d+\.\d+$/.test(tag), "Use an exact release tag");
assert(!required || /^[a-f0-9]{40}$/.test(required), "Use a full required commit");
assert(["github-release", "open-vsx"].includes(channel));
const directory = path.join(requiredEnv("RUNNER_TEMP"), "typescript-provider-public");
fs.mkdirSync(directory, { recursive: true });
const vsix = `vize-${target}.vsix`;
const assets: {
  name: string;
  bytes: number;
  sha256: string;
  url?: string;
  origin?: string;
  run?: number;
  artifact?: string;
}[] = [];
let producerWorkflow = null;
async function download(name: string, url: string, headers = {}) {
  const response = await fetch(url, { headers, signal: AbortSignal.timeout(120_000) });
  assert(response.ok, `${url}: HTTP ${response.status}`);
  const bytes = Buffer.from(await response.arrayBuffer());
  fs.writeFileSync(path.join(directory, name), bytes);
  assets.push({
    name,
    url,
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  });
  return bytes;
}
if (channel === "github-release") {
  const base = `https://github.com/ubugeeei-prod/vize/releases/download/${tag}/`;
  for (const name of [
    vsix,
    `${vsix}.receipt.json`,
    `${vsix}.sha256`,
    `${vsix}.receipt.json.sha256`,
  ])
    await download(name, base + name);
  for (const name of [vsix, `${vsix}.receipt.json`]) {
    const checksum = fs.readFileSync(path.join(directory, `${name}.sha256`), "utf8").trim();
    const [digest, filename] = checksum.split(/\s+/);
    assert.equal(filename, name);
    assert.equal(assets.find((asset) => asset.name === name)?.sha256, digest);
  }
} else {
  assert(
    /^[1-9]\d*$/.test(requiredEnv("PRODUCER_RUN")),
    "Open VSX requires the original producer run",
  );
  const runUrl = `https://api.github.com/repos/ubugeeei-prod/vize/actions/runs/${process.env.PRODUCER_RUN}`;
  const run = JSON.parse(
    (
      await download("producer-run.json", runUrl, {
        Authorization: `Bearer ${process.env.GITHUB_READ_TOKEN}`,
        Accept: "application/vnd.github+json",
      })
    ).toString("utf8"),
  );
  assert.equal(run.repository.full_name, "ubugeeei-prod/vize");
  assert.equal(run.path, ".github/workflows/release.yml");
  producerWorkflow = { run: run.id, workflowHead: run.head_sha, path: run.path, url: run.html_url };
  const base = `https://open-vsx.org/api/ubugeeei/vize/${target}/${tag.slice(1)}`;
  const metadata = JSON.parse((await download("openvsx-metadata.json", base)).toString("utf8"));
  assert.equal(metadata.namespace, "ubugeeei");
  assert.equal(metadata.name, "vize");
  assert.equal(metadata.version, tag.slice(1));
  assert.equal(metadata.targetPlatform, target);
  assert.equal(metadata.downloadable, true);
  const downloadUrl = new URL(metadata.files.download);
  const checksumUrl = new URL(metadata.files.sha256);
  for (const url of [downloadUrl, checksumUrl]) {
    assert.equal(url.origin, "https://open-vsx.org");
    assert(url.pathname.startsWith(`${new URL(base).pathname}/file/`));
  }
  // Only the public registry response supplies the installed VSIX bytes.
  await download(vsix, downloadUrl.href);
  const checksum = (await download(`${vsix}.openvsx.sha256`, checksumUrl.href))
    .toString("utf8")
    .trim();
  assert.equal(assets.find((asset) => asset.name === vsix)?.sha256, checksum);
  const sidecar = `${vsix}.receipt.json`;
  const original = path.join(requiredEnv("RUNNER_TEMP"), "typescript-provider-producer", sidecar);
  const bytes = fs.readFileSync(original);
  fs.writeFileSync(path.join(directory, sidecar), bytes);
  // This is the genuine Actions producer sidecar, not a registry sidecar.
  // Copy its original bytes and record that distinct provenance explicitly.
  assets.push({
    name: sidecar,
    origin: "original-actions-producer-artifact",
    run: Number(process.env.PRODUCER_RUN),
    artifact: `editor-extension-platform-${target}`,
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  });
}
const producer = JSON.parse(fs.readFileSync(path.join(directory, `${vsix}.receipt.json`), "utf8"));
assert.equal(producer.schema, "vize.platformVsix.v1");
assert(/^[a-f0-9]{40}$/.test(producer.source));
assert.equal(producer.version, tag.slice(1));
assert.equal(producer.target, target);
assert.equal(producer.vsix, vsix);
const archive = assets.find((asset) => asset.name === vsix);
assert.ok(archive);
assert.equal(producer.sha256, archive.sha256);
assert.equal(producer.bytes, archive.bytes);
assert.equal(producer.provider.schema, "vize.vscodeNativeProvider.v1");
assert.equal(producer.provider.source, producer.source);
assert.equal(producer.provider.releaseVersion, producer.version);
assert.equal(producer.provider.targetPlatform, target);
// The source checkout follows the genuine producer identity. Tag, source,
// public channel and producer-sidecar provenance remain distinct evidence.
fs.writeFileSync(
  path.join(directory, "public-download.json"),
  JSON.stringify(
    {
      tag,
      channel,
      workflowSource: process.env.GITHUB_SHA,
      requiredSource: required || null,
      producerSource: producer.source,
      producerWorkflow,
      assets,
    },
    null,
    2,
  ),
);
fs.appendFileSync(
  requiredEnv("GITHUB_OUTPUT"),
  `source=${producer.source}\nvsix=${path.join(directory, vsix).replaceAll("\\", "/")}\n`,
);
