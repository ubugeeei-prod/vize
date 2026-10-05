import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import type {
  PublishedAuthority,
  PublishedReceipt,
} from "../../tooling/support/lsp/published-launch.ts";
import { sha256 } from "./warm-type-backed-source.ts";
import { unpackPublicArchive } from "./warm-type-backed-public-archives.ts";

const apiRoot = "https://api.github.com/repos/ubugeeei-prod/vize";
const headers = () => ({
  Accept: "application/vnd.github+json",
  Authorization: `Bearer ${process.env.GH_TOKEN!}`,
});

export async function officialJson(endpoint: string, destination: string) {
  assert.ok(process.env.GH_TOKEN);
  const response = await fetch(`${apiRoot}/${endpoint}`, { headers: headers() });
  const bytes = Buffer.from(await response.arrayBuffer());
  fs.writeFileSync(destination, bytes);
  fs.writeFileSync(
    `${destination}.http.json`,
    JSON.stringify({
      url: response.url,
      status: response.status,
      bytes: bytes.length,
      sha256: sha256(bytes),
    }),
  );
  assert.equal(response.status, 200);
  assert.ok(bytes.length < 16 * 1024 * 1024);
  return JSON.parse(bytes.toString("utf8"));
}

async function download(
  url: string,
  destination: string,
  size: number,
  digest: string,
  authenticated = false,
) {
  assert.ok(Number.isSafeInteger(size) && size > 0 && size < 256 * 1024 * 1024);
  const response = await fetch(
    url,
    authenticated ? { headers: { ...headers(), Accept: "application/vnd.github+json" } } : {},
  );
  fs.writeFileSync(
    `${destination}.http.json`,
    JSON.stringify({ url: response.url, status: response.status }),
  );
  assert.equal(response.status, 200);
  assert.ok(response.body);
  const file = fs.openSync(destination, "wx");
  const hash = createHash("sha256");
  let bytes = 0;
  try {
    for await (const chunk of response.body) {
      const content = Buffer.from(chunk);
      bytes += content.length;
      fs.writeFileSync(file, content);
      hash.update(content);
      assert.ok(bytes <= size, "official payload exceeded its frozen size");
    }
  } finally {
    fs.closeSync(file);
    fs.writeFileSync(
      `${destination}.digest.json`,
      JSON.stringify({ bytes, sha256: hash.digest("hex") }),
    );
  }
  const measured = JSON.parse(fs.readFileSync(`${destination}.digest.json`, "utf8"));
  assert.equal(measured.bytes, size);
  assert.equal(measured.sha256, digest);
}

export async function installPublicAsset(
  authority: PublishedAuthority,
  output: string,
): Promise<PublishedReceipt> {
  const release = await officialJson(
    `releases/tags/v${authority.releaseVersion}`,
    path.join(output, "release.json"),
  );
  const tagCommit = await officialJson(
    `commits/v${authority.releaseVersion}`,
    path.join(output, "tag-commit.json"),
  );
  assert.equal(release.draft, false);
  assert.equal(release.tag_name, `v${authority.releaseVersion}`);
  assert.ok(release.published_at);
  assert.equal(tagCommit.sha, authority.cut.sha);
  assert.equal(tagCommit.commit.tree.sha, authority.cut.tree);
  const selected = release.assets.filter(
    (entry: { name: string }) => entry.name === "vize-x86_64-unknown-linux-gnu.tar.gz",
  );
  assert.equal(selected.length, 1);
  const asset = selected[0];
  assert.equal(asset.id, authority.asset.id);
  assert.equal(asset.size, authority.asset.size);
  assert.equal(asset.digest, `sha256:${authority.asset.sha256}`);
  assert.equal(
    asset.browser_download_url,
    `https://github.com/ubugeeei-prod/vize/releases/download/v${authority.releaseVersion}/${asset.name}`,
  );
  const archive = path.join(output, "public-cli.tar.gz");
  await download(asset.browser_download_url, archive, authority.asset.size, authority.asset.sha256);
  const directory = path.join(output, "public-install");
  const audit = unpackPublicArchive("tar", archive, directory);
  assert.equal(audit.members.length, 1);
  return {
    schema: "vize.original400.published.installation",
    version: 1,
    authority,
    release,
    tagCommit,
    asset,
    downloadSha256: authority.asset.sha256,
    installed: {
      path: fs.realpathSync(path.join(directory, "vize")),
      sha256: audit.members[0].sha256,
      archiveMember: audit.members[0].path,
    },
  };
}

export async function loadSourceReference(authority: PublishedAuthority, output: string) {
  const reference = authority.sourceArtifact;
  const official = await officialJson(
    `actions/artifacts/${reference.id}`,
    path.join(output, "source-artifact.json"),
  );
  assert.equal(official.expired, false);
  assert.equal(official.size_in_bytes, reference.size);
  assert.equal(official.digest, `sha256:${reference.sha256}`);
  assert.equal(official.workflow_run.id, reference.run);
  assert.equal(official.workflow_run.head_sha, reference.driverRevision);
  const run = await officialJson(
    `actions/runs/${reference.run}`,
    path.join(output, "source-run.json"),
  );
  assert.equal(run.conclusion, "success");
  assert.equal(run.name, "Davinci Canon distinct-host scaling");
  assert.equal(run.head_sha, reference.driverRevision);
  assert.equal(run.event, "workflow_dispatch");
  const archive = path.join(output, "source-reference.zip");
  await download(
    `${apiRoot}/actions/artifacts/${reference.id}/zip`,
    archive,
    reference.size,
    reference.sha256,
    true,
  );
  const directory = path.join(output, "source-reference");
  unpackPublicArchive("zip", archive, directory);
  return path.join(directory, "warm-pair");
}
