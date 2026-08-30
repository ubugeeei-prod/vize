import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { validateDownloadedArtifact } from "./npm-bootstrap-contract.mjs";

const strictVersionPattern =
  /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*))(?:\.(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*)))*)?$/;

function sha512(buffer) {
  return createHash("sha512").update(buffer).digest();
}

function requireSingleLine(label, value, pattern) {
  if (!pattern.test(value)) {
    throw new Error(`Invalid ${label}: ${value || "(empty)"}`);
  }
  return value;
}

export function cliHandoffNames(packageName, version) {
  const match = /^@([a-z0-9][a-z0-9._-]*)\/([a-z0-9][a-z0-9._-]*)$/.exec(packageName);
  if (match == null) {
    throw new Error(`CLI handoff requires a lowercase scoped npm package, got ${packageName}`);
  }
  requireSingleLine("package version", version, strictVersionPattern);
  const slug = `${match[1]}-${match[2]}`;
  return {
    artifactName: `npm-cli-first-publish-${slug}-${version}`,
    tarballName: `${slug}-${version}.tgz`,
  };
}

function runNpmPack({ npmBin, packagePath, destination }) {
  const result = spawnSync(
    npmBin,
    ["pack", "--ignore-scripts", "--json", "--pack-destination", destination],
    {
      cwd: packagePath,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
      timeout: 120_000,
    },
  );
  if (result.error?.code === "ETIMEDOUT") {
    throw new Error("npm pack timed out after 120000ms");
  }
  if (result.error != null) throw result.error;
  if (result.signal != null) throw new Error(`npm pack was terminated by ${result.signal}`);
  if (result.status !== 0) {
    const detail = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
    throw new Error(
      [`npm pack failed with exit ${result.status}`, detail].filter(Boolean).join("\n"),
    );
  }

  let entries;
  try {
    entries = JSON.parse(result.stdout);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    throw new Error(`npm pack did not return JSON metadata: ${detail}`, { cause: error });
  }
  if (!Array.isArray(entries) || entries.length !== 1) {
    throw new Error(`npm pack must describe exactly one tarball, got ${JSON.stringify(entries)}`);
  }
  return entries[0];
}

function packOnce({
  npmBin,
  packagePath,
  destination,
  expectedName,
  expectedVersion,
  tarballName,
}) {
  const metadata = runNpmPack({ npmBin, packagePath, destination });
  if (
    metadata?.name !== expectedName ||
    metadata?.version !== expectedVersion ||
    metadata?.filename !== tarballName
  ) {
    throw new Error(
      `npm pack produced ${String(metadata?.name)}@${String(metadata?.version)} in ${String(metadata?.filename)}, expected ${expectedName}@${expectedVersion} in ${tarballName}`,
    );
  }

  const tarballPath = path.join(destination, tarballName);
  const stat = fs.lstatSync(tarballPath);
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error(`npm pack output must be a regular file: ${tarballPath}`);
  }
  const contents = fs.readFileSync(tarballPath);
  const digest = sha512(contents);
  const integrity = `sha512-${digest.toString("base64")}`;
  if (metadata.integrity !== integrity) {
    throw new Error(`npm pack integrity mismatch for ${tarballName}`);
  }
  return { contents, integrity, sha512: digest.toString("hex") };
}

export function formatCliHandoffSummary(handoff) {
  return [
    "## npm CLI first-publish handoff",
    "",
    `- Package: \`${handoff.package.name}@${handoff.package.version}\``,
    `- Handoff artifact: \`${handoff.handoffArtifact}\``,
    `- Tarball: \`${handoff.tarball.file}\``,
    `- SHA-512: \`${handoff.tarball.sha512}\``,
    `- Source: \`${handoff.source.tagName}\` / Release run \`${handoff.source.releaseRunId}\` / \`${handoff.source.artifactName}\``,
    "",
    "Download the handoff artifact, verify `SHA512SUMS`, authenticate with the npm CLI as an owner with 2FA, then run:",
    "",
    "```bash",
    handoff.publish.command,
    "```",
    "",
    "This workflow did not request an OIDC token and did not publish. The one-time local CLI publish does not carry GitHub Actions OIDC provenance. Immediately configure `release.yml` as the trusted publisher with:",
    "",
    "```bash",
    handoff.publish.trustCommand,
    "```",
    "",
  ].join("\n");
}

export function createCliPublishHandoff({
  packagePath,
  outputPath,
  expectedName,
  expectedVersion,
  sourceArtifactName,
  releaseRunId,
  releaseTagName,
  releaseTagSha,
  npmBin = process.env.NPM_BIN || "npm",
}) {
  requireSingleLine("Release run ID", releaseRunId, /^[1-9]\d*$/);
  requireSingleLine("Release tag SHA", releaseTagSha, /^[0-9a-f]{40}$/);
  requireSingleLine("Release artifact name", sourceArtifactName, /^release-package-[a-z0-9-]+$/);
  if (releaseTagName !== `v${expectedVersion}`) {
    throw new Error(
      `Release tag ${releaseTagName || "(empty)"} must match package version ${expectedVersion}`,
    );
  }

  const packageManifest = fs.readFileSync(path.join(packagePath, "package.json"), "utf8");
  validateDownloadedArtifact({ packageManifest, expectedName, expectedVersion });
  const packageJson = JSON.parse(packageManifest);
  if (packageJson.publishConfig?.access !== "public") {
    throw new Error(`${expectedName} must declare publishConfig.access as public`);
  }

  const { artifactName, tarballName } = cliHandoffNames(expectedName, expectedVersion);
  fs.mkdirSync(outputPath);
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npm-cli-handoff-"));
  try {
    const firstDestination = path.join(tempRoot, "first");
    const secondDestination = path.join(tempRoot, "second");
    fs.mkdirSync(firstDestination);
    fs.mkdirSync(secondDestination);
    const first = packOnce({
      npmBin,
      packagePath,
      destination: firstDestination,
      expectedName,
      expectedVersion,
      tarballName,
    });
    const second = packOnce({
      npmBin,
      packagePath,
      destination: secondDestination,
      expectedName,
      expectedVersion,
      tarballName,
    });
    if (!first.contents.equals(second.contents) || first.integrity !== second.integrity) {
      throw new Error(`npm pack did not reproduce ${tarballName} byte-for-byte`);
    }

    fs.writeFileSync(path.join(outputPath, tarballName), first.contents, { flag: "wx" });
    fs.writeFileSync(path.join(outputPath, "SHA512SUMS"), `${first.sha512}  ${tarballName}\n`, {
      flag: "wx",
    });

    const handoff = {
      schemaVersion: 1,
      package: { name: expectedName, version: expectedVersion },
      source: {
        artifactName: sourceArtifactName,
        releaseRunId,
        tagName: releaseTagName,
        tagSha: releaseTagSha,
      },
      handoffArtifact: artifactName,
      tarball: { file: tarballName, integrity: first.integrity, sha512: first.sha512 },
      publish: {
        authentication: "interactive npm CLI owner session with 2FA",
        command: `npm publish ./${tarballName} --access public`,
        provenance: "none: local CLI first publish is outside GitHub Actions OIDC",
        trustCommand: `npm trust github ${expectedName} --file release.yml --repo ubugeeei-prod/vize --env npm --allow-publish --yes`,
      },
    };
    fs.writeFileSync(
      path.join(outputPath, "npm-publish-handoff.json"),
      `${JSON.stringify(handoff, null, 2)}\n`,
      { flag: "wx" },
    );
    return handoff;
  } finally {
    fs.rmSync(tempRoot, { force: true, recursive: true });
  }
}

function appendFileFromEnvironment(name, contents) {
  const filePath = process.env[name];
  if (!filePath) throw new Error(`${name} is required for npm CLI handoff`);
  fs.appendFileSync(filePath, contents);
}

function main(env = process.env) {
  if (env.BOOTSTRAP_ARTIFACT_PATH !== "bootstrap-package") {
    throw new Error("BOOTSTRAP_ARTIFACT_PATH must be bootstrap-package");
  }
  if (env.BOOTSTRAP_HANDOFF_PATH !== "npm-cli-first-publish") {
    throw new Error("BOOTSTRAP_HANDOFF_PATH must be npm-cli-first-publish");
  }
  const handoff = createCliPublishHandoff({
    packagePath: env.BOOTSTRAP_ARTIFACT_PATH,
    outputPath: env.BOOTSTRAP_HANDOFF_PATH,
    expectedName: env.EXPECTED_PACKAGE_NAME ?? "",
    expectedVersion: env.EXPECTED_PACKAGE_VERSION ?? "",
    sourceArtifactName: env.RELEASE_ARTIFACT_NAME ?? "",
    releaseRunId: env.RELEASE_RUN_ID ?? "",
    releaseTagName: env.RELEASE_TAG_NAME ?? "",
    releaseTagSha: env.RELEASE_TAG_SHA ?? "",
  });
  appendFileFromEnvironment(
    "GITHUB_OUTPUT",
    `artifact_name=${handoff.handoffArtifact}\ntarball_name=${handoff.tarball.file}\nsha512=${handoff.tarball.sha512}\n`,
  );
  appendFileFromEnvironment("GITHUB_STEP_SUMMARY", formatCliHandoffSummary(handoff));
  console.log(
    `Prepared deterministic npm CLI handoff ${handoff.handoffArtifact}/${handoff.tarball.file}.`,
  );
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  try {
    main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
