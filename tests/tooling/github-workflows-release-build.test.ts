import assert from "node:assert/strict";
import { test } from "node:test";

import { hostedOrBlacksmith, readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

test("release workflow explicitly installs matrix Rust targets", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  for (const jobName of ["build-cli", "build-native-all"]) {
    const job = workflowJobBody(workflow, jobName);
    const setupRust = job.indexOf("name: Setup Rust");
    const installTarget = job.indexOf("name: Install Rust target");
    const cacheRust = job.indexOf("name: Cache Rust");

    assert.notEqual(setupRust, -1, `${jobName} is missing Setup Rust`);
    assert.notEqual(installTarget, -1, `${jobName} is missing Install Rust target`);
    assert.notEqual(cacheRust, -1, `${jobName} is missing Cache Rust`);
    assert.ok(
      setupRust < installTarget && installTarget < cacheRust,
      `${jobName} must install the matrix Rust target before caching/building`,
    );
    assert.match(
      job,
      /run:\s*rustup target add \$\{\{\s*matrix\.settings\.target\s*\}\}/,
      `${jobName} must install the matrix Rust target explicitly`,
    );
  }
});

test("release workflow plans slow platform cadence before building", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const planJob = workflowJobBody(workflow, "plan-release-platforms");
  const buildCliJob = workflowJobBody(workflow, "build-cli");
  const buildNativeJob = workflowJobBody(workflow, "build-native-all");

  assert.match(planJob, /node tools\/github\/release-platforms\.mjs github-output/);
  assert.match(buildCliJob, /needs:\s*plan-release-platforms/);
  assert.match(
    buildCliJob,
    /settings:\s*\$\{\{\s*fromJSON\(needs\.plan-release-platforms\.outputs\.cli_matrix\)\s*\}\}/,
  );
  assert.match(buildNativeJob, /needs:\s*plan-release-platforms/);
  assert.match(
    buildNativeJob,
    /settings:\s*\$\{\{\s*fromJSON\(needs\.plan-release-platforms\.outputs\.native_matrix\)\s*\}\}/,
  );
  assert.match(workflow, /release-platforms\.mjs apply-cadence/);
});

test("release workflow jobs cap runtime with explicit timeouts", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");

  for (const [jobName, minutes] of [
    ["plan-release-platforms", 5],
    ["build-cli", 90],
    ["build-editor-extensions", 30],
    ["release-vscode-extension", 15],
    ["build-release-packages", 45],
    ["build-wasm-package", 30],
    ["build-native-all", 90],
    ["smoke-release-packages", 30],
    ["release-npm-native", 30],
    ["release-npm-fresco-native", 20],
    ["release-npm-wasm", 30],
    ["release-npm-vite-plugin", 15],
    ["release-npm-oxlint-plugin", 15],
    ["release-npm-unplugin", 15],
    ["release-npm-fresco", 15],
    ["release-npm-musea-mcp-server", 15],
    ["release-npm-vite-plugin-musea", 15],
    ["release-npm-rspack-plugin", 15],
    ["release-npm-musea-nuxt", 15],
    ["release-npm-nuxt", 15],
    ["release-crates", 30],
    ["create-github-release", 20],
    ["release-npm-cli", 15],
  ] as const) {
    assert.match(
      workflowJobBody(workflow, jobName),
      new RegExp(`timeout-minutes:\\s*${minutes}\\b`),
      `${jobName} should have a ${minutes} minute timeout`,
    );
  }
});

test("release workflow smoke installs npm tarballs before publishing", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const smokeJob = workflowJobBody(workflow, "smoke-release-packages");

  assert.match(
    smokeJob,
    /needs:\s*\[plan-release-platforms, build-release-packages, build-native-all\]/,
  );
  assert.match(smokeJob, /name:\s*Smoke release npm package installs/);
  assert.match(smokeJob, /name:\s*Apply slow platform release cadence/);
  assert.match(smokeJob, /name:\s*Prepare native package tarballs/);
  assert.match(smokeJob, /name:\s*Prepare Fresco native package tarball/);
  assert.match(
    smokeJob,
    /node tools\/npm\/smoke-release-install\.mjs --prepare-manifests --runtime-checks/,
  );

  for (const packageDir of [
    "npm/vize-native",
    "npm/fresco-native",
    "npm/vize",
    "npm/vite-plugin-vize",
    "npm/oxlint-plugin-vize",
    "npm/unplugin-vize",
    "npm/fresco",
    "npm/musea-mcp-server",
    "npm/vite-plugin-musea",
    "npm/rspack-vize-plugin",
    "npm/musea-nuxt",
    "npm/nuxt",
  ]) {
    assert.match(smokeJob, new RegExp(packageDir.replaceAll("/", "\\/")));
  }

  for (const jobName of [
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
  ]) {
    assert.match(workflowJobBody(workflow, jobName), /smoke-release-packages/);
  }

  for (const [jobName, smokeStep, publishStep] of [
    [
      "release-npm-native",
      "name: Smoke install native package tarballs",
      "name: Publish platform packages",
    ],
    [
      "release-npm-fresco-native",
      "name: Smoke install Fresco native package tarball",
      "name: Publish",
    ],
    ["release-npm-wasm", "name: Smoke install WASM package tarball", "name: Publish @vizejs/wasm"],
  ] as const) {
    const job = workflowJobBody(workflow, jobName);
    const smokeIndex = job.indexOf(smokeStep);
    const publishIndex = job.indexOf(publishStep);
    assert.notEqual(smokeIndex, -1, `${jobName} is missing ${smokeStep}`);
    assert.notEqual(publishIndex, -1, `${jobName} is missing ${publishStep}`);
    assert.ok(smokeIndex < publishIndex, `${jobName} must smoke install before publishing`);
    if (jobName === "release-npm-native") {
      assert.match(job, /smoke-release-install\.mjs --prepare-manifests --runtime-checks/);
    }
  }
});

test("release workflow builds native targets on MoonBit-supported runners", () => {
  const releasePlatforms = readRepoFile("tools", "github", "release-platforms.mjs");

  assert.doesNotMatch(
    releasePlatforms,
    /host:\s*"macos-15-intel"[\s\S]*target:\s*"x86_64-apple-darwin"/,
    "MoonBit native scripts cannot run on macOS Intel runners",
  );

  for (const [host, target] of [
    [hostedOrBlacksmith("macos-15"), "x86_64-apple-darwin"],
    [hostedOrBlacksmith("macos-15"), "aarch64-apple-darwin"],
    [hostedOrBlacksmith("ubuntu-24.04"), "x86_64-unknown-linux-gnu"],
    [hostedOrBlacksmith("ubuntu-24.04-arm"), "aarch64-unknown-linux-gnu"],
    [hostedOrBlacksmith("windows-2025"), "x86_64-pc-windows-msvc"],
    ["windows-11-arm", "aarch64-pc-windows-msvc"],
  ] as const) {
    assert.match(releasePlatforms, new RegExp(`host:\\s*"${host}"[\\s\\S]*target:\\s*"${target}"`));
  }
});

test("release workflow keeps the Windows ARM64 CLI cross build on a compatible hosted runner", () => {
  const releasePlatforms = readRepoFile("tools", "github", "release-platforms.mjs");

  assert.match(
    releasePlatforms,
    /host:\s*"windows-2025",\n\s*target:\s*"aarch64-pc-windows-msvc"/,
    "Blacksmith Windows x64 images expose x64 MSVC SDK libs after setup-moonbit, which breaks ARM64 linking",
  );
  assert.doesNotMatch(
    releasePlatforms,
    /host:\s*"blacksmith-\d+vcpu-windows-2025",\n\s*target:\s*"aarch64-pc-windows-msvc"/,
  );
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

test("release workflow tunes Windows production Rust builds for cold runners", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const profileSteps = [...workflow.matchAll(/- name: Tune Windows release profile/g)];

  assert.equal(profileSteps.length, 2);
  assert.match(
    workflow,
    /Tune Windows release profile[\s\S]*if: runner\.os == 'Windows'[\s\S]*CARGO_PROFILE_RELEASE_LTO=thin/,
  );
  assert.match(
    workflow,
    /Tune Windows release profile[\s\S]*CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16/,
  );
  assert.match(
    workflow,
    /Tune Windows release profile[\s\S]*Build CLI[\s\S]*cargo build --release -p vize --target \$\{\{ matrix\.settings\.target \}\}/,
  );
  assert.match(
    workflow,
    /Tune Windows release profile[\s\S]*Build vize-native[\s\S]*tools\/moon\/scripts\/github\/build_napi_package\.mbtx/,
  );
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
