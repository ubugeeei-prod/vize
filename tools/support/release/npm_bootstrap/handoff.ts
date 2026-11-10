import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const strictVersion =
  /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*))(?:\.(?:(?:0|[1-9]\d*)|(?:\d*[A-Za-z-][0-9A-Za-z-]*)))*)?$/;

function requireValue(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}

function required(value: string, pattern: RegExp, message: string): string {
  requireValue(pattern.test(value), message);
  return value;
}

export function cliHandoffNames(packageName: string, version: string) {
  const match = /^@([a-z0-9][a-z0-9._-]*)\/([a-z0-9][a-z0-9._-]*)$/.exec(packageName);
  requireValue(match !== null, "lowercase scoped npm package name required");
  required(version, strictVersion, "strict package version required");
  const slug = `${match[1]}-${match[2]}`;
  return {
    artifactName: `npm-cli-first-publish-${slug}-${version}`,
    tarballName: `${slug}-${version}.tgz`,
  };
}

interface PackResult {
  contents: Buffer;
  integrity: string;
  sha512: string;
}

type Pack = (destination: string) => PackResult;

interface PackMetadata {
  filename?: unknown;
  integrity?: unknown;
  name?: unknown;
  version?: unknown;
}

function npmPack(
  packageRoot: string,
  expectedName: string,
  expectedVersion: string,
  tarballName: string,
  destination: string,
): PackResult {
  const npm = process.env.NPM_BIN ?? "npm";
  const result = spawnSync(
    npm,
    ["pack", "--ignore-scripts", "--json", "--pack-destination", destination],
    {
      cwd: packageRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
      timeout: 120_000,
    },
  );
  if (result.error !== undefined) throw result.error;
  requireValue(
    result.status === 0,
    `npm pack failed with exit ${String(result.status)}: ${result.stderr.trim()}`,
  );
  let rows: unknown;
  try {
    rows = JSON.parse(result.stdout) as unknown;
  } catch (error) {
    throw new Error("npm pack returned invalid JSON metadata", { cause: error });
  }
  requireValue(Array.isArray(rows) && rows.length === 1, "npm pack must produce one tarball");
  const metadata = rows[0] as PackMetadata;
  requireValue(
    metadata.name === expectedName &&
      metadata.version === expectedVersion &&
      metadata.filename === tarballName,
    "npm pack package identity differs",
  );
  const tarballPath = path.join(destination, tarballName);
  const stat = fs.lstatSync(tarballPath);
  requireValue(stat.isFile() && !stat.isSymbolicLink(), "npm pack output must be a regular file");
  const contents = fs.readFileSync(tarballPath);
  const digest = createHash("sha512").update(contents).digest();
  const integrity = `sha512-${digest.toString("base64")}`;
  requireValue(metadata.integrity === integrity, "npm pack SHA-512 integrity differs");
  return { contents, integrity, sha512: digest.toString("hex") };
}

export interface HandoffInput {
  artifactDigest: string;
  artifactId: string;
  candidateSha: string;
  originalManifestSha512: string;
  outputRoot: string;
  packageName: string;
  packagePath: string;
  packageRoot: string;
  parentSha: string;
  releaseRunId: string;
  tagName: string;
  version: string;
}

function validateInput(input: HandoffInput): void {
  required(input.artifactId, /^[1-9]\d*$/, "artifact ID required");
  required(input.releaseRunId, /^[1-9]\d*$/, "Release run ID required");
  required(input.artifactDigest, /^sha256:[0-9a-f]{64}$/, "artifact digest required");
  required(input.candidateSha, /^[0-9a-f]{40}$/, "candidate SHA required");
  required(input.parentSha, /^[0-9a-f]{40}$/, "candidate parent SHA required");
  required(input.originalManifestSha512, /^[0-9a-f]{128}$/, "original manifest SHA-512 required");
  requireValue(input.tagName === `v${input.version}`, "tag and package version differ");
  requireValue(input.packagePath.startsWith("npm/"), "approved npm package path required");
  requireValue(path.isAbsolute(input.packageRoot), "absolute package artifact path required");
  requireValue(path.isAbsolute(input.outputRoot), "absolute handoff output path required");
}

export function createCliPublishHandoff(input: HandoffInput, pack?: Pack): Record<string, unknown> {
  validateInput(input);
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(input.packageRoot, "package.json"), "utf8"),
  ) as Record<string, unknown>;
  requireValue(
    packageJson.name === input.packageName && packageJson.version === input.version,
    "prepared artifact package identity differs",
  );
  requireValue(
    !/"(?:workspace|catalog):/.test(JSON.stringify(packageJson)),
    "prepared artifact contains unresolved dependency protocols",
  );
  const { artifactName, tarballName } = cliHandoffNames(input.packageName, input.version);
  const runPack =
    pack ??
    ((destination: string) =>
      npmPack(input.packageRoot, input.packageName, input.version, tarballName, destination));
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npm-bootstrap-"));
  try {
    const firstPath = path.join(temporary, "first");
    const secondPath = path.join(temporary, "second");
    fs.mkdirSync(firstPath);
    fs.mkdirSync(secondPath);
    const first = runPack(firstPath);
    const second = runPack(secondPath);
    requireValue(
      first.contents.equals(second.contents) &&
        first.integrity === second.integrity &&
        first.sha512 === second.sha512,
      "npm pack output is not byte-for-byte deterministic",
    );

    fs.mkdirSync(input.outputRoot);
    fs.writeFileSync(path.join(input.outputRoot, tarballName), first.contents, { flag: "wx" });
    fs.writeFileSync(
      path.join(input.outputRoot, "SHA512SUMS"),
      `${first.sha512}  ${tarballName}\n`,
      { flag: "wx" },
    );
    const handoff = {
      schemaVersion: 1,
      package: {
        name: input.packageName,
        path: input.packagePath,
        version: input.version,
      },
      source: {
        artifactDigest: input.artifactDigest,
        artifactId: input.artifactId,
        artifactName: "npm-packages",
        candidateSha: input.candidateSha,
        originalManifestSha512: input.originalManifestSha512,
        parentSha: input.parentSha,
        releaseRunId: input.releaseRunId,
        tagName: input.tagName,
      },
      handoffArtifact: artifactName,
      tarball: { file: tarballName, integrity: first.integrity, sha512: first.sha512 },
      publish: {
        authentication: "interactive npm owner session with settings 2FA",
        command: `npm publish ./${tarballName} --access public`,
        provenance: "none: one-time local first publish is outside GitHub Actions OIDC",
        trustCommand: `npm trust github ${input.packageName} --file release.yml --repo ubugeeei-prod/vize --env npm --allow-publish --yes`,
      },
    };
    fs.writeFileSync(
      path.join(input.outputRoot, "npm-publish-handoff.json"),
      `${JSON.stringify(handoff, null, 2)}\n`,
      { flag: "wx" },
    );
    return handoff;
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
}

function environmentInput(env: NodeJS.ProcessEnv): HandoffInput {
  return {
    artifactDigest: env.SOURCE_ARTIFACT_DIGEST ?? "",
    artifactId: env.SOURCE_ARTIFACT_ID ?? "",
    candidateSha: env.RELEASE_CANDIDATE_SHA ?? "",
    originalManifestSha512: env.ORIGINAL_MANIFEST_SHA512 ?? "",
    outputRoot: env.BOOTSTRAP_OUTPUT_ROOT ?? "",
    packageName: env.EXPECTED_PACKAGE_NAME ?? "",
    packagePath: env.BOOTSTRAP_PACKAGE_PATH ?? "",
    packageRoot: env.BOOTSTRAP_PACKAGE_ROOT ?? "",
    parentSha: env.RELEASE_PARENT_SHA ?? "",
    releaseRunId: env.RELEASE_RUN_ID ?? "",
    tagName: env.RELEASE_TAG_NAME ?? "",
    version: env.RELEASE_VERSION ?? "",
  };
}

function appendEnvironmentFile(name: string, contents: string): void {
  const target = process.env[name];
  requireValue(target !== undefined && target !== "", `${name} required`);
  fs.appendFileSync(target, contents);
}

function summary(handoff: Record<string, unknown>): string {
  const packageInfo = handoff.package as { name: string; version: string };
  const source = handoff.source as { artifactId: string; releaseRunId: string; tagName: string };
  const tarball = handoff.tarball as { file: string; sha512: string };
  const publish = handoff.publish as { command: string; trustCommand: string };
  return `## npm CLI first-publish handoff

- Package: \`${packageInfo.name}@${packageInfo.version}\`
- Source: \`${source.tagName}\`, Release run \`${source.releaseRunId}\`, artifact \`${source.artifactId}\`
- Tarball: \`${tarball.file}\`
- SHA-512: \`${tarball.sha512}\`

Verify \`SHA512SUMS\`, authenticate locally as an npm owner with settings 2FA, and run:

\`\`\`bash
${publish.command}
\`\`\`

Then configure the normal trusted publisher:

\`\`\`bash
${publish.trustCommand}
\`\`\`
`;
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  try {
    requireValue(process.argv.length === 2, "handoff.ts takes no arguments");
    const handoff = createCliPublishHandoff(environmentInput(process.env));
    appendEnvironmentFile("GITHUB_OUTPUT", `artifact_name=${String(handoff.handoffArtifact)}\n`);
    appendEnvironmentFile("GITHUB_STEP_SUMMARY", summary(handoff));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
