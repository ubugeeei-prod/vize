import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { apiRoot, gitBlob, sha256 } from "./custody.mjs";
import { reservedScenarios, reservedNormalized } from "./reserved_scenarios.mjs";

export const reservedFixtureDirectory = "tests/_fixtures/differential/lint/released-reserved-8090";
export const reservedManifestHash =
  "108fb93819bad4dc59cace1441a955a55aaa5b5b122aeb5a0602a2c987d649df";
const manifestBytes = fs.readFileSync(
  new URL("../../../../" + reservedFixtureDirectory + "/authorities.json", import.meta.url),
);
assert.equal(sha256(manifestBytes), reservedManifestHash);
export const reservedManifest = JSON.parse(manifestBytes.toString("utf8"));
export const reservedContract = reservedManifest.contract;
assert.equal(reservedManifest.authorities.length, 97);
assert.equal(reservedManifest.sourceCount, 82);
assert.equal(reservedContract.processes, 443);
assert.deepEqual(reservedContract.suites, {
  "reserved-bare": 113,
  "reserved-prefix": 161,
  "reserved-alias": 169,
});
assert.equal(
  reservedContract.sha256,
  "a8ecd6d5638eb36b0b0853e6078df2a33c0665316e622df1950bf4ca3191dfdd",
);

/** Fetch every original authority from H; the inert archive is never imported here. */
export async function loadReservedFixtures(head, destination, json, treeSha) {
  assert.match(head, /^[0-9a-f]{40}$/);
  assert.match(treeSha ?? "", /^[0-9a-f]{40}$/);
  const tree = await json(`${apiRoot}/git/trees/${treeSha}?recursive=1`);
  assert.equal(tree.sha, treeSha);
  assert.equal(tree.truncated, false);
  const files = new Map(tree.tree.map((entry) => [entry.path, entry]));
  assert.equal(files.size, tree.tree.length, "unambiguous complete H tree required");
  const bodies = new Map(),
    authorities = [];
  async function read(authority) {
    const entry = files.get(authority.path);
    assert(entry && entry.type === "blob" && ["100644", "100755"].includes(entry.mode));
    const blob = await json(`${apiRoot}/git/blobs/${entry.sha}`);
    assert.equal(blob.encoding, "base64");
    assert.equal(blob.sha, entry.sha);
    // The existing core JSON is 477353 bytes. Every authority has its own exact
    // authored size; no broad or response-derived byte allowance is used.
    assert.equal(blob.size, authority.bytes);
    const bytes = Buffer.from(blob.content, "base64");
    assert.equal(bytes.length, authority.bytes);
    assert.equal(gitBlob(bytes), entry.sha);
    assert.equal(
      sha256(bytes),
      authority.sha256,
      `original reserved authority changed: ${authority.path}`,
    );
    const file = path.join(destination, authority.path);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, bytes, { flag: "wx" });
    bodies.set(authority.path, bytes);
    return {
      path: authority.path,
      source: head,
      blob: entry.sha,
      bytes: bytes.length,
      sha256: authority.sha256,
    };
  }
  for (let offset = 0; offset < reservedManifest.authorities.length; offset += 8) {
    const batch = reservedManifest.authorities.slice(offset, offset + 8);
    const results = await Promise.allSettled(batch.map(read));
    const errors = [];
    for (const result of results) {
      if (result.status === "fulfilled") authorities.push(result.value);
      else errors.push(result.reason);
    }
    if (errors.length) throw new AggregateError(errors, "complete original H authorities required");
  }
  assert.equal(authorities.length, 97);
  const { vectors, sourcePaths } = reservedScenarios((name) => bodies.get(name));
  assert.equal(sourcePaths.length, 82);
  const helper = await import(
    pathToFileURL(path.join(destination, "tests/tooling/lint-default-correctness-reference.mjs"))
      .href
  );
  const normalized = reservedNormalized(vectors, helper.wholeJson);
  assert.equal(normalized.length, 443);
  assert.equal(
    sha256(JSON.stringify(normalized)),
    reservedContract.sha256,
    "original complete reserved vectors changed",
  );
  return { vectors, authorities, wholeJson: helper.wholeJson };
}
