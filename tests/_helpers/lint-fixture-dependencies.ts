import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { type AppConfig, syncGitFixtureWorktree } from "./apps.ts";
import { prepareSeparateLintInstallGraph } from "./lint-fixture-install-graph.ts";
import { resolveLintEnvironmentCli } from "./lint-fixture-runtime.ts";
import { prepareInstalledNpmxNuxt } from "./npmx-check-preparation.ts";
import {
  assertPinnedFixtureFiles,
  exportPinnedFixtureGitlinks,
  readPinnedFixtureSource,
  sha256,
} from "./pinned-fixture-source.ts";

const testsDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repositoryDir = path.dirname(testsDir);
const dependencyDir = path.join(testsDir, "_fixtures", "_projects", "lint-dependencies");

export interface PreparedAppFixture {
  app: AppConfig;
  root: string;
  env: NodeJS.ProcessEnv;
  verify: () => void;
}

function prepareMissingUpstreamLock(name: string, root: string, revision: string): void {
  if (fs.existsSync(path.join(root, "pnpm-lock.yaml"))) return;
  const closureDir = path.join(dependencyDir, name);
  const closure = JSON.parse(fs.readFileSync(path.join(closureDir, "source.json"), "utf8"));
  assert.equal(closure.revision, revision, "generated dependency lock must match the source pin");
  for (const [relativePath, expected] of Object.entries(closure.inputs)) {
    assert.equal(
      sha256(fs.readFileSync(path.join(root, relativePath))),
      expected,
      `generated lock input changed: ${relativePath}`,
    );
  }
  const lock = fs.readFileSync(path.join(closureDir, "pnpm-lock.yaml"));
  assert.equal(sha256(lock), closure.lockSha256, "generated dependency lock bytes changed");
  fs.writeFileSync(path.join(root, "pnpm-lock.yaml"), lock);
}

export function prepareLintFixtureDependencies(app: AppConfig): PreparedAppFixture {
  return prepareFixtureDependencies(app, "lint");
}

export function prepareCheckFixtureDependencies(app: AppConfig): PreparedAppFixture {
  return prepareFixtureDependencies(app, "check");
}

/** Both CLI commands evaluate the original configs and need their real providers. */
function prepareFixtureDependencies(app: AppConfig, suite: "lint" | "check"): PreparedAppFixture {
  const name = app.name;
  assert.ok(["naive-ui", "npmx.dev", "voicevox", "misskey"].includes(name));
  assert.ok(app[suite], `missing ${suite} fixture configuration`);
  const sourceDir = path.join(testsDir, "_fixtures", "_git", name);
  const expectedRevision = execFileSync("git", ["rev-parse", `HEAD:tests/_fixtures/_git/${name}`], {
    cwd: repositoryDir,
    encoding: "utf8",
  }).trim();
  const source = readPinnedFixtureSource(sourceDir, expectedRevision);
  const root = syncGitFixtureWorktree(name, suite);
  exportPinnedFixtureGitlinks(root, sourceDir, source);
  assertPinnedFixtureFiles(root, source);
  const installRoot = name === "npmx.dev" ? syncGitFixtureWorktree(name, `${suite}-install`) : root;
  const migration =
    name === "npmx.dev"
      ? prepareSeparateLintInstallGraph(root, installRoot, source, path.join(dependencyDir, name))
      : undefined;
  prepareMissingUpstreamLock(name, root, source.revision);
  const lockPath = path.join(root, "pnpm-lock.yaml");
  const lockSha256 = sha256(fs.readFileSync(lockPath));
  const pkg = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
  const manager = /^pnpm@([0-9]+\.[0-9]+\.[0-9]+)(?:\+sha(?:224|256|384|512)\.[a-f0-9]+)?$/.exec(
    pkg.packageManager,
  );
  assert.ok(manager, "lint fixture must pin its complete pnpm packageManager");
  const env = {
    ...process.env,
    PATH: `${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH ?? ""}`,
    npm_package_name: pkg.name,
    npm_package_version: pkg.version,
    npm_package_json: path.join(root, "package.json"),
    GIT_CEILING_DIRECTORIES: path.dirname(root),
    HUSKY: "0",
  };
  const prefix = ["env", "exec", "--node", process.versions.node, "pnpm"];
  const environmentCli = resolveLintEnvironmentCli();
  const actualManager = execFileSync(environmentCli, [...prefix, "--version"], {
    cwd: root,
    env,
    encoding: "utf8",
    timeout: 120_000,
  }).trim();
  assert.equal(actualManager, manager[1], "fixture must run its authored package-manager version");
  console.log(`[vize:${suite}-setup] ${name}@${source.revision} with ${pkg.packageManager}`);
  execFileSync(
    environmentCli,
    [...prefix, "install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
    { cwd: installRoot, env, stdio: "inherit", timeout: 900_000 },
  );
  migration?.attach();
  const nuxt =
    name === "npmx.dev" && suite === "check"
      ? prepareInstalledNpmxNuxt(root, installRoot, source, env)
      : undefined;
  // This exact authored postinstall command provides the eagerly imported
  // Electron path. Browser downloads and application generators are unrelated
  // to reading these original lint inputs and remain in their existing suites.
  if (name === "voicevox") {
    execFileSync(environmentCli, [...prefix, "exec", "install-electron"], {
      cwd: root,
      env,
      stdio: "inherit",
      timeout: 600_000,
    });
    fs.copyFileSync(path.join(root, ".env.example"), path.join(root, ".env"));
  }
  if (name === "misskey") {
    execFileSync(environmentCli, [...prefix, "--dir", "packages/i18n", "build"], {
      cwd: root,
      env,
      stdio: "inherit",
      timeout: 600_000,
    });
    fs.copyFileSync(
      path.join(root, ".config", "example.yml"),
      path.join(root, ".config", "default.yml"),
    );
  }
  const generatedInputs =
    name === "voicevox"
      ? [".env"]
      : name === "misskey"
        ? [
            ".config/default.yml",
            ...fs
              .readdirSync(path.join(root, "packages/i18n/built"), { recursive: true })
              .filter((file) => fs.statSync(path.join(root, "packages/i18n/built", file)).isFile())
              .map((file) => path.join("packages/i18n/built", file)),
          ]
        : (nuxt?.generatedInputs ?? []);
  const generatedHashes = Object.fromEntries(
    generatedInputs.map((file) => [file, sha256(fs.readFileSync(path.join(root, file)))]),
  );
  const verify = () => {
    assertPinnedFixtureFiles(root, source);
    assert.equal(sha256(fs.readFileSync(lockPath)), lockSha256, "frozen dependency lock changed");
    migration?.verify();
    nuxt?.verify();
    for (const [file, expected] of Object.entries(generatedHashes)) {
      assert.equal(
        sha256(fs.readFileSync(path.join(root, file))),
        expected,
        `authored generated config input changed: ${file}`,
      );
    }
  };
  verify();
  const relativeCwd =
    name === "misskey"
      ? path.join("packages", "frontend")
      : path.relative(sourceDir, app[suite]!.cwd);
  assert.ok(!relativeCwd.startsWith(".."), `${suite} must remain inside the authenticated fixture`);
  const requireProject = createRequire(path.join(root, relativeCwd, "package.json"));
  const viteManifest = requireProject.resolve("vite/package.json");
  const viteEntrypoint = requireProject.resolve("vite");
  const receipt = `${JSON.stringify(
    {
      name,
      suite,
      revision: source.revision,
      originalFiles: source.blobs.length,
      inventorySha256: source.inventorySha256,
      gitlinks: source.gitlinks,
      lockSha256,
      packageManager: pkg.packageManager,
      actualManager,
      environmentCli,
      environmentCliSha256: sha256(fs.readFileSync(environmentCli)),
      node: process.versions.node,
      originalFrozenClosure: name !== "npmx.dev" && name !== "naive-ui",
      generatedLock: name === "naive-ui",
      ...(migration ? { migration: migration.receipt } : {}),
      ...(nuxt ? { nuxt: nuxt.receipt } : {}),
      generatedInputs: generatedHashes,
      vite: {
        manifest: JSON.parse(fs.readFileSync(viteManifest, "utf8")),
        manifestSha256: sha256(fs.readFileSync(viteManifest)),
        entrypointSha256: sha256(fs.readFileSync(viteEntrypoint)),
      },
    },
    null,
    2,
  )}\n`;
  fs.writeFileSync(path.join(root, `.vize-${suite}-dependencies.json`), receipt);
  const artifactDir = path.join(repositoryDir, ".vize", "artifacts", `${suite}-fixtures`);
  fs.mkdirSync(artifactDir, { recursive: true });
  fs.writeFileSync(path.join(artifactDir, `${name}.json`), receipt);
  return {
    app: { ...app, [suite]: { ...app[suite]!, cwd: path.join(root, relativeCwd) } },
    root,
    env,
    verify,
  };
}
