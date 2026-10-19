import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  checkWorkflowJobSources,
  hostedOrBlacksmith,
  readRepoFile,
  workflowJobBody,
} from "./support/github-workflows.ts";

type WorkflowJob = {
  steps?: Array<{ uses?: string; with?: Record<string, unknown> }>;
  strategy?: { matrix?: Record<string, unknown> };
};

test("apt-based CI setup pins Blacksmith to the canonical Ubuntu archive", () => {
  const action = readRepoFile(".github", "actions", "setup-ubuntu-archive", "action.yml");
  assert.match(action, /\/etc\/apt\/blacksmith-ubuntu-mirrors\.txt/);
  assert.match(action, /https:\/\/archive\.ubuntu\.com\/ubuntu/);
  assert.match(action, /https:\/\/security\.ubuntu\.com\/ubuntu/);
  assert.match(action, /Acquire::Retries "5";/);
  assert.match(action, /Acquire::http::Timeout "30";/);
  assert.match(action, /Acquire::https::Timeout "30";/);
  assert.match(action, /Acquire::Queue-Mode "access";/);
  assert.match(action, /Acquire::http::Pipeline-Depth "0";/);
  assert.match(action, /Acquire::https::Pipeline-Depth "0";/);
  assert.match(action, /\/usr\/local\/bin\/vize-ci-apt-retry/);
  assert.match(action, /VIZE_CI_APT_RETRIES:-4/);

  const row = readRepoFile(".github", "actions", "app-e2e-row", "action.yml");
  const rowSetup = row.indexOf("uses: ./.github/actions/setup-ubuntu-archive");
  const rowApt = row.indexOf("playwright install-deps chromium");
  assert.ok(rowSetup >= 0 && rowSetup < rowApt);
  assert.match(
    row,
    /vize-ci-apt-retry vp exec --filter '\.\/tests' -- playwright install-deps chromium/,
  );

  const editor = readRepoFile(".github", "actions", "vscode-host-smoke", "action.yml");
  const editorSetup = editor.indexOf("uses: ./.github/actions/setup-ubuntu-archive");
  const editorApt = editor.indexOf("sudo apt-get update");
  assert.ok(editorSetup >= 0 && editorSetup < editorApt);

  const check = workflowJobBody(checkWorkflowJobSources(), "playground-test");
  assert.ok(check.includes("uses: ./.github/actions/install-playwright-browsers"));

  const playground = readRepoFile(
    ".github",
    "actions",
    "install-playwright-browsers",
    "action.yml",
  );
  const playgroundSetup = playground.indexOf("uses: ./.github/actions/setup-ubuntu-archive");
  const playgroundInstall = playground.indexOf("install-playwright-browsers.ts");
  assert.ok(playgroundSetup >= 0 && playgroundSetup < playgroundInstall);
  assert.match(
    playground,
    /vize-ci-apt-retry node tools\/support\/github\/install-playwright-browsers\.ts$/m,
  );
});

test("deploy-docs deploy job sets up Node before running the site assembler", () => {
  const workflow = readRepoFile(".github", "workflows", "deploy-docs.yml");
  const deployJob = workflow.slice(workflow.indexOf("\n  deploy:\n"));
  const setupIndex = deployJob.indexOf("uses: voidzero-dev/setup-vp@");
  const runIndex = deployJob.indexOf("run: node tools/support/github/create-site-structure.ts\n");

  assert.notEqual(setupIndex, -1);
  assert.notEqual(runIndex, -1);
  assert.ok(setupIndex < runIndex);
  assert.doesNotMatch(deployJob, /setup-moonbit|moon run/);
});

test("deploy-docs deploy job keeps a full checkout so local actions and scripts remain available", () => {
  const workflow = readRepoFile(".github", "workflows", "deploy-docs.yml");
  const deployJob = workflow.slice(workflow.indexOf("\n  deploy:\n"));

  assert.match(deployJob, /uses: actions\/checkout@[0-9a-f]{40}\s*# v6/);
  assert.doesNotMatch(deployJob, /sparse-checkout:/);
});

test("docs build isolates musea example cargo checks in a dedicated sticky disk", () => {
  const manifest = JSON.parse(readRepoFile("examples", "vite-musea", "package.json")) as {
    scripts?: Record<string, string>;
  };
  const checkScript = manifest.scripts?.check;
  assert.ok(checkScript, "examples/vite-musea/package.json must define a check script");

  assert.match(
    checkScript,
    /cargo run\s[^&]*--target-dir\s+\.\.\/\.\.\/\.docs-example-target\b/,
    "musea example check script must use the dedicated docs example target",
  );

  const workflow = readRepoFile(".github", "workflows", "build-docs.yml");
  const job = workflowJobBody(workflow, "build-playground");
  assert.match(job, /secondary-key: docs-example/);
  assert.match(job, /secondary-target-path: \.docs-example-target/);
  assert.doesNotMatch(job, /cargo clean --target-dir/);
});

test("WASM build jobs set up Node before invoking the TypeScript builders", () => {
  const cases = [
    {
      workflowName: "full-qualification.yml",
      jobName: "playground-test",
      setup: "- name: Setup Vite+ and Node.js",
      command: "run: node tools/support/github/build-vitrine-wasm.ts playground/src/wasm",
    },
    {
      workflowName: "pr-source-checks.yml",
      jobName: "pr-playground-test",
      setup: "- uses: voidzero-dev/setup-vp@",
      command: "run: node tools/support/github/build-vitrine-wasm.ts playground/src/wasm",
    },
    {
      workflowName: "build-docs.yml",
      jobName: "build-playground",
      setup: "- name: Setup Vite+ and Node.js",
      command: "run: node tools/support/github/build-vitrine-wasm.ts npm/wasm playground/src/wasm",
    },
    {
      workflowName: "release.yml",
      jobName: "build-wasm",
      setup: "- uses: actions/setup-node@",
      command: "run: node tools/support/tasks/build-vize-wasm-package.ts",
    },
  ] as const;

  for (const { workflowName, jobName, setup, command } of cases) {
    const workflow = readRepoFile(".github", "workflows", workflowName);
    const jobStart = workflow.indexOf(`\n  ${jobName}:\n`);
    const remaining = workflow.slice(jobStart + 1);
    const nextJobMatch = /\n  [a-z0-9-]+:\n/g.exec(remaining.slice(1));
    const jobBody = remaining.slice(0, nextJobMatch ? nextJobMatch.index + 1 : undefined);
    const setupIndex = jobBody.indexOf(setup);
    const commandIndex = jobBody.indexOf(command);

    assert.notEqual(setupIndex, -1, `${workflowName}:${jobName} is missing its Node setup`);
    assert.notEqual(commandIndex, -1, `${workflowName}:${jobName} is missing the wasm build step`);
    assert.ok(setupIndex < commandIndex, `${workflowName}:${jobName} runs Node before its setup`);
    assert.doesNotMatch(jobBody, /setup-moonbit|moon run/, `${workflowName}:${jobName}`);
  }
});

test("setup-rust-script pins Windows MSVC linkers before cargo install", () => {
  const action = readRepoFile(".github", "actions", "setup-rust-script", "action.yml");
  const linkerIndex = action.indexOf("Configure Windows Rust linker");
  const installIndex = action.indexOf("Install rust-script");

  assert.ok(linkerIndex >= 0, "setup-rust-script must configure Windows linkers");
  assert.ok(installIndex >= 0, "setup-rust-script must install rust-script");
  assert.ok(linkerIndex < installIndex, "Windows linker config must run before cargo install");
  assert.match(action, /if: runner\.os == 'Windows'/);
  assert.match(action, /CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=rust-lld/);
  assert.match(action, /CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER=rust-lld/);
  assert.match(action, />> "\$GITHUB_ENV"/);
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
    [hostedOrBlacksmith("ubuntu-22.04"), "linux-x64-gnu"],
    [hostedOrBlacksmith("ubuntu-22.04-arm"), "linux-arm64-gnu"],
    ["macos-15-intel", "darwin-x64"],
    [hostedOrBlacksmith("macos-15"), "darwin-arm64"],
    [hostedOrBlacksmith("windows-2025"), "win32-x64-msvc"],
    ["windows-11-arm", "win32-arm64-msvc"],
  ] as const) {
    assert.match(job, new RegExp(`runner:\\s*${runner}[\\s\\S]*target:\\s*${target}`));
  }
  assert.match(job, /cargo build --profile ci -p vize/);
  assert.match(job, /vp run --filter '\.\/npm\/native' build:ci/);
  assert.match(job, /verify-glibc-symbols\.rs --max 2\.36 npm\/native\/\*\.linux-\*-gnu\.node/);
  assert.match(job, /require\('\.\/npm\/native'\)/);
  assert.match(job, /smoke-release-install\.rs --prepare-manifests npm\/native/);
});

test("native smoke workflow fresh-installs runtime tarballs across supported targets", () => {
  const workflow = readRepoFile(".github", "workflows", "native-smoke.yml");
  const parsed = parse(workflow) as { jobs?: Record<string, WorkflowJob> };
  const parsedJob = parsed.jobs?.["fresh-install-smoke"];
  const job = workflowJobBody(workflow, "fresh-install-smoke");
  const runtimeSetup = parsedJob?.steps?.find(
    (step) => step.uses === "./.github/actions/setup-runtime-package-managers",
  );

  for (const [runner, target] of [
    [hostedOrBlacksmith("ubuntu-22.04"), "linux-x64-gnu"],
    [hostedOrBlacksmith("ubuntu-22.04-arm"), "linux-arm64-gnu"],
    ["macos-15-intel", "darwin-x64"],
    [hostedOrBlacksmith("macos-15"), "darwin-arm64"],
    [hostedOrBlacksmith("windows-2025"), "win32-x64-msvc"],
    ["windows-11-arm", "win32-arm64-msvc"],
  ] as const) {
    assert.match(job, new RegExp(`runner:\\s*${runner}[\\s\\S]*target:\\s*${target}`));
  }
  assert.deepEqual(parsedJob?.strategy?.matrix?.["node-version"], ["22", "24"]);
  assert.doesNotMatch(job, /\.node-version\.ci/);
  assert.equal(runtimeSetup?.with?.["node-version"], "${{ matrix.node-version }}");
  assert.match(job, /vp exec napi create-npm-dirs/);
  assert.match(job, /vp exec napi pre-publish -t npm --no-gh-release --skip-optional-publish/);
  assert.match(
    job,
    /smoke-release-install\.rs --prepare-manifests --runtime-checks[\s\S]*npm\/native npm\/native\/npm\/\*[\s\S]*npm\/cli npm\/builder\/vite/,
  );
});

test("pkg.pr.new workflow publishes built npm packages from the lockfile", () => {
  const workflow = readRepoFile(".github", "workflows", "pkg-pr-new.yml");
  const job = workflowJobBody(workflow, "publish-preview");

  assert.match(job, /runs-on:\s*blacksmith-32vcpu-ubuntu-2404/);
  assert.match(job, /timeout-minutes:\s*30/);
  const rustScriptInstallIndex = job.indexOf("name: Install rust-script");
  const buildIndex = job.indexOf("run: vp run --workspace-root build:packages");
  assert.ok(rustScriptInstallIndex >= 0, "pkg.pr.new workflow must install rust-script");
  assert.ok(buildIndex >= 0, "pkg.pr.new workflow must build packages");
  assert.ok(
    rustScriptInstallIndex < buildIndex,
    "rust-script must be installed before package builds",
  );
  const rustChannel = /^channel = "([^"]+)"$/m.exec(readRepoFile("rust-toolchain.toml"))?.[1];
  assert.ok(rustChannel, "repository must pin its Rust toolchain");
  assert.ok(
    job.includes(`rustup toolchain install ${rustChannel} --profile minimal --no-self-update`),
  );
  assert.ok(job.includes(`rustup default ${rustChannel}\n`));
  assert.match(job, /grep -Eq '\(\^\|\[\[:space:\]\]\)0\\\.34\\\.0\(\[\[:space:\]\]\|\$\)'/);
  assert.match(job, /cargo install rust-script --version 0\.34\.0 --locked/);
  assert.match(job, /vp run --workspace-root build:packages/);
  assert.match(job, /vp exec pkg-pr-new publish --pnpm --packageManager=pnpm --comment=update/);
  assert.doesNotMatch(job, /\b(?:npx|bunx)\b|pnpm dlx|yarn dlx/);
  assert.equal([...job.matchAll(/pkg-pr-new publish/g)].length, 1);

  for (const packagePath of [
    "./npm/cli",
    "./npm/builder/vite",
    "./npm/oxlint",
    "./npm/builder/unplugin",
    "./npm/fresco",
    "./npm/mcp-musea",
    "./npm/builder/vite-musea",
    "./npm/builder/rspack",
    "./npm/framework/musea-nuxt",
    "./npm/framework/nuxt-lint-config",
    "./npm/framework/nuxt",
  ]) {
    assert.match(job, new RegExp(packagePath.replaceAll("/", "\\/").replace(".", "\\.")));
  }
});
