import assert from "node:assert/strict";
import { test } from "node:test";

import {
  assertPackageIsUnpublished,
  bootstrapArtifacts,
  bootstrapPackages,
  validateBootstrapManifest,
  validateDownloadedArtifact,
  validateRegistryResponse,
  validateReleaseCommit,
} from "../../legacy-tools/github/npm-bootstrap-contract.mjs";
import {
  artifactName,
  cargoToml,
  mainSha,
  packageManifest,
  packageName,
  packagePath,
  releaseRunId,
  request,
  tagName,
  tagSha,
} from "./support/npm-bootstrap.ts";

test("npm bootstrap allowlist binds one package path to one Release artifact", () => {
  assert.deepEqual([...bootstrapPackages], [[packagePath, packageName]]);
  assert.deepEqual([...bootstrapArtifacts], [[packagePath, artifactName]]);
  assert.deepEqual(request(), {
    artifactName,
    packageName,
    packagePath,
    releaseRunId,
    tagName,
    workflowSha: tagSha,
  });
});

test("npm bootstrap rejects non-main dispatches and every non-allowlisted path", () => {
  assert.throws(() => request({ workflowRef: "refs/heads/release" }), /dispatched from main/);
  for (const rejectedPath of [
    "npm/framework/nuxt",
    "npm/framework/nuxt-lint-config/..",
    "npm/framework/nuxt-lint-config\n--tag latest",
    "",
  ]) {
    assert.throws(() => request({ packagePath: rejectedPath }), /not approved/, rejectedPath);
  }
});

test("npm bootstrap validates tag, run ID, and dispatch SHA before using them", () => {
  for (const rejectedTag of [
    "1.2.3",
    "v01.2.3",
    "v1.2.3-alpha..1",
    "v1.2.3:refs/heads/main",
    "v1.2.*",
    "",
  ]) {
    assert.throws(() => request({ tagName: rejectedTag }), /strict v-prefixed SemVer/, rejectedTag);
  }
  for (const rejectedRunId of ["0", "-1", "12x", "9007199254740992", ""]) {
    assert.throws(() => request({ releaseRunId: rejectedRunId }), /positive safe integer/);
  }
  assert.throws(() => request({ workflowSha: "main" }), /GITHUB_SHA must be a full commit SHA/);
  assert.doesNotThrow(() => request({ tagName: "v1.2.3-rc.1" }));
});

test("npm bootstrap binds tag, workspace, and public package metadata to one version", () => {
  assert.equal(
    validateBootstrapManifest({
      tagName,
      tagSha,
      packagePath,
      packageName,
      cargoToml,
      packageManifest,
    }),
    "1.2.3",
  );
  assert.throws(
    () =>
      validateBootstrapManifest({
        tagName: "v1.2.4",
        tagSha,
        packagePath,
        packageName,
        cargoToml,
        packageManifest,
      }),
    /does not match workspace version/,
  );
  assert.throws(
    () =>
      validateBootstrapManifest({
        tagName,
        tagSha,
        packagePath,
        packageName,
        cargoToml,
        packageManifest: JSON.stringify({
          name: "@vizejs/wrong",
          version: "1.2.3",
          publishConfig: { access: "public" },
        }),
      }),
    /expected @vizejs\/nuxt-lint-config/,
  );
  assert.throws(
    () =>
      validateBootstrapManifest({
        tagName,
        tagSha,
        packagePath,
        packageName,
        cargoToml,
        packageManifest: JSON.stringify({ name: packageName, version: "1.2.3" }),
      }),
    /publishConfig\.access as public/,
  );
});

test("npm bootstrap requires the tag to equal dispatch SHA on main first-parent", () => {
  assert.doesNotThrow(() =>
    validateReleaseCommit({ tagSha, workflowSha: tagSha, mainSha, isOnFirstParent: true }),
  );
  assert.throws(
    () =>
      validateReleaseCommit({
        tagSha,
        workflowSha: "c".repeat(40),
        mainSha,
        isOnFirstParent: true,
      }),
    /exactly match repository dispatch SHA/,
  );
  assert.throws(
    () => validateReleaseCommit({ tagSha, workflowSha: tagSha, mainSha, isOnFirstParent: false }),
    /not on the first-parent history/,
  );
});

test("npm bootstrap binds the downloaded package manifest to preflight outputs", () => {
  assert.doesNotThrow(() =>
    validateDownloadedArtifact({
      packageManifest,
      expectedName: packageName,
      expectedVersion: "1.2.3",
    }),
  );
  assert.throws(
    () =>
      validateDownloadedArtifact({
        packageManifest,
        expectedName: packageName,
        expectedVersion: "1.2.4",
      }),
    /expected @vizejs\/nuxt-lint-config@1\.2\.4/,
  );
  assert.throws(
    () =>
      validateDownloadedArtifact({
        packageManifest: "{",
        expectedName: packageName,
        expectedVersion: "1.2.3",
      }),
    /invalid package\.json/,
  );
});

test("npm bootstrap proceeds only on an authoritative registry 404", async () => {
  assert.doesNotThrow(() => validateRegistryResponse(packageName, 404));
  assert.throws(() => validateRegistryResponse(packageName, 200), /already exists on npm/);
  assert.throws(() => validateRegistryResponse(packageName, 500), /returned HTTP 500/);

  let requestedUrl = "";
  await assertPackageIsUnpublished(packageName, async (url) => {
    requestedUrl = String(url);
    return new Response(null, { status: 404 });
  });
  assert.equal(requestedUrl, "https://registry.npmjs.org/%40vizejs%2Fnuxt-lint-config");
});
