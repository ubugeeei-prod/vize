import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("release workflow does not pin a separate hard-coded Node version for VS Code publishing", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.doesNotMatch(workflow, /node-version:\s*"24\.14\.0"/);
  assert.match(workflow, /node-version-file:\s*"\.node-version"/);
});

test("release workflow overwrites existing GitHub release assets when a tag is re-driven", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.match(
    workflow,
    /uses: softprops\/action-gh-release@[0-9a-f]{40}\s*# v2[\s\S]*overwrite_files:\s*true/,
  );
});

test("release workflow publishes npm packages through Trusted Publishing only", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.doesNotMatch(workflow, /secrets\.NPM_TOKEN/);
  assert.doesNotMatch(workflow, /NPM_TOKEN/);
  assert.doesNotMatch(workflow, /configure_npm_auth/);

  const npmPublishJobs = [
    "release-npm-native",
    "release-npm-fresco-native",
    "release-npm-wasm",
    "release-npm-vite-plugin",
    "release-npm-oxlint-plugin",
    "release-npm-unplugin",
    "release-npm-fresco",
    "release-npm-musea-mcp-server",
    "release-npm-vite-plugin-musea",
    "release-npm-rspack-plugin",
    "release-npm-musea-nuxt",
    "release-npm-nuxt",
    "release-npm-cli",
  ];

  for (const jobName of npmPublishJobs) {
    const job = workflowJobBody(workflow, jobName);
    assert.match(job, /runs-on:\s*ubuntu-24\.04\b/);
    assert.doesNotMatch(job, /runs-on:\s*blacksmith-/);
    assert.match(job, /environment:\s*npm/);
    assert.match(job, /id-token:\s*write/);
    assert.match(job, /--provenance/);
    assert.doesNotMatch(job, /NODE_AUTH_TOKEN|_authToken/);
  }
});

test("release workflow publishes npm packages from package-specific artifacts", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.doesNotMatch(workflow, /name:\s*release-npm-packages/);

  for (const artifactName of [
    "release-package-vize",
    "release-package-vite-plugin-vize",
    "release-package-oxlint-plugin-vize",
    "release-package-unplugin-vize",
    "release-package-fresco",
    "release-package-musea-mcp-server",
    "release-package-vite-plugin-musea",
    "release-package-rspack-vize-plugin",
    "release-package-musea-nuxt",
    "release-package-nuxt",
    "release-package-vize-wasm",
  ]) {
    assert.match(workflow, new RegExp(`name:\\s*${artifactName}`));
  }

  const downloadTargets = [
    ["release-npm-wasm", "release-package-vize-wasm", "npm/wasm"],
    ["release-npm-vite-plugin", "release-package-vite-plugin-vize", "npm/builder/vite"],
    ["release-npm-oxlint-plugin", "release-package-oxlint-plugin-vize", "npm/oxint"],
    ["release-npm-unplugin", "release-package-unplugin-vize", "npm/builder/unplugin"],
    ["release-npm-fresco", "release-package-fresco", "npm/fresco"],
    ["release-npm-musea-mcp-server", "release-package-musea-mcp-server", "npm/mcp-musea"],
    [
      "release-npm-vite-plugin-musea",
      "release-package-vite-plugin-musea",
      "npm/builder/vite-musea",
    ],
    ["release-npm-rspack-plugin", "release-package-rspack-vize-plugin", "npm/builder/rspack"],
    ["release-npm-musea-nuxt", "release-package-musea-nuxt", "npm/framework/musea-nuxt"],
    ["release-npm-nuxt", "release-package-nuxt", "npm/framework/nuxt"],
    ["release-npm-cli", "release-package-vize", "npm/cli"],
  ] as const;

  for (const [jobName, artifactName, downloadPath] of downloadTargets) {
    const jobStart = workflow.indexOf(`\n  ${jobName}:\n`);
    assert.notEqual(jobStart, -1, `missing job ${jobName}`);
    const remaining = workflow.slice(jobStart + 1);
    const nextJobMatch = /\n  [a-z0-9-]+:\n/g.exec(remaining.slice(1));
    const jobBody = remaining.slice(0, nextJobMatch ? nextJobMatch.index + 1 : undefined);

    assert.match(jobBody, new RegExp(`name:\\s*${artifactName}`));
    assert.match(jobBody, new RegExp(`path:\\s*${downloadPath.replace("/", "\\/")}`));
  }
});

test("release workflow smokes the wasm package wrapper before publishing", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const buildJob = workflowJobBody(workflow, "build-wasm-package");
  const publishJob = workflowJobBody(workflow, "release-npm-wasm");

  assert.match(buildJob, /runs-on:\s*blacksmith-\d+vcpu-ubuntu-2404/);
  assert.match(buildJob, /npm\/wasm\/index\.js/);
  assert.match(buildJob, /npm\/wasm\/index\.d\.ts/);
  assert.match(buildJob, /tools\/moon\/scripts\/build_vize_wasm_package\.mbtx/);
  assert.match(buildJob, /name:\s*release-package-vize-wasm/);
  assert.match(publishJob, /needs:\s*build-wasm-package/);
  assert.match(publishJob, /name:\s*release-package-vize-wasm/);
  assert.match(publishJob, /path:\s*npm\/wasm/);

  const setupNode = publishJob.indexOf("name: Setup Vite+ and Node.js");
  const download = publishJob.indexOf("name: Download prebuilt WASM package");
  const smoke = publishJob.indexOf("name: Smoke @vizejs/wasm package");
  const publish = publishJob.indexOf("name: Publish @vizejs/wasm");

  assert.notEqual(setupNode, -1);
  assert.notEqual(download, -1);
  assert.notEqual(smoke, -1);
  assert.notEqual(publish, -1);
  assert.ok(setupNode < download && download < smoke && smoke < publish);
  assert.match(publishJob, /node tools\/npm\/smoke-wasm-package\.mjs npm\/wasm/);
});

test("release workflow creates GitHub Releases only after registry publishing succeeds", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const releaseJob = workflowJobBody(workflow, "create-github-release");

  for (const requiredNeed of [
    "build-cli",
    "release-vscode-extension",
    "release-npm-native",
    "release-npm-fresco-native",
    "release-npm-wasm",
    "smoke-release-packages",
    "release-npm-cli",
    "release-npm-vite-plugin",
    "release-npm-oxlint-plugin",
    "release-npm-unplugin",
    "release-npm-fresco",
    "release-npm-musea-mcp-server",
    "release-npm-vite-plugin-musea",
    "release-npm-rspack-plugin",
    "release-npm-musea-nuxt",
    "release-npm-nuxt",
    "release-crates",
  ]) {
    assert.match(releaseJob, new RegExp(`- ${requiredNeed}\\b`));
  }

  const createRelease = releaseJob.indexOf("name: Create Release");
  assert.notEqual(createRelease, -1);
});
