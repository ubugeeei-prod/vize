import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { hostedOrBlacksmith, readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

type ReleaseJob = {
  env?: Record<string, string>;
  environment?: string;
  if?: string;
  needs?: string | string[];
  permissions?: Record<string, string>;
  "runs-on"?: string;
  steps?: Array<{
    env?: Record<string, string>;
    run?: string;
    uses?: string;
    with?: Record<string, unknown>;
  }>;
  "timeout-minutes"?: number;
  uses?: string;
};

function jobNeeds(job: ReleaseJob): string[] {
  if (job.needs == null) return [];
  return Array.isArray(job.needs) ? job.needs : [job.needs];
}

type PublishedManifest = {
  name: string;
  dependencies?: Record<string, string>;
  optionalDependencies?: Record<string, string>;
  peerDependencies?: Record<string, string>;
  peerDependenciesMeta?: Record<string, { optional?: boolean }>;
  scripts?: Record<string, string>;
};

function assertNoPublishHooks(manifest: PublishedManifest): void {
  assert.deepEqual(
    Object.keys(manifest.scripts ?? {}).filter((name) =>
      ["prepublishOnly", "prepare", "prepack", "postpack", "publish", "postpublish"].includes(name),
    ),
    [],
    `${manifest.name} publishes its prebuilt artifact without lifecycle hooks`,
  );
}

function assertPublishedDependencies(
  jobs: Record<string, ReleaseJob>,
  manifests: Array<[string, PublishedManifest]>,
): void {
  const owners = new Map(manifests.map(([job, manifest]) => [manifest.name, job]));
  const native = manifests.find(([job]) => job === "release-npm-native")?.[1];
  for (const name of Object.keys(native?.optionalDependencies ?? {}))
    owners.set(name, "release-npm-native");
  function waitsFor(job: string, target: string, seen = new Set<string>()): boolean {
    if (seen.has(job)) return false;
    seen.add(job);
    return jobNeeds(jobs[job]).some(
      (dependency) => dependency === target || waitsFor(dependency, target, seen),
    );
  }
  for (const [job, manifest] of manifests) {
    const peers = Object.keys(manifest.peerDependencies ?? {}).filter(
      (name) => !manifest.peerDependenciesMeta?.[name]?.optional,
    );
    for (const name of [
      ...Object.keys(manifest.dependencies ?? {}),
      ...Object.keys(manifest.optionalDependencies ?? {}),
      ...peers,
    ]) {
      const dependency = owners.get(name);
      if (dependency && dependency !== job)
        assert.equal(waitsFor(job, dependency), true, `${manifest.name} latest requires ${name}`);
    }
  }
}

function publicationJobNames(jobs: Record<string, ReleaseJob>): string[] {
  return Object.entries(jobs)
    .filter(([, job]) => {
      if (job.if === "inputs.release_pr == ''") return false;
      const serialized = JSON.stringify(job);
      return (
        ["npm", "crates-io", "vscode-marketplace"].includes(job.environment ?? "") ||
        /publish_npm_package|npm publish|cargo publish|vsce publish|crates-io-auth-action@|softprops\/action-gh-release@/.test(
          serialized,
        )
      );
    })
    .map(([name]) => name)
    .sort();
}

const sharedPublisher = "./.github/actions/publish-release-package";
const sharedRecipe = () =>
  parse(readRepoFile(".github", "actions", "publish-release-package", "action.yml"));

function setupVpSteps(job: ReleaseJob): NonNullable<ReleaseJob["steps"]> {
  return (job.steps ?? [])
    .flatMap((step) => (step.uses === sharedPublisher ? sharedRecipe().runs.steps : [step]))
    .filter((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
}

test("every publication edge waits for credential-free release preflight", () => {
  const source = readRepoFile(".github", "workflows", "release.yml");
  const workflow = parse(source) as { jobs?: Record<string, ReleaseJob> };
  const jobs = workflow.jobs ?? {};
  const publishJobs = publicationJobNames(jobs);

  assert.deepEqual(publishJobs, [
    "create-github-release",
    "release-crates",
    "release-npm-cli",
    "release-npm-composable",
    "release-npm-fresco",
    "release-npm-fresco-native",
    "release-npm-marquette",
    "release-npm-musea-mcp-server",
    "release-npm-musea-nuxt",
    "release-npm-native",
    "release-npm-nuxt",
    "release-npm-nuxt-lint-config",
    "release-npm-oxlint-plugin",
    "release-npm-plugin-sdk",
    "release-npm-rspack-plugin",
    "release-npm-ui",
    "release-npm-unplugin",
    "release-npm-vite-plugin",
    "release-npm-vite-plugin-musea",
    "release-npm-wasm",
    "release-vscode-extension",
  ]);
  for (const jobName of publishJobs) {
    assert.ok(jobNeeds(jobs[jobName]).includes("release-preflight"), jobName);
  }

  const preflight = jobs["release-preflight"];
  assert.ok(preflight);
  assert.equal(preflight.uses, "./.github/workflows/release-promotion.yml");
  assert.deepEqual(preflight.permissions, {
    contents: "read",
    "pull-requests": "read",
  });
  assert.deepEqual(jobNeeds(preflight), ["candidate-ready"]);
  assert.doesNotMatch(JSON.stringify(preflight), /environment|id-token|secrets\./);
});

test("release workflow runtime setup reads the manifest for every publishing job", () => {
  const source = readRepoFile(".github", "workflows", "release.yml");
  const workflow = parse(source) as { jobs?: Record<string, ReleaseJob> };
  const jobs = workflow.jobs ?? {};

  assert.doesNotMatch(source, /node-version:\s*"24\.14\.0"/);
  for (const jobName of publicationJobNames(jobs)) {
    for (const step of setupVpSteps(jobs[jobName])) {
      assert.equal(step.with?.["node-version-file"], "package.json", jobName);
      assert.equal(step.with?.["node-version"], undefined, jobName);
    }
  }
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
    "release-npm-nuxt-lint-config",
    "release-npm-nuxt",
    "release-npm-cli",
    "release-npm-marquette",
    "release-npm-composable",
    "release-npm-ui",
    "release-npm-plugin-sdk",
  ];

  for (const jobName of npmPublishJobs) {
    const job = workflowJobBody(workflow, jobName);
    assert.match(job, /runs-on:\s*ubuntu-24\.04\b/);
    assert.doesNotMatch(job, /runs-on:\s*blacksmith-/);
    assert.match(job, /environment:\s*npm/);
    assert.match(job, /id-token:\s*write/);
    const commands = job.includes(sharedPublisher)
      ? JSON.stringify(sharedRecipe().runs.steps)
      : job;
    assert.match(commands, /--provenance/);
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
    "release-package-nuxt-lint-config",
    "release-package-nuxt",
    "release-package-vize-wasm",
    "release-package-marquette",
    "release-package-composable",
    "release-package-ui",
  ]) {
    assert.match(workflow, new RegExp(`(?:name|artifact):\\s*${artifactName}`));
  }

  const downloadTargets = [
    ["release-npm-wasm", "release-package-vize-wasm", "npm/wasm"],
    ["release-npm-vite-plugin", "release-package-vite-plugin-vize", "npm/builder/vite"],
    ["release-npm-oxlint-plugin", "release-package-oxlint-plugin-vize", "npm/oxlint"],
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
    [
      "release-npm-nuxt-lint-config",
      "release-package-nuxt-lint-config",
      "npm/framework/nuxt-lint-config",
    ],
    ["release-npm-nuxt", "release-package-nuxt", "npm/framework/nuxt"],
    ["release-npm-cli", "release-package-vize", "npm/cli"],
    ["release-npm-marquette", "release-package-marquette", "npm/marquette"],
    ["release-npm-composable", "release-package-composable", "npm/compose/core"],
    ["release-npm-ui", "release-package-ui", "npm/ui"],
    ["release-npm-plugin-sdk", "release-package-plugin-sdk", "npm/plugin-sdk"],
  ] as const;

  const jobs = (parse(workflow) as { jobs: Record<string, ReleaseJob> }).jobs;
  const manifests: Array<[string, PublishedManifest]> = [];
  for (const [jobName, artifactName, downloadPath] of downloadTargets) {
    const shared = jobs[jobName].steps?.find((step) => step.uses === sharedPublisher);
    const download = jobs[jobName].steps?.find((step) =>
      step.uses?.startsWith("actions/download-artifact@"),
    );
    assert.deepEqual(
      shared?.with ?? download?.with,
      shared
        ? { artifact: artifactName, "package-path": downloadPath }
        : { name: artifactName, path: downloadPath },
      jobName,
    );
    const manifest = JSON.parse(
      readRepoFile(...downloadPath.split("/"), "package.json"),
    ) as PublishedManifest;
    assertNoPublishHooks(manifest);
    manifests.push([jobName, manifest]);
  }
  for (const [job, directory] of [
    ["release-npm-native", "native"],
    ["release-npm-fresco-native", "fresco-native"],
  ])
    manifests.push([job, JSON.parse(readRepoFile("npm", directory, "package.json"))]);
  assertPublishedDependencies(jobs, manifests);

  for (const job of ["release-npm-plugin-sdk", "release-npm-musea-nuxt"])
    assert.deepEqual(jobNeeds(jobs[job]), [
      "build-release-packages",
      "smoke-release-packages",
      "release-preflight",
    ]);
  assert.equal(jobNeeds(jobs["release-npm-nuxt"]).includes("release-npm-musea-nuxt"), false);
  assert.deepEqual(jobNeeds(jobs["release-jsr"]), [
    "release-npm-cli",
    "release-npm-native",
    "release-npm-vite-plugin",
  ]);

  for (const [job, dependency] of [
    ["release-npm-cli", "release-npm-native"],
    ["release-npm-vite-plugin", "release-npm-unplugin"],
    ["release-npm-vite-plugin-musea", "release-npm-vite-plugin"],
  ]) {
    const premature = structuredClone(jobs);
    premature[job].needs = jobNeeds(premature[job]).filter((name) => name !== dependency);
    assert.throws(() => assertPublishedDependencies(premature, manifests), /latest requires/);
  }
  assert.throws(
    () => assertNoPublishHooks({ name: "future-publisher", scripts: { prepack: "npm install" } }),
    /without lifecycle hooks/,
  );
});

test("release workflow smokes the wasm package wrapper before publishing", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const buildJob = workflowJobBody(workflow, "build-wasm-package");
  const publishJob = workflowJobBody(workflow, "release-npm-wasm");

  assert.match(buildJob, new RegExp(`runs-on:\\s*${hostedOrBlacksmith("ubuntu-24.04")}`));
  assert.match(buildJob, /npm\/wasm\/index\.js/);
  assert.match(buildJob, /npm\/wasm\/index\.d\.ts/);
  assert.match(buildJob, /node tools\/support\/tasks\/build-vize-wasm-package\.ts/);
  assert.match(buildJob, /name:\s*release-package-vize-wasm/);
  const publishWorkflow = parse(workflow) as { jobs?: Record<string, ReleaseJob> };
  const publishWasm = publishWorkflow.jobs?.["release-npm-wasm"];
  assert.ok(publishWasm);
  assert.ok(jobNeeds(publishWasm).includes("build-wasm-package"));
  assert.ok(jobNeeds(publishWasm).includes("release-preflight"));
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
  assert.match(
    publishJob,
    /rust-script tools\/commands\/release\/npm\/smoke-wasm-package\.rs npm\/wasm/,
  );
});

test("release workflow creates GitHub Releases only after registry publishing succeeds", () => {
  const source = readRepoFile(".github", "workflows", "release.yml");
  const workflow = parse(source) as { jobs?: Record<string, ReleaseJob> };
  const jobs = workflow.jobs ?? {};
  const releaseJob = jobs["create-github-release"];
  assert.ok(releaseJob);
  const releaseNeeds = jobNeeds(releaseJob);

  for (const requiredNeed of ["build-cli", "smoke-release-packages", "release-preflight"]) {
    assert.ok(releaseNeeds.includes(requiredNeed), requiredNeed);
  }

  const registryPublishJobs = publicationJobNames(jobs).filter(
    (jobName) => jobName !== "create-github-release",
  );
  assert.deepEqual(
    registryPublishJobs.filter((jobName) => !releaseNeeds.includes(jobName)),
    [],
    "GitHub Release must wait for every registry publish job",
  );

  assert.match(JSON.stringify(releaseJob), /softprops\/action-gh-release@/);
});

test("release workflow requires VS Code Marketplace publication", () => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const publishJob = workflowJobBody(workflow, "release-vscode-extension");
  const parsed = parse(workflow) as {
    env?: Record<string, string | number>;
    jobs?: Record<string, ReleaseJob>;
  };
  const publishJobContract = parsed.jobs?.["release-vscode-extension"];

  assert.ok(publishJobContract);
  assert.equal(publishJobContract["timeout-minutes"], 30);
  assert.equal(publishJobContract.env?.PUBLISH_RESOLUTION_RETRY_LIMIT, "90");
  assert.equal(publishJobContract.env?.VSCE_DLX_BIN, "npx");
  assert.equal(parsed.env?.PUBLISH_RESOLUTION_RETRY_DELAY, 10);

  assert.match(publishJob, /environment:\s*vscode-marketplace/);
  assert.match(publishJob, /VSCE_PAT:\s*"?\$\{\{ secrets\.VSCE_PAT \}\}"?/);
  assert.match(publishJob, /name:\s*Require VS Code Marketplace credentials/);
  assert.match(publishJob, /if \[ -z "\$\{VSCE_PAT:-\}" \]/);
  assert.match(publishJob, /VSCE_PAT is required in the protected vscode-marketplace environment/);
  assert.match(publishJob, /name:\s*Publish VS Code extension/);
  assert.match(publishJob, /node tools\/support\/release\/publish_vscode_extension\/main\.ts /);
  assert.doesNotMatch(publishJob, /Skip publish|continue-on-error|if:\s*env\.VSCE_PAT/);
});

test("release npm publication waits long enough for registry dist-tag visibility", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "release.yml")) as {
    env?: Record<string, string | number>;
  };

  assert.equal(workflow.env?.PUBLISH_RESOLUTION_RETRY_LIMIT, 60);
  assert.equal(workflow.env?.PUBLISH_RESOLUTION_RETRY_DELAY, 10);
});

test("Open VSX publication is an explicit, fail-closed opt-in", () => {
  const workflow = readRepoFile(".github", "workflows", "release-open-vsx.yml");
  const parsed = parse(workflow) as {
    env?: Record<string, string | number>;
  };
  const publishJob = workflowJobBody(workflow, "release-open-vsx-extension");

  assert.match(workflow, /name:\s*Publish Open VSX \(optional\)/);
  assert.equal(parsed.env?.PNPM_CONFIG_STRICT_DEP_BUILDS, "false");
  assert.match(workflow, /group:\s*publish-open-vsx-\$\{\{ inputs\.tag_name \}\}/);
  assert.match(workflow, /cancel-in-progress:\s*false/);
  assert.doesNotMatch(workflow, /\n\s*release:\s*\n\s*types:\s*\[published\]/);
  assert.match(
    workflow,
    /workflow_dispatch:\s*\n\s*inputs:\s*\n\s*tag_name:\s*\n\s*description:\s*Published GitHub Release tag to publish to Open VSX[\s\S]*required:\s*true[\s\S]*type:\s*string/,
  );
  assert.match(publishJob, /environment:\s*open-vsx-registry/);
  assert.match(publishJob, /OVSX_PAT:\s*\$\{\{ secrets\.OVSX_PAT \}\}/);
  assert.match(publishJob, /name:\s*Resolve release tag/);
  assert.match(publishJob, /RELEASE_TAG_NAME:\s*\$\{\{ inputs\.tag_name \}\}/);
  assert.match(publishJob, /gh release view "\$RELEASE_TAG_NAME" --repo "\$GITHUB_REPOSITORY"/);
  assert.match(publishJob, /--json isDraft,tagName/);
  assert.match(publishJob, /select\(\.isDraft == false\)/);
  assert.match(publishJob, /tag_name=%s\\n/);
  assert.match(publishJob, /name:\s*Require Open VSX credentials/);
  assert.match(publishJob, /if \[ -z "\$\{OVSX_PAT:-\}" \]/);
  assert.match(publishJob, /OVSX_PAT is required in the protected open-vsx-registry environment/);
  assert.match(publishJob, /ref:\s*\$\{\{ steps\.release\.outputs\.tag_name \}\}/);
  assert.match(publishJob, /persist-credentials:\s*false/);
  assert.match(publishJob, /name:\s*Download VSIX from GitHub Release/);
  assert.match(publishJob, /gh release download "\$\{\{ steps\.release\.outputs\.tag_name \}\}"/);
  assert.match(publishJob, /--pattern "\*\.vsix"/);
  assert.match(publishJob, /test "\$\{#vsix_files\[@\]\}" -eq 1/);
  assert.match(publishJob, /node tools\/support\/release\/publish_open_vsx_extension\/main\.ts /);
  assert.doesNotMatch(publishJob, /Skip publish|continue-on-error|if:\s*env\.OVSX_PAT/);
});
