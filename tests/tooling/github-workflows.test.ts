import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

test("GitHub workflows opt JavaScript actions into Node 24", () => {
  for (const workflowName of ["check.yml", "deploy-docs.yml", "release.yml"]) {
    const workflow = readRepoFile(".github", "workflows", workflowName);
    assert.match(workflow, /FORCE_JAVASCRIPT_ACTIONS_TO_NODE24:\s*true/);
  }
});

test("deploy-docs deploy job installs MoonBit before running script-mode helpers", () => {
  const workflow = readRepoFile(".github", "workflows", "deploy-docs.yml");
  const deployJob = workflow.slice(workflow.indexOf("\n  deploy:\n"));
  const setupIndex = deployJob.indexOf("- uses: ./.github/actions/setup-moonbit");
  const moonRunIndex = deployJob.indexOf(
    "run: moon run --target native - -- < tools/moon/scripts/github/create_site_structure.mbtx",
  );

  assert.notEqual(setupIndex, -1);
  assert.notEqual(moonRunIndex, -1);
  assert.ok(setupIndex < moonRunIndex);
});

test("deploy-docs deploy job keeps a full checkout so local actions and scripts remain available", () => {
  const workflow = readRepoFile(".github", "workflows", "deploy-docs.yml");
  const deployJob = workflow.slice(workflow.indexOf("\n  deploy:\n"));

  assert.match(deployJob, /- uses: actions\/checkout@v4/);
  assert.doesNotMatch(deployJob, /sparse-checkout:/);
});

test("WASM build jobs install MoonBit before invoking moon run", () => {
  const cases = [
    {
      workflowName: "check.yml",
      jobName: "playground-test",
      moonRun:
        "run: moon run --target native - -- playground/src/wasm < tools/moon/scripts/github/build_vitrine_wasm.mbtx",
    },
    {
      workflowName: "deploy-docs.yml",
      jobName: "build-playground",
      moonRun:
        "run: moon run --target native - -- npm/vize-wasm playground/src/wasm < tools/moon/scripts/github/build_vitrine_wasm.mbtx",
    },
  ] as const;

  for (const { workflowName, jobName, moonRun } of cases) {
    const workflow = readRepoFile(".github", "workflows", workflowName);
    const jobStart = workflow.indexOf(`\n  ${jobName}:\n`);
    const remaining = workflow.slice(jobStart + 1);
    const nextJobMatch = /\n  [a-z0-9-]+:\n/g.exec(remaining.slice(1));
    const jobBody = remaining.slice(0, nextJobMatch ? nextJobMatch.index + 1 : undefined);
    const setupIndex = jobBody.indexOf("- uses: ./.github/actions/setup-moonbit");
    const moonRunIndex = jobBody.indexOf(moonRun);

    assert.notEqual(setupIndex, -1, `${workflowName}:${jobName} is missing setup-moonbit`);
    assert.notEqual(moonRunIndex, -1, `${workflowName}:${jobName} is missing the wasm build step`);
    assert.ok(
      setupIndex < moonRunIndex,
      `${workflowName}:${jobName} runs moon before setup-moonbit`,
    );
  }
});

test("setup-moonbit defines explicit Windows and Unix execution paths", () => {
  const action = readRepoFile(".github", "actions", "setup-moonbit", "action.yml");

  assert.match(action, /Cache MoonBit toolchain/);
  assert.match(action, /uses: actions\/cache@v4/);
  assert.match(action, /Setup MSVC toolchain \(Windows\)/);
  assert.match(action, /uses: ilammy\/msvc-dev-cmd@v1/);
  assert.match(action, /Install MoonBit \(Windows\)/);
  assert.match(action, /if: runner\.os == 'Windows'/);
  assert.match(action, /shell: pwsh/);
  assert.match(action, /Install MoonBit \(Unix\)/);
  assert.match(action, /if: runner\.os != 'Windows'/);
  assert.match(action, /shell: bash/);
});

test("setup-moonbit smoke test validates the native async process runtime", () => {
  const installer = readRepoFile(".github", "actions", "setup-moonbit", "install-moonbit.mjs");

  assert.match(installer, /function hasExistingMoonInstall\(\)/);
  assert.match(installer, /\["run", "-q", "--target", "native", "-", "--"\]/);
  assert.match(installer, /"moonbitlang\/async@0\.16\.8\/process"/);
  assert.match(installer, /@process\.run/);
});

test("setup-moonbit writes both command and shell shims on Windows so bash steps can resolve moon", () => {
  const installer = readRepoFile(".github", "actions", "setup-moonbit", "install-moonbit.mjs");

  assert.match(installer, /const shimMoonCmd = path\.join\(shimDir, "moon\.cmd"\);/);
  assert.match(installer, /const shimMoonShell = path\.join\(shimDir, "moon"\);/);
  assert.match(installer, /fs\.writeFileSync\(\s*shimMoonCmd,/);
  assert.match(installer, /fs\.writeFileSync\(\s*shimMoonShell,/);
});

test("release workflow does not pin a separate hard-coded Node version for VS Code publishing", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.doesNotMatch(workflow, /node-version:\s*"24\.14\.0"/);
  assert.match(workflow, /node-version-file:\s*"\.node-version"/);
});

test("release workflow overwrites existing GitHub release assets when a tag is re-driven", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.match(workflow, /uses: softprops\/action-gh-release@v2[\s\S]*overwrite_files:\s*true/);
});

test("release workflow configures npm auth fallback for every npm publish job", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const fallbackSteps = [...workflow.matchAll(/- name: Configure npm auth fallback/g)];

  assert.equal(fallbackSteps.length, 13);
  assert.match(workflow, /NPM_TOKEN:\s*\$\{\{\s*secrets\.NPM_TOKEN\s*\}\}/);
  assert.match(workflow, /tools\/moon\/scripts\/github\/configure_npm_auth\.mbtx/);
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
  ]) {
    assert.match(workflow, new RegExp(`name:\\s*${artifactName}`));
  }

  const downloadTargets = [
    ["release-npm-vite-plugin", "release-package-vite-plugin-vize", "npm/vite-plugin-vize"],
    ["release-npm-oxlint-plugin", "release-package-oxlint-plugin-vize", "npm/oxlint-plugin-vize"],
    ["release-npm-unplugin", "release-package-unplugin-vize", "npm/unplugin-vize"],
    ["release-npm-fresco", "release-package-fresco", "npm/fresco"],
    ["release-npm-musea-mcp-server", "release-package-musea-mcp-server", "npm/musea-mcp-server"],
    ["release-npm-vite-plugin-musea", "release-package-vite-plugin-musea", "npm/vite-plugin-musea"],
    ["release-npm-rspack-plugin", "release-package-rspack-vize-plugin", "npm/rspack-vize-plugin"],
    ["release-npm-musea-nuxt", "release-package-musea-nuxt", "npm/musea-nuxt"],
    ["release-npm-nuxt", "release-package-nuxt", "npm/nuxt"],
    ["release-npm-cli", "release-package-vize", "npm/vize"],
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

test("release workflow bundles fresco-native binaries into the root package instead of publishing platform packages", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const frescoJobStart = workflow.indexOf("\n  release-npm-fresco-native:\n");
  const nextJobStart = workflow.indexOf("\n  # Build and publish WASM package", frescoJobStart);
  const frescoJob = workflow.slice(frescoJobStart, nextJobStart);

  assert.match(
    frescoJob,
    /Clean bundled native binaries[\s\S]*tools\/moon\/scripts\/github\/clean_node_binaries\.mbtx/,
  );
  assert.match(
    frescoJob,
    /Stage bundled native binaries[\s\S]*tools\/moon\/scripts\/github\/collect_native_artifacts\.mbtx/,
  );
  assert.doesNotMatch(frescoJob, /napi create-npm-dirs/);
  assert.doesNotMatch(frescoJob, /publish_npm_package_dirs\.mbtx/);
});

test("cargo config forces the bundled Rust linker for Windows MSVC targets", () => {
  const cargoConfig = readRepoFile(".cargo", "config.toml");

  assert.match(cargoConfig, /\[target\.x86_64-pc-windows-msvc\]\s*linker = "rust-lld"/);
  assert.match(cargoConfig, /\[target\.aarch64-pc-windows-msvc\]\s*linker = "rust-lld"/);
});

test("release workflow runs GitHub helper scripts with the native target on every runner", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  assert.doesNotMatch(workflow, /MOON_HELPER_TARGET/);
  assert.match(
    workflow,
    /Install cross-compilation tools \(Linux ARM64\)[\s\S]*moon run --target native - -- < tools\/moon\/scripts\/github\/install_cross_compile_tools\.mbtx/,
  );
  assert.match(
    workflow,
    /Create archive \(Windows\)[\s\S]*moon run --target native - -- \$\{\{ matrix\.settings\.target \}\} \$\{\{ matrix\.settings\.archive \}\} vize\.exe < tools\/moon\/scripts\/github\/create_cli_archive\.mbtx/,
  );
  assert.match(workflow, /Build vize-native[\s\S]*moon run --target native - -- npm\/vize-native/);
});

test("check workflow only installs Playwright browsers on cache misses", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");

  assert.match(workflow, /- name: Cache Playwright browsers\s+id: cache-playwright/);
  assert.match(
    workflow,
    /- name: Install Playwright browsers\s+if: steps\.cache-playwright\.outputs\.cache-hit != 'true'/,
  );
});

test("check and docs workflows use the CI Rust profile for non-release native builds", () => {
  const checkWorkflow = readRepoFile(".github", "workflows", "check.yml");
  const deployDocsWorkflow = readRepoFile(".github", "workflows", "deploy-docs.yml");

  assert.match(checkWorkflow, /cargo build --profile ci -p vize/);
  assert.match(checkWorkflow, /cp target\/ci\/vize \/usr\/local\/bin\/vize/);
  assert.match(checkWorkflow, /vp run --filter '\.\/npm\/vize-native' build:ci/);
  assert.match(deployDocsWorkflow, /vp run --filter '\.\/npm\/vize-native' build:ci/);
});
