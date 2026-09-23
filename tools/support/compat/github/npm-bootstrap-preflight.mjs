import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  assertPackageIsUnpublished,
  validateBootstrapManifest,
  validateBootstrapRequest,
  validateDownloadedArtifact,
  validateReleaseCommit,
  verifyReleaseRunEvidence,
} from "./npm-bootstrap-contract.mjs";

function runGit(args, acceptedExitCodes = [0]) {
  const result = spawnSync("git", args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 30_000,
  });
  if (result.error?.code === "ETIMEDOUT") {
    throw new Error(`git ${args.join(" ")} timed out after 30000ms`);
  }
  if (result.error != null) throw result.error;
  if (result.signal != null) {
    throw new Error(`git ${args.join(" ")} was terminated by ${result.signal}`);
  }
  if (!acceptedExitCodes.includes(result.status)) {
    const detail = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
    throw new Error(
      [`git ${args.join(" ")} failed with exit ${result.status}`, detail]
        .filter(Boolean)
        .join("\n"),
    );
  }
  return result;
}

function readTaggedFile(tagSha, filePath) {
  return runGit(["show", `${tagSha}:${filePath}`]).stdout;
}

/** @param {Record<string, string>} outputs */
function writeOutputs(outputs) {
  const outputPath = process.env.GITHUB_OUTPUT;
  if (!outputPath) throw new Error("GITHUB_OUTPUT is required for npm bootstrap preflight");
  fs.appendFileSync(
    outputPath,
    Object.entries(outputs)
      .map(([name, value]) => `${name}=${value}\n`)
      .join(""),
  );
}

async function runPreflight(env = process.env) {
  const request = validateBootstrapRequest({
    tagName: env.RELEASE_TAG_NAME ?? "",
    packagePath: env.BOOTSTRAP_PACKAGE_PATH ?? "",
    releaseRunId: env.RELEASE_RUN_ID ?? "",
    workflowRef: env.GITHUB_REF ?? "",
    workflowSha: env.GITHUB_SHA ?? "",
  });

  runGit(["fetch", "--quiet", "--no-tags", "origin", "+refs/heads/main:refs/remotes/origin/main"]);
  runGit([
    "fetch",
    "--quiet",
    "--no-tags",
    "origin",
    `+refs/tags/${request.tagName}:refs/tags/${request.tagName}`,
  ]);
  const tagSha = runGit(["rev-parse", `refs/tags/${request.tagName}^{commit}`]).stdout.trim();
  const mainSha = runGit(["rev-parse", "refs/remotes/origin/main"]).stdout.trim();
  const mainFirstParentHistory = runGit(["rev-list", "--first-parent", "refs/remotes/origin/main"])
    .stdout.trim()
    .split(/\r?\n/);
  validateReleaseCommit({
    tagSha,
    workflowSha: request.workflowSha,
    mainSha,
    isOnFirstParent: mainFirstParentHistory.includes(tagSha),
  });

  const packageManifest = readTaggedFile(tagSha, `${request.packagePath}/package.json`);
  const version = validateBootstrapManifest({
    ...request,
    tagSha,
    cargoToml: readTaggedFile(tagSha, "Cargo.toml"),
    packageManifest,
  });
  await verifyReleaseRunEvidence({
    apiUrl: env.GITHUB_API_URL ?? "https://api.github.com",
    repository: env.GITHUB_REPOSITORY ?? "",
    token: env.GITHUB_TOKEN ?? "",
    releaseRunId: request.releaseRunId,
    tagName: request.tagName,
    tagSha,
    artifactName: request.artifactName,
  });
  await assertPackageIsUnpublished(request.packageName);
  writeOutputs({
    artifact_name: request.artifactName,
    package_name: request.packageName,
    package_path: request.packagePath,
    release_run_id: request.releaseRunId,
    tag_sha: tagSha,
    version,
  });
  console.log(
    `npm bootstrap preflight passed for ${request.packageName}@${version} from ${request.tagName} (${tagSha}).`,
  );
}

async function runRegistryRecheck(env = process.env) {
  const request = validateBootstrapRequest({
    tagName: env.RELEASE_TAG_NAME ?? "",
    packagePath: env.BOOTSTRAP_PACKAGE_PATH ?? "",
    releaseRunId: env.RELEASE_RUN_ID ?? "",
    workflowRef: env.GITHUB_REF ?? "",
    workflowSha: env.GITHUB_SHA ?? "",
  });
  await assertPackageIsUnpublished(request.packageName);
  console.log(`${request.packageName} is still absent from npm immediately before first publish.`);
}

function runArtifactValidation(env = process.env) {
  const artifactPath = env.BOOTSTRAP_ARTIFACT_PATH ?? "";
  if (artifactPath !== "bootstrap-package") {
    throw new Error(
      `Bootstrap artifact path must be bootstrap-package, got ${artifactPath || "(empty)"}`,
    );
  }
  const expectedName = env.EXPECTED_PACKAGE_NAME ?? "";
  const expectedVersion = env.EXPECTED_PACKAGE_VERSION ?? "";
  validateDownloadedArtifact({
    packageManifest: fs.readFileSync(path.join(artifactPath, "package.json"), "utf8"),
    expectedName,
    expectedVersion,
  });
  console.log(`Validated downloaded Release artifact ${expectedName}@${expectedVersion}.`);
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  const [command = "preflight", extra] = process.argv.slice(2);
  Promise.resolve()
    .then(() => {
      if (extra != null || !["artifact", "preflight", "registry-recheck"].includes(command)) {
        throw new Error(
          "Usage: rust-script tools/commands/ci/github/npm-bootstrap-preflight.rs [artifact|preflight|registry-recheck]",
        );
      }
      if (command === "preflight") return runPreflight();
      return command === "artifact" ? runArtifactValidation() : runRegistryRecheck();
    })
    .catch((error) => {
      console.error(error instanceof Error ? error.message : String(error));
      process.exit(1);
    });
}
