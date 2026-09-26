import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { repoRoot } from "./_helpers/moonbit.ts";

import {
  assertPackageIsUnpublished,
  bootstrapArtifacts,
  bootstrapPackages,
  validateBootstrapManifest,
  validateDownloadedArtifact,
  validateRegistryResponse,
  validateReleaseCommit,
  validateReleaseControlVersion,
} from "../../tools/support/compat/github/npm-bootstrap-contract.mjs";
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

test("npm bootstrap allowlist binds each approved package path to one Release artifact", () => {
  assert.deepEqual(
    [...bootstrapPackages],
    [
      [packagePath, packageName],
      ["npm/plugin-sdk", "@vizejs/plugin-sdk"],
    ],
  );
  assert.deepEqual(
    [...bootstrapArtifacts],
    [
      [packagePath, artifactName],
      ["npm/plugin-sdk", "release-package-plugin-sdk"],
    ],
  );
  assert.deepEqual(request(), {
    artifactName,
    packageName,
    packagePath,
    releaseRunId,
    tagName,
    workflowSha: mainSha,
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

test("npm bootstrap binds controls to current main and the immutable tag to first-parent history", () => {
  assert.doesNotThrow(() =>
    validateReleaseCommit({ tagSha, workflowSha: mainSha, mainSha, isOnFirstParent: true }),
  );
  assert.doesNotThrow(() =>
    validateReleaseCommit({ tagSha, workflowSha: tagSha, mainSha: tagSha, isOnFirstParent: true }),
  );
  for (const workflowSha of [tagSha, "c".repeat(40)]) {
    assert.throws(
      () => validateReleaseCommit({ tagSha, workflowSha, mainSha, isOnFirstParent: true }),
      /exactly match current origin\/main/,
    );
  }
  for (const field of ["tagSha", "workflowSha", "mainSha"]) {
    assert.throws(
      () =>
        validateReleaseCommit({
          tagSha,
          workflowSha: mainSha,
          mainSha,
          isOnFirstParent: true,
          [field]: "main",
        }),
      /full commit SHAs/,
    );
  }
  assert.throws(
    () => validateReleaseCommit({ tagSha, workflowSha: mainSha, mainSha, isOnFirstParent: false }),
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

test("Rust and JS bootstrap controls require current main and the same tagged release version", () => {
  const invoke = (mode: string, payload: Record<string, unknown>) =>
    spawnSync(
      "rust-script",
      [
        path.join(repoRoot, "tools/commands/ci/github/npm-bootstrap-preflight.rs"),
        "__contract",
        mode,
        JSON.stringify(payload),
      ],
      { encoding: "utf8" },
    );
  const commit = { tagSha, workflowSha: mainSha, mainSha, isOnFirstParent: true };
  assert.equal(invoke("release-commit", commit).status, 0);
  for (const changed of [
    { workflowSha: tagSha },
    { mainSha: "c".repeat(40) },
    { isOnFirstParent: false },
    { tagSha: "main" },
    { workflowSha: "main" },
    { mainSha: "main" },
  ]) {
    assert.notEqual(invoke("release-commit", { ...commit, ...changed }).status, 0);
  }
  const ownership = {
    releaseVersion: "0.429.0",
    mainCargoToml: '[workspace.package]\nversion = "0.429.0"\n',
  };
  assert.doesNotThrow(() => validateReleaseControlVersion(ownership));
  const accepted = invoke("control-version", ownership);
  assert.equal(accepted.status, 0, accepted.stderr);
  for (const mainCargoToml of ['[workspace.package]\nversion = "0.430.0"\n', "[workspace]\n"]) {
    assert.throws(() => validateReleaseControlVersion({ ...ownership, mainCargoToml }));
    assert.notEqual(invoke("control-version", { ...ownership, mainCargoToml }).status, 0);
  }
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
