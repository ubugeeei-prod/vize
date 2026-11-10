import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { parse } from "yaml";

import {
  cliHandoffNames,
  createCliPublishHandoff,
  type HandoffInput,
} from "../../../tools/support/release/npm_bootstrap/handoff.ts";
import {
  bootstrapPackages,
  validateArtifactInventory,
  validateCandidate,
  validatePackageManifest,
  validateRegistryStatus,
  validateReleaseJobs,
  validateRequest,
  type BootstrapRequest,
} from "../../../tools/support/release/npm_bootstrap/policy.ts";
import { readRepoFile } from "../support/github-workflows.ts";

interface Step {
  env?: Record<string, string>;
  id?: string;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
}

interface Workflow {
  env?: Record<string, string>;
  jobs: Record<
    string,
    {
      environment?: string;
      permissions?: Record<string, string>;
      steps: Step[];
    }
  >;
  on: {
    workflow_dispatch: {
      inputs: Record<string, { options?: string[]; required: boolean; type: string }>;
    };
  };
  permissions: Record<string, string>;
}

const workflowSource = readRepoFile(".github", "workflows", "release-npm-bootstrap.yml");
const workflow = parse(workflowSource) as Workflow;
const sha = (character: string) => character.repeat(40);

function request(): BootstrapRequest {
  return validateRequest({
    packagePath: "npm/framework/nuxt-lint-config",
    releaseRunId: "42",
    tagName: "v1.0.0-alpha.1",
    version: "1.0.0-alpha.1",
    workflowRef: "refs/heads/main",
  });
}

function releaseRun() {
  return {
    conclusion: "failure",
    event: "workflow_dispatch",
    head_branch: "main",
    head_repository: { full_name: "ubugeeei-prod/vize" },
    head_sha: sha("a"),
    id: 42,
    name: "Release",
    path: ".github/workflows/release.yml",
    status: "completed",
  };
}

function releaseJobs() {
  return [
    ["Prepare the release candidate", "success"],
    ["Build npm packages", "success"],
    ["Smoke package tarballs", "success"],
    ["Tag the release", "success"],
    ["Admit the immutable release publication", "success"],
    ["Publish npm packages", "failure"],
  ].map(([name, conclusion]) => ({ name, conclusion, status: "completed" }));
}

test("bootstrap workflow is a read-only manual handoff without publishing authority", () => {
  assert.deepEqual(Object.keys(workflow.on), ["workflow_dispatch"]);
  assert.deepEqual(workflow.permissions, { actions: "read", contents: "read" });
  assert.deepEqual(Object.keys(workflow.on.workflow_dispatch.inputs), [
    "version",
    "tag_name",
    "release_run_id",
    "package_path",
  ]);
  assert.deepEqual(workflow.on.workflow_dispatch.inputs.package_path?.options, [
    ...bootstrapPackages.keys(),
  ]);
  assert.ok(Object.values(workflow.on.workflow_dispatch.inputs).every((input) => input.required));
  const job = workflow.jobs.handoff;
  assert.ok(job);
  assert.equal(job.environment, undefined);
  assert.equal(job.permissions, undefined, "job inherits the exact read-only workflow permissions");
  assert.equal(workflow.env?.GITHUB_TOKEN, undefined);
  const preflight = job.steps.find((step) => step.id === "policy");
  assert.equal(preflight?.env?.GITHUB_TOKEN, "${{ github.token }}");
  assert.equal(
    job.steps
      .slice(job.steps.indexOf(preflight!) + 1)
      .some((step) => step.env?.GITHUB_TOKEN !== undefined),
    false,
  );
  assert.doesNotMatch(
    workflowSource,
    /id-token|secrets\.|NPM_TOKEN|NODE_AUTH_TOKEN|_authToken|--provenance|npm publish|publish_npm_package_dirs/,
  );
});

test("bootstrap workflow downloads one verified artifact ID and only uploads the final handoff", () => {
  const steps = workflow.jobs.handoff!.steps;
  const preflight = steps.find((step) => step.id === "policy");
  const download = steps.find((step) => step.name === "Download the exact npm-packages artifact");
  const artifact = steps.find((step) => step.id === "artifact");
  const prepare = steps.find(
    (step) => step.name === "Resolve publish-time workspace and catalog protocols",
  );
  const prepared = steps.find((step) => step.name === "Validate the standalone publish manifest");
  const registry = steps.find(
    (step) => step.name === "Confirm the package is still absent from npm",
  );
  const handoff = steps.find((step) => step.id === "handoff");
  const upload = steps.find((step) => step.name === "Upload the exact npm CLI handoff");
  for (const step of [
    preflight,
    download,
    artifact,
    prepare,
    prepared,
    registry,
    handoff,
    upload,
  ]) {
    assert.ok(step);
  }
  assert.equal(download!.with?.["artifact-ids"], "${{ steps.policy.outputs.artifact_id }}");
  assert.equal(download!.with?.["run-id"], "${{ inputs.release_run_id }}");
  assert.equal(download!.with?.["github-token"], "${{ github.token }}");
  assert.equal(download!.with?.repository, "${{ github.repository }}");
  assert.equal(download!.with?.path, "${{ runner.temp }}/npm-bootstrap-stage/npm");
  assert.equal(upload!.with?.path, "${{ runner.temp }}/npm-cli-first-publish");
  assert.equal(upload!.with?.["compression-level"], 0);
  assert.equal(upload!.with?.["retention-days"], 7);
  const indices = [preflight, download, artifact, prepare, prepared, registry, handoff, upload].map(
    (step) => steps.indexOf(step!),
  );
  assert.deepEqual(
    indices,
    [...indices].sort((left, right) => left - right),
  );
  assert.equal(steps.at(-1), upload);
});

test("supply-chain guide hands the exact tarball to a local owner before npm trust", () => {
  const docs = readRepoFile("docs", "release", "supply-chain.md");
  const section = docs.slice(docs.indexOf("### First-publish bootstrap"));
  for (const required of [
    "gh workflow run release-npm-bootstrap.yml --ref main",
    "npm-publish-handoff.json",
    "shasum -a 512 --check SHA512SUMS",
    "npm publish ./<scope>-<package>-X.Y.Z.tgz --access public",
    "npm trust github",
    "--file release.yml",
    "--env npm",
    "--allow-publish",
  ]) {
    assert.ok(section.includes(required), required);
  }
  assert.match(section, /only `actions: read` and `contents: read`/);
  assert.match(section, /does not have GitHub Actions OIDC provenance/);
  assert.doesNotMatch(section, /repository_dispatch|event_type=npm-bootstrap|NPM_TOKEN/);
});

test("request policy requires strict version, matching tag, main and the explicit allowlist", () => {
  assert.equal(request().packageName, "@vizejs/nuxt-lint-config");
  const base = {
    packagePath: "npm/plugin-sdk",
    releaseRunId: "42",
    tagName: "v1.2.3",
    version: "1.2.3",
    workflowRef: "refs/heads/main",
  };
  assert.equal(validateRequest(base).packageName, "@vizejs/plugin-sdk");
  for (const mutation of [
    { version: "01.2.3", tagName: "v01.2.3" },
    { version: "1.2", tagName: "v1.2" },
    { version: "1.2.3", tagName: "v1.2.4" },
    { packagePath: "npm/unknown" },
    { releaseRunId: "0" },
    { workflowRef: "refs/heads/feature" },
  ]) {
    assert.throws(() => validateRequest({ ...base, ...mutation }));
  }
});

test("candidate tag is the exact single-parent child of the failed main Release run", () => {
  const candidate = { sha: sha("b"), parents: [sha("a")] };
  assert.deepEqual(validateCandidate(request(), candidate, releaseRun()), {
    candidateSha: sha("b"),
    parentSha: sha("a"),
  });
  assert.throws(() => validateCandidate(request(), { ...candidate, parents: [] }, releaseRun()));
  assert.throws(() =>
    validateCandidate(request(), { ...candidate, parents: [sha("a"), sha("c")] }, releaseRun()),
  );
  for (const [key, value] of [
    ["head_sha", sha("c")],
    ["head_branch", "release/v1.0.0-alpha.1"],
    ["path", ".github/workflows/other.yml"],
    ["conclusion", "success"],
    ["status", "in_progress"],
  ] as const) {
    assert.throws(() => validateCandidate(request(), candidate, { ...releaseRun(), [key]: value }));
  }
});

test("release evidence requires the successful candidate gates and failed npm publisher", () => {
  validateReleaseJobs(releaseJobs());
  const missing = releaseJobs().slice(1);
  assert.throws(() => validateReleaseJobs(missing));
  const wrong = releaseJobs();
  wrong.at(-1)!.conclusion = "success";
  assert.throws(() => validateReleaseJobs(wrong));
  assert.throws(() => validateReleaseJobs([...releaseJobs(), releaseJobs()[0]!]));
});

test("artifact policy binds one live npm-packages ID and digest to the exact run", () => {
  const artifact = {
    digest: `sha256:${"d".repeat(64)}`,
    expired: false,
    id: 7,
    name: "npm-packages",
    workflow_run: { id: 42, head_branch: "main", head_sha: sha("a") },
  };
  assert.deepEqual(validateArtifactInventory([artifact], request(), sha("a")), {
    digest: artifact.digest,
    id: "7",
  });
  assert.throws(() => validateArtifactInventory([], request(), sha("a")));
  assert.throws(() => validateArtifactInventory([artifact, artifact], request(), sha("a")));
  assert.throws(() =>
    validateArtifactInventory([{ ...artifact, expired: true }], request(), sha("a")),
  );
  assert.throws(() =>
    validateArtifactInventory(
      [{ ...artifact, workflow_run: { ...artifact.workflow_run, head_sha: sha("c") } }],
      request(),
      sha("a"),
    ),
  );
});

test("package policy requires exact path-owned name, version, public access and registry 404", () => {
  const manifest = JSON.stringify({
    name: "@vizejs/nuxt-lint-config",
    version: "1.0.0-alpha.1",
    publishConfig: { access: "public" },
  });
  const cargo = `[workspace.package]\nversion = "1.0.0-alpha.1"\nedition = "2024"\n`;
  validatePackageManifest(request(), manifest, cargo);
  assert.throws(() =>
    validatePackageManifest(request(), manifest.replace("nuxt-lint-config", "wrong"), cargo),
  );
  assert.throws(() =>
    validatePackageManifest(request(), manifest.replace("1.0.0-alpha.1", "1.0.0"), cargo),
  );
  assert.throws(() =>
    validatePackageManifest(request(), manifest.replace("public", "restricted"), cargo),
  );
  assert.throws(() =>
    validatePackageManifest(
      request(),
      JSON.stringify({ ...JSON.parse(manifest), dependencies: { vize: "workspace:*" } }),
      undefined,
      true,
    ),
  );
  validateRegistryStatus(request().packageName, 404);
  assert.throws(() => validateRegistryStatus(request().packageName, 200));
  assert.throws(() => validateRegistryStatus(request().packageName, 503));
});

function handoffFixture(t: TestContext) {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npm-handoff-test-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const packageRoot = path.join(temporary, "package");
  fs.mkdirSync(packageRoot);
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    `${JSON.stringify({
      name: "@vizejs/nuxt-lint-config",
      version: "1.0.0-alpha.1",
      publishConfig: { access: "public" },
    })}\n`,
  );
  const input: HandoffInput = {
    artifactDigest: `sha256:${"d".repeat(64)}`,
    artifactId: "7",
    candidateSha: sha("b"),
    originalManifestSha512: "e".repeat(128),
    outputRoot: path.join(temporary, "handoff"),
    packageName: "@vizejs/nuxt-lint-config",
    packagePath: "npm/framework/nuxt-lint-config",
    packageRoot,
    parentSha: sha("a"),
    releaseRunId: "42",
    tagName: "v1.0.0-alpha.1",
    version: "1.0.0-alpha.1",
  };
  return { input, temporary };
}

test("handoff double-packs exact bytes and records SHA-512 for local first publish", (t) => {
  const { input } = handoffFixture(t);
  const contents = Buffer.from("deterministic tarball");
  const digest = createHash("sha512").update(contents).digest();
  let calls = 0;
  const handoff = createCliPublishHandoff(input, () => {
    calls += 1;
    return {
      contents,
      integrity: `sha512-${digest.toString("base64")}`,
      sha512: digest.toString("hex"),
    };
  });
  assert.equal(calls, 2);
  const names = cliHandoffNames(input.packageName, input.version);
  assert.equal(handoff.handoffArtifact, names.artifactName);
  assert.deepEqual(fs.readdirSync(input.outputRoot).sort(), [
    "SHA512SUMS",
    "npm-publish-handoff.json",
    names.tarballName,
  ]);
  assert.equal(
    fs.readFileSync(path.join(input.outputRoot, "SHA512SUMS"), "utf8"),
    `${digest.toString("hex")}  ${names.tarballName}\n`,
  );
  const receipt = JSON.parse(
    fs.readFileSync(path.join(input.outputRoot, "npm-publish-handoff.json"), "utf8"),
  );
  assert.equal(receipt.source.artifactId, "7");
  assert.equal(receipt.source.artifactDigest, input.artifactDigest);
  assert.equal(receipt.publish.command, `npm publish ./${names.tarballName} --access public`);
  assert.match(receipt.publish.trustCommand, /^npm trust github /);
  assert.match(receipt.publish.provenance, /^none:/);
});

test("real npm pack is deterministic and its reported SHA-512 is independently reproduced", (t) => {
  const { input } = handoffFixture(t);
  fs.writeFileSync(path.join(input.packageRoot, "index.js"), "export const ready = true;\n");
  const handoff = createCliPublishHandoff(input) as {
    tarball: { file: string; integrity: string; sha512: string };
  };
  const bytes = fs.readFileSync(path.join(input.outputRoot, handoff.tarball.file));
  const digest = createHash("sha512").update(bytes).digest();
  assert.equal(handoff.tarball.sha512, digest.toString("hex"));
  assert.equal(handoff.tarball.integrity, `sha512-${digest.toString("base64")}`);
});

test("handoff refuses nondeterministic double-pack output", (t) => {
  const { input } = handoffFixture(t);
  let calls = 0;
  assert.throws(() =>
    createCliPublishHandoff(input, () => {
      const contents = Buffer.from(`tarball-${calls++}`);
      const digest = createHash("sha512").update(contents).digest();
      return {
        contents,
        integrity: `sha512-${digest.toString("base64")}`,
        sha512: digest.toString("hex"),
      };
    }),
  );
  assert.equal(fs.existsSync(input.outputRoot), false);
});
