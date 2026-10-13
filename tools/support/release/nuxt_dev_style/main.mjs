import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { consumerPaths, githubReader, sha256 } from "../typed_defaults/custody.mjs";
import { assertStableFix, assertStableOfficial } from "../typed_defaults/post_observation.mjs";
import { installPackageGraph } from "../lint_reports/package_graph.mjs";
import { authenticateNuxt, loadNuxtFixtures, reportOptions } from "./custody.mjs";
import { copyProject, projectDependencies } from "./projects.mjs";
import { replayBrowser } from "./browser.mjs";

const checkout = fs.realpathSync(fileURLToPath(new URL("../../../../", import.meta.url)));
const options = reportOptions(process.env);
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
assert.equal(process.env.GITHUB_REF, "refs/heads/main");
assert.match(process.env.GITHUB_WORKFLOW_SHA ?? "", /^[0-9a-f]{40}$/);
const driver = spawnSync("git", ["rev-parse", "HEAD"], { cwd: checkout, encoding: "utf8" });
assert.equal(driver.status, 0);
assert.equal(driver.stdout.trim(), process.env.GITHUB_WORKFLOW_SHA);
const { consumer, output } = consumerPaths(process.env, checkout);
fs.mkdirSync(consumer, { mode: 0o700 });
fs.mkdirSync(output, { mode: 0o700 });
const receipt = {
  schema: "vize.released-nuxt-dev-ssr-8631",
  version: 1,
  options,
  observer: {
    source: driver.stdout.trim(),
    run: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
  },
  startedAt: new Date().toISOString(),
  consumer,
  success: false,
};
const persist = () =>
  fs.writeFileSync(path.join(output, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
let browser;
try {
  persist();
  const json = githubReader(output);
  const authenticated = await authenticateNuxt(options, json);
  receipt.authority = authenticated;
  persist();
  const fixture = await loadNuxtFixtures(
    options.head,
    authenticated.official.commit.tree.sha,
    path.join(consumer, "source-fixtures"),
    json,
  );
  receipt.fixtureAuthorities = fixture.authorities;
  persist();
  const installations = {};
  for (const [name, version] of [
    ["baseline", "0.443.0"],
    ["published", options.tag.slice(1)],
  ]) {
    const directory = path.join(consumer, `${name}-packages`),
      evidence = path.join(output, name);
    fs.mkdirSync(directory);
    fs.mkdirSync(evidence);
    const dependencies = projectDependencies(version);
    const installation = await installPackageGraph(
      directory,
      evidence,
      dependencies,
      (name) =>
        name.startsWith("@vizejs/") ||
        name.startsWith("@vue/") ||
        name.startsWith("@nuxt/") ||
        ["vue", "nuxt", "playwright", "playwright-core"].includes(name),
    );
    for (const row of installation.records.filter((row) =>
      row.metadata.name.startsWith("@vizejs/"),
    ))
      assert.equal(row.metadata.version, version);
    const require = createRequire(path.join(directory, "package.json"));
    const native = require("@vizejs/native");
    const provider = fs.realpathSync(require.resolve("@vizejs/native-linux-x64-gnu"));
    assert(
      provider.startsWith(
        path.join(directory, "node_modules/@vizejs/native-linux-x64-gnu") + path.sep,
      ),
    );
    assert.equal(path.extname(provider), ".node");
    assert.equal(require.cache[provider]?.loaded, true);
    assert.equal(require.cache[provider]?.exports, native);
    receipt[name] = {
      version,
      directory,
      records: installation.records,
      nativeProbe: { entry: provider, sha256: sha256(fs.readFileSync(provider)), loaded: true },
    };
    installations[name] = { ...installation, directory };
    persist();
  }
  const projects = {};
  for (const variant of ["stock", "baseline", "published"])
    projects[variant] = copyProject(
      path.join(consumer, variant),
      fixture,
      installations[variant === "published" ? "published" : "baseline"].directory,
      variant,
      options.tag.slice(1),
    );
  const browserRequire = createRequire(
    path.join(installations.published.directory, "package.json"),
  );
  const { chromium } = browserRequire("playwright");
  const install = spawnSync(
    process.execPath,
    [
      path.join(path.dirname(browserRequire.resolve("playwright/package.json")), "cli.js"),
      "install",
      "--with-deps",
      "chromium",
    ],
    { cwd: installations.published.directory, timeout: 180_000, maxBuffer: 8 * 1024 * 1024 },
  );
  receipt.chromiumInstall = {
    status: install.status,
    signal: install.signal,
    error: install.error?.message ?? null,
    stdoutBase64: install.stdout?.toString("base64"),
    stderrBase64: install.stderr?.toString("base64"),
  };
  persist();
  assert.equal(install.error, undefined);
  assert.equal(install.signal, null);
  assert.equal(install.status, 0);
  const assertPackages = () => {
    installations.baseline.assertDependencies();
    installations.published.assertDependencies();
  };
  assertPackages();
  browser = await chromium.launch();
  receipt.browser = browser.version();
  receipt.observations = await replayBrowser(browser, projects, output, assertPackages);
  await browser.close();
  browser = undefined;
  const final = await authenticateNuxt(options, json);
  receipt.finalAuthority = final;
  persist();
  assertStableOfficial(authenticated.official, final.official);
  assertStableFix(authenticated, final);
  assertPackages();
  receipt.completedAt = new Date().toISOString();
  receipt.success = true;
  receipt.evidence =
    "original pinned Nuxt first dev SSR/CSS receipts, no-JavaScript complete CSS+geometry and hydration laws on actual published adapters/native, compared with stock Vue and public4430 historical control; no full Nuxt ecosystem or production qualification";
  persist();
  console.log(
    JSON.stringify({
      success: true,
      issue: 8631,
      head: options.head,
      version: options.tag.slice(1),
      receiptSha256: sha256(fs.readFileSync(path.join(output, "receipt.json"))),
    }),
  );
} catch (error) {
  receipt.failure = { name: error.name, message: error.message };
  persist();
  throw error;
} finally {
  await browser?.close();
}
