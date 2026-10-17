import assert from "node:assert/strict";
import { test } from "node:test";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  extractReleaseAssets,
  packReleaseAssets,
  releaseIdentity,
} from "../../tools/support/release/release-assets.ts";

const identity = { sourceSha: "a".repeat(40), runId: "42", attempt: 1, tag: "v0.458.0" };

function fixture(t: { after: (cleanup: () => void) => void }) {
  const root = mkdtempSync(join(tmpdir(), "vize-release-assets-law-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const source = join(root, "source");
  const destination = join(root, "destination");
  for (const directory of ["cli-artifacts", "editor-extension-artifacts"]) {
    mkdirSync(join(source, directory), { recursive: true });
    for (const suffix of ["", ".sha256", ".sig", ".pem", ".cosign.bundle"]) {
      writeFileSync(join(source, directory, `product.zip${suffix}`), `signed bytes${suffix}\0`);
    }
  }
  for (const format of ["cyclonedx", "spdx"]) {
    for (const suffix of ["", ".sig", ".pem", ".cosign.bundle"]) {
      writeFileSync(join(source, `vize-${identity.tag}-${format}.sbom.json${suffix}`), format);
    }
  }
  mkdirSync(destination);
  return { source, destination, archive: join(root, "assets.tar") };
}

test("release assets retain binary, signature and checksum bytes across a failed-job rerun", async (t) => {
  const { source, destination, archive } = fixture(t);
  const digest = await packReleaseAssets(source, archive, identity);
  await extractReleaseAssets(destination, archive, digest, { ...identity, attempt: 2 });
  for (const suffix of ["", ".sha256", ".sig", ".pem", ".cosign.bundle"]) {
    const path = `cli-artifacts/product.zip${suffix}`;
    assert.deepEqual(readFileSync(join(destination, path)), readFileSync(join(source, path)));
  }
  assert.deepEqual(
    JSON.parse(readFileSync(join(destination, "release-assets.json"), "utf8")),
    identity,
  );
});

test("changed asset bytes fail before anything is extracted", async (t) => {
  const { source, destination, archive } = fixture(t);
  const digest = await packReleaseAssets(source, archive, identity);
  const changed = readFileSync(archive);
  changed[changed.length - 1] ^= 1;
  writeFileSync(archive, changed);
  await assert.rejects(extractReleaseAssets(destination, archive, digest, identity), /digest/);
  assert.equal(existsSync(join(destination, "cli-artifacts")), false);
});

test("valid bytes cannot authorize a different source, run, tag or future attempt", async (t) => {
  const { source, destination, archive } = fixture(t);
  const digest = await packReleaseAssets(source, archive, { ...identity, attempt: 2 });
  for (const foreign of [
    { ...identity, attempt: 2, sourceSha: "b".repeat(40) },
    { ...identity, attempt: 2, runId: "43" },
    { ...identity, attempt: 2, tag: "v0.459.0" },
    identity,
  ]) {
    await assert.rejects(
      extractReleaseAssets(destination, archive, digest, foreign),
      /different source, run, tag or attempt|unexpected path/,
    );
  }
  assert.equal(existsSync(join(destination, "cli-artifacts")), false);
});

test("preparation refuses symlinks and absent signature companions", async (t) => {
  const { source, archive } = fixture(t);
  const link = join(source, "cli-artifacts", "foreign");
  symlinkSync("product.zip", link);
  await assert.rejects(packReleaseAssets(source, archive, identity), /not a regular file/);
  rmSync(link);
  rmSync(join(source, `vize-${identity.tag}-spdx.sbom.json.sig`));
  await assert.rejects(packReleaseAssets(source, archive, identity), /ENOENT/);
});

test("asset identity requires authorized H to equal both event and actual checkout", () => {
  const env = {
    RELEASE_SOURCE_SHA: identity.sourceSha,
    GITHUB_SHA: identity.sourceSha,
    GITHUB_RUN_ID: identity.runId,
    GITHUB_RUN_ATTEMPT: "1",
    RELEASE_TAG_NAME: identity.tag,
  };
  assert.deepEqual(releaseIdentity(env, identity.sourceSha), identity);
  assert.throws(() => releaseIdentity(env, "b".repeat(40)), /exact checked-out source/);
  assert.throws(() => releaseIdentity({ ...env, GITHUB_SHA: "b".repeat(40) }, identity.sourceSha));
});
