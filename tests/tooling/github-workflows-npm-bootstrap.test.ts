import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

type WorkflowStep = {
  env?: Record<string, string>;
  id?: string;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
  "working-directory"?: string;
};

type BootstrapWorkflow = {
  jobs?: Record<
    string,
    {
      environment?: string;
      permissions?: Record<string, string>;
      "runs-on"?: string;
      steps?: WorkflowStep[];
    }
  >;
  on?: {
    repository_dispatch?: { types?: string[] };
  };
  permissions?: Record<string, string>;
};

test("npm bootstrap workflow uses one default-branch repository dispatch event", () => {
  const source = readRepoFile(".github", "workflows", "release-npm-bootstrap.yml");
  const workflow = parse(source) as BootstrapWorkflow;
  assert.deepEqual(Object.keys(workflow.on ?? {}), ["repository_dispatch"]);
  assert.deepEqual(workflow.on?.repository_dispatch?.types, ["npm-bootstrap"]);
  assert.doesNotMatch(source, /workflow_dispatch|\$\{\{\s*inputs\./);
  assert.deepEqual(workflow.permissions, { contents: "read" });

  const controlsCheckout = workflow.jobs?.handoff?.steps?.find(
    (step) => step.name === "Checkout bootstrap controls from main",
  );
  assert.ok(controlsCheckout);
  assert.equal(controlsCheckout.with?.ref, undefined);
  assert.equal(controlsCheckout.with?.path, undefined);
  assert.equal(controlsCheckout.with?.["fetch-depth"], 0);
  assert.equal(controlsCheckout.with?.["persist-credentials"], false);

  const release = readRepoFile(".github", "workflows", "release.yml");
  assert.doesNotMatch(release, /NPM_TOKEN|NODE_AUTH_TOKEN|_authToken/);
});

test("npm bootstrap validates an exact tag before building with existing release helpers", () => {
  const source = readRepoFile(".github", "workflows", "release-npm-bootstrap.yml");
  const workflow = parse(source) as BootstrapWorkflow;
  const job = workflow.jobs?.handoff;
  assert.ok(job);
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job.environment, undefined);
  assert.deepEqual(job.permissions, {
    actions: "read",
    contents: "read",
  });

  const steps = job.steps ?? [];
  const preflight = steps.find((step) => step.id === "preflight");
  assert.equal(preflight?.run, "node tools/github/npm-bootstrap-preflight.mjs preflight");
  assert.equal(
    preflight?.env?.BOOTSTRAP_PACKAGE_PATH,
    "${{ github.event.client_payload.package_path }}",
  );
  assert.equal(preflight?.env?.RELEASE_TAG_NAME, "${{ github.event.client_payload.tag_name }}");
  assert.equal(preflight?.env?.RELEASE_RUN_ID, "${{ github.event.client_payload.release_run_id }}");
  assert.equal(preflight?.env?.GITHUB_TOKEN, "${{ github.token }}");

  const install = steps.find((step) => step.name === "Install package build dependencies");
  const testPackage = steps.find(
    (step) => step.name === "Test and build package at the dispatch SHA",
  );
  assert.match(install?.run ?? "", /tools\/moon\/cmd\/github\/vp_install/);
  assert.match(testPackage?.run ?? "", /vp run --filter .* test/);

  const download = steps.find(
    (step) => step.name === "Download the smoke-tested Release package artifact",
  );
  assert.match(download?.uses ?? "", /^actions\/download-artifact@[0-9a-f]{40}$/);
  assert.equal(download?.with?.name, "${{ steps.preflight.outputs.artifact_name }}");
  assert.equal(download?.with?.path, "bootstrap-package");
  assert.equal(download?.with?.["run-id"], "${{ steps.preflight.outputs.release_run_id }}");
  assert.equal(download?.with?.["github-token"], "${{ github.token }}");

  const smoke = steps.find(
    (step) => step.name === "Validate and smoke the exact Release package artifact",
  );
  assert.match(smoke?.run ?? "", /npm-bootstrap-preflight\.mjs artifact/);
  assert.match(smoke?.run ?? "", /smoke-release-install\.mjs --prepare-manifests/);

  const recheck = steps.find((step) => step.name === "Confirm package is still unpublished");
  assert.equal(recheck?.run, "node tools/github/npm-bootstrap-preflight.mjs registry-recheck");

  const requiredOrder = [preflight, install, testPackage, download, smoke, recheck];
  assert.ok(requiredOrder.every((step) => step != null));
  const requiredIndices = requiredOrder.map((step) => steps.indexOf(step!));
  assert.deepEqual(
    requiredIndices,
    [...requiredIndices].sort((left, right) => left - right),
    "every release-artifact validation must run before the CLI handoff is packed",
  );
});

test("npm bootstrap creates a credential-free deterministic CLI handoff", () => {
  const source = readRepoFile(".github", "workflows", "release-npm-bootstrap.yml");
  const workflow = parse(source) as BootstrapWorkflow;
  const steps = workflow.jobs?.handoff?.steps ?? [];
  const pack = steps.find(
    (step) => step.name === "Pack deterministic npm CLI first-publish handoff",
  );
  assert.ok(pack);
  assert.equal(pack.id, "handoff");
  assert.equal(pack.run, "node tools/github/npm-bootstrap-handoff.mjs");
  assert.equal(pack.env?.BOOTSTRAP_ARTIFACT_PATH, "bootstrap-package");
  assert.equal(pack.env?.BOOTSTRAP_HANDOFF_PATH, "npm-cli-first-publish");
  assert.equal(pack.env?.EXPECTED_PACKAGE_NAME, "${{ steps.preflight.outputs.package_name }}");
  assert.equal(pack.env?.EXPECTED_PACKAGE_VERSION, "${{ steps.preflight.outputs.version }}");
  assert.equal(pack.env?.RELEASE_TAG_SHA, "${{ steps.preflight.outputs.tag_sha }}");

  const upload = steps.find((step) => step.name === "Upload npm CLI first-publish handoff");
  assert.ok(upload);
  assert.match(upload.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.equal(upload.with?.name, "${{ steps.handoff.outputs.artifact_name }}");
  assert.equal(upload.with?.path, "npm-cli-first-publish");
  assert.equal(upload.with?.["if-no-files-found"], "error");
  assert.equal(upload.with?.["compression-level"], 0);
  assert.equal(upload.with?.["retention-days"], 7);
  assert.equal(steps.at(-1), upload);
  const registryRecheck = steps.find(
    (step) => step.name === "Confirm package is still unpublished",
  );
  assert.ok(registryRecheck);
  assert.ok(steps.indexOf(registryRecheck) < steps.indexOf(pack));
  assert.ok(steps.indexOf(pack) < steps.indexOf(upload));

  assert.doesNotMatch(
    source,
    /NPM_TOKEN|NODE_AUTH_TOKEN|_authToken|secrets\.|id-token|--provenance|npm publish|publish_npm_package/,
  );
});

test("npm bootstrap handoff documents the exact trusted publisher command", () => {
  const docs = readRepoFile("docs", "release", "supply-chain.md");
  const dispatchBlock = docs
    .split(/```bash\n|\n```/)
    .find((block) => block.includes("gh api repos/ubugeeei-prod/vize/dispatches"));
  assert.ok(dispatchBlock, "docs must document the npm-bootstrap dispatch command");
  for (const token of [
    "FRESH_TAG=vX.Y.Z",
    "-f event_type=npm-bootstrap",
    "client_payload[tag_name]=$FRESH_TAG",
    "client_payload[release_run_id]=$RELEASE_RUN_ID",
    "client_payload[package_path]=npm/framework/nuxt-lint-config",
  ]) {
    assert.ok(dispatchBlock.includes(token), `dispatch command must contain ${token}`);
  }
  assert.match(dispatchBlock, /RELEASE_RUN_ID=[0-9]+/);
  assert.match(
    docs,
    /npm trust github @vizejs\/nuxt-lint-config --file release\.yml --repo ubugeeei-prod\/vize --env npm --allow-publish --yes/,
  );
  assert.match(docs, /Use npm CLI 11\.17\.0 or newer/);
  const bootstrapSectionStart = docs.indexOf("### First-publish bootstrap");
  assert.notEqual(bootstrapSectionStart, -1, "docs must keep the First-publish bootstrap section");
  assert.doesNotMatch(docs.slice(bootstrapSectionStart), /v0\.314\.0/);
  assert.match(docs, /npm-cli-first-publish-vizejs-nuxt-lint-config-X\.Y\.Z/);
  assert.match(docs, /npm publish \.\/vizejs-nuxt-lint-config-X\.Y\.Z\.tgz --access public/);
  assert.match(docs, /does not request an OIDC token/i);
  assert.match(docs, /does not have GitHub Actions OIDC provenance/i);
  assert.doesNotMatch(docs.slice(bootstrapSectionStart), /Granular Access Token|NPM_TOKEN/);
  assert.match(docs, /Freeze `main`/);
  assert.match(docs, /Disable PR auto-merge/);
  assert.match(docs, /neither direct pushes nor other merges/);
  assert.match(docs, /do not create replacement tags/);
});
