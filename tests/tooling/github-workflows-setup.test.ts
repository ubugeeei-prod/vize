import assert from "node:assert/strict";
import { test } from "node:test";

import { hostedOrBlacksmith, readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

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

  assert.match(deployJob, /- uses: actions\/checkout@[0-9a-f]{40}\s*# v6/);
  assert.doesNotMatch(deployJob, /sparse-checkout:/);
});

test("deploy-docs isolates musea example cargo checks from the sticky target cache", () => {
  const manifest = JSON.parse(readRepoFile("examples", "vite-musea", "package.json")) as {
    scripts?: Record<string, string>;
  };
  const checkScript = manifest.scripts?.check;
  assert.ok(checkScript, "examples/vite-musea/package.json must define a check script");

  assert.match(
    checkScript,
    /cargo run\s[^&]*--target-dir\s+\.\.\/\.\.\/target\/docs-example\b/,
    "musea example check script must pin cargo's target dir to target/docs-example so it does not reuse the sticky target cache",
  );
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
    {
      workflowName: "release.yml",
      jobName: "build-wasm-package",
      moonRun:
        "run: moon run --target native - -- < tools/moon/scripts/build_vize_wasm_package.mbtx",
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
  assert.match(action, /uses: actions\/cache@[0-9a-f]{40}\s*# v5/);
  assert.match(action, /Setup MSVC toolchain \(Windows\)/);
  assert.match(action, /uses: ilammy\/msvc-dev-cmd@[0-9a-f]{40}\s*# v1/);
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
  assert.match(installer, /"moonbitlang\/async@0\.19\.0\/process"/);
  assert.match(installer, /@process\.run/);
});

test("setup-moonbit patches Darwin secure memcpy macros before smoke testing", () => {
  const installer = readRepoFile(".github", "actions", "setup-moonbit", "install-moonbit.mjs");

  assert.match(installer, /function patchDarwinMoonbitHeader\(\)/);
  assert.match(installer, /os\.type\(\) !== "Darwin"/);
  assert.match(installer, /#undef memcpy/);
  assert.match(installer, /patchDarwinMoonbitHeader\(\);\nsmokeTestMoon\(\);/);
});

test("setup-moonbit writes both command and shell shims on Windows so bash steps can resolve moon", () => {
  const installer = readRepoFile(".github", "actions", "setup-moonbit", "install-moonbit.mjs");

  assert.match(installer, /const shimMoonCmd = path\.join\(shimDir, "moon\.cmd"\);/);
  assert.match(installer, /const shimMoonShell = path\.join\(shimDir, "moon"\);/);
  assert.match(installer, /fs\.writeFileSync\(\s*shimMoonCmd,/);
  assert.match(installer, /fs\.writeFileSync\(\s*shimMoonShell,/);
});

test("native smoke workflow covers host platforms before release tags", () => {
  const workflow = readRepoFile(".github", "workflows", "native-smoke.yml");
  const job = workflowJobBody(workflow, "host-native-smoke");

  assert.match(workflow, /workflow_dispatch:/);
  assert.match(workflow, /schedule:/);
  assert.doesNotMatch(workflow, /pull_request:/);
  assert.match(
    workflow,
    /Full native\/fresh-install smoke is release evidence, not a per-push gate/,
  );
  for (const [runner, target] of [
    [hostedOrBlacksmith("ubuntu-24.04"), "linux-x64-gnu"],
    [hostedOrBlacksmith("ubuntu-24.04-arm"), "linux-arm64-gnu"],
    ["macos-15-intel", "darwin-x64"],
    [hostedOrBlacksmith("macos-15"), "darwin-arm64"],
    [hostedOrBlacksmith("windows-2025"), "win32-x64-msvc"],
    ["windows-11-arm", "win32-arm64-msvc"],
  ] as const) {
    assert.match(job, new RegExp(`runner:\\s*${runner}[\\s\\S]*target:\\s*${target}`));
  }
  assert.match(job, /cargo build --profile ci -p vize/);
  assert.match(job, /vp run --filter '\.\/npm\/vize-native' build:ci/);
  assert.match(job, /require\('\.\/npm\/vize-native'\)/);
  assert.match(job, /smoke-release-install\.mjs --prepare-manifests npm\/vize-native/);
});

test("native smoke workflow fresh-installs runtime tarballs across supported targets", () => {
  const workflow = readRepoFile(".github", "workflows", "native-smoke.yml");
  const job = workflowJobBody(workflow, "fresh-install-smoke");

  for (const [runner, target] of [
    [hostedOrBlacksmith("ubuntu-24.04"), "linux-x64-gnu"],
    [hostedOrBlacksmith("ubuntu-24.04-arm"), "linux-arm64-gnu"],
    ["macos-15-intel", "darwin-x64"],
    [hostedOrBlacksmith("macos-15"), "darwin-arm64"],
    [hostedOrBlacksmith("windows-2025"), "win32-x64-msvc"],
    ["windows-11-arm", "win32-arm64-msvc"],
  ] as const) {
    assert.match(job, new RegExp(`runner:\\s*${runner}[\\s\\S]*target:\\s*${target}`));
  }
  assert.match(job, /node-version:\s*\["22", "24"\]/);
  assert.match(job, /echo "\$\{\{\s*matrix\.node-version\s*\}\}" > \.node-version\.ci/);
  assert.match(job, /node-version-file:\s*"\.node-version\.ci"/);
  assert.match(job, /vp exec napi create-npm-dirs/);
  assert.match(job, /vp exec napi pre-publish -t npm --no-gh-release --skip-optional-publish/);
  assert.match(
    job,
    /smoke-release-install\.mjs --prepare-manifests --runtime-checks[\s\S]*npm\/vize-native npm\/vize-native\/npm\/\*[\s\S]*npm\/vize npm\/vite-plugin-vize/,
  );
});

test("pkg.pr.new workflow publishes built npm packages from the lockfile", () => {
  const workflow = readRepoFile(".github", "workflows", "pkg-pr-new.yml");
  const job = workflowJobBody(workflow, "publish-preview");

  assert.match(job, /timeout-minutes:\s*30/);
  assert.match(job, /vp run --workspace-root build:packages/);
  assert.match(job, /vp exec pkg-pr-new publish --pnpm --packageManager=pnpm --comment=update/);
  assert.doesNotMatch(job, /\b(?:npx|bunx)\b|pnpm dlx|yarn dlx/);
  assert.equal([...job.matchAll(/pkg-pr-new publish/g)].length, 1);

  for (const packagePath of [
    "./npm/vize",
    "./npm/vite-plugin-vize",
    "./npm/oxlint-plugin-vize",
    "./npm/unplugin-vize",
    "./npm/fresco",
    "./npm/musea-mcp-server",
    "./npm/vite-plugin-musea",
    "./npm/rspack-vize-plugin",
    "./npm/musea-nuxt",
    "./npm/nuxt",
  ]) {
    assert.match(job, new RegExp(packagePath.replaceAll("/", "\\/").replace(".", "\\.")));
  }
});
