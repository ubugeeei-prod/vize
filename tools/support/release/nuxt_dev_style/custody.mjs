import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { apiRoot, authenticate, gitBlob, sha256 } from "../typed_defaults/custody.mjs";
import { reportOptions } from "../lint_reports/custody.mjs";
export { reportOptions };
export const fixMerge = "ebbc40dad6187ad17048885f9208b3972848d77f";
export const fixtureRoot = "tests/_fixtures/_projects/nuxt-dev-ssr-style";
export const pins = JSON.parse(fs.readFileSync(new URL("./pins.json", import.meta.url), "utf8"));
export async function authenticateNuxt(options, json) {
  const official = await authenticate(options, json);
  const fix = await json(`${apiRoot}/pulls/8602`);
  assert.equal(fix.number, 8602);
  assert.equal(fix.merged, true);
  assert.equal(fix.merge_commit_sha, fixMerge);
  assert.equal(fix.base.repo.full_name, "ubugeeei-prod/vize");
  assert.equal(fix.head.repo.full_name, "ubugeeei-prod/vize");
  const ancestry = await json(`${apiRoot}/compare/${fixMerge}...${options.head}`);
  assert.equal(ancestry.base_commit.sha, fixMerge);
  assert.equal(ancestry.merge_base_commit.sha, fixMerge);
  assert(["ahead", "identical"].includes(ancestry.status));
  return { official, fix, ancestry };
}
export async function loadNuxtFixtures(head, treeSha, destination, json) {
  const tree = await json(`${apiRoot}/git/trees/${treeSha}?recursive=1`);
  assert.equal(tree.sha, treeSha);
  assert.equal(tree.truncated, false);
  const entries = new Map(tree.tree.map((entry) => [entry.path, entry]));
  const source = new Map(),
    authorities = [];
  for (const [name, pin] of Object.entries(pins)) {
    const entry = entries.get(name);
    assert(entry?.type === "blob" && entry.mode === "100644");
    const blob = await json(`${apiRoot}/git/blobs/${entry.sha}`);
    assert.equal(blob.encoding, "base64");
    const bytes = Buffer.from(blob.content, "base64");
    assert(bytes.length <= 512 * 1024);
    assert.equal(blob.sha, entry.sha);
    assert.equal(blob.size, bytes.length);
    assert.equal(gitBlob(bytes), entry.sha);
    assert.equal(sha256(bytes), pin, `original Nuxt fixture changed: ${name}`);
    const filename = path.join(destination, name);
    fs.mkdirSync(path.dirname(filename), { recursive: true });
    fs.writeFileSync(filename, bytes, { flag: "wx" });
    source.set(name, bytes);
    authorities.push({
      path: name,
      source: head,
      blob: entry.sha,
      bytes: bytes.length,
      sha256: pin,
    });
  }
  return { source, authorities };
}
