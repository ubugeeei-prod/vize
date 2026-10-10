import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  installedAuthority,
  exactPath,
  sha256,
  validateCampaignPlan,
} from "./n8n-installed-authority.ts";
import { rejectOverrides } from "../../../tools/support/release/public_acceptance/installed.ts";
import type { HtmlCampaignPlan } from "./oxlint-installed-html-types.ts";

export const sourceRoot = fileURLToPath(new URL("../../../", import.meta.url)).replace(/\/$/u, "");
const campaignPaths = [
  "tests/tooling/support/oxlint-installed-html-types.ts",
  "tests/tooling/support/oxlint-installed-html-authority.ts",
  "tests/tooling/support/oxlint-installed-html-observer.cjs",
  "tests/tooling/support/oxlint-installed-html-runtime.ts",
  "tests/tooling/support/oxlint-installed-html-packets.ts",
  "tests/tooling/support/oxlint-installed-html-stock.ts",
  "tests/tooling/support/oxlint-installed-html-plugin.ts",
  "tests/tooling/support/oxlint-installed-html-qualification.ts",
  "tests/tooling/support/oxlint-installed-html-acceptance.ts",
  "tests/tooling/support/oxlint-installed-html-acceptance.rs",
  "npm/oxlint/src/test-support/html-cli-oracles.mjs",
  "npm/oxlint/src/test-support/html-cli-oracles.d.mts",
  "npm/oxlint/src/format.ts",
  "npm/oxlint/src/cli/presentation-context.ts",
  "npm/native/index.d.ts",
  "tests/tooling/support/n8n-installed-authority.ts",
  "tests/tooling/support/n8n-installed-cli.ts",
  "tests/tooling/support/n8n-cli-config-inputs.mjs",
  "tests/tooling/support/n8n-cli-config-oracle.mjs",
  "tests/tooling/support/n8n-cli-config-workspace.mjs",
  "tests/_fixtures/n8n-cli-adoption.json",
  "tools/support/release/public_acceptance/installed.ts",
  "npm/oxlint/src/test-support/html-cli-original-packets.json",
  "npm/oxlint/src/__snapshots__/stylish-standalone-html-output.txt",
].sort();

export function installedHtmlPreflight() {
  rejectOverrides();
  assert.ok(!process.env.VIZE_OXLINT_PUBLIC_CUSTODY, "ambient public HTML observer must be absent");
}

export function wholeProcessError(error: Error): unknown {
  return {
    name: error.name,
    ...Object.fromEntries(
      Object.getOwnPropertyNames(error).map((key) => {
        const value = (error as unknown as Record<string, unknown>)[key];
        return [key, value instanceof Error ? wholeProcessError(value) : value];
      }),
    ),
  };
}

export function campaignAuthority() {
  const files = campaignPaths.map((relative) => ({
    path: relative,
    sha256: sha256(fs.readFileSync(exactPath(path.join(sourceRoot, relative), sourceRoot))),
  }));
  const corpus = path.join(
    sourceRoot,
    "tests/_fixtures/differential/lint/oxlint-original-html-operation-7903",
  );
  for (const name of fs.readdirSync(corpus).sort()) {
    const filename = exactPath(path.join(corpus, name), sourceRoot);
    files.push({
      path: path.relative(sourceRoot, filename),
      sha256: sha256(fs.readFileSync(filename)),
    });
  }
  files.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  return {
    schema: "vize.oxlint.installed-html-producer-v1",
    sha256: sha256(files.map((file) => file.path + "\0" + file.sha256 + "\n").join("")),
    files,
    scope: "reviewed campaign and unchanged original inputs; no installed execution credit",
  };
}

export function validateHtmlPlan(plan: HtmlCampaignPlan) {
  assert.equal(plan.schema, "vize.oxlint.installed-html-campaign-v1");
  validateCampaignPlan({ ...plan, schema: "vize.n8n.installed-campaign-v1" });
  assert.match(plan.campaignSha256, /^[0-9a-f]{64}$/u);
  assert.equal(plan.includedRoutes.binding.pr, 8406);
  assert.equal(plan.includedRoutes.cli.pr, 8435);
  for (const { commit } of Object.values(plan.includedRoutes))
    assert.match(commit, /^[0-9a-f]{40}$/u);
  assert.notEqual(plan.includedRoutes.binding.commit, plan.includedRoutes.cli.commit);
  assert.deepEqual(
    plan.providers.map(({ version }) => version),
    ["1.78.0", "1.86.0"],
  );
  for (const provider of plan.providers) {
    assert.match(provider.packageLockSha256, /^[0-9a-f]{64}$/u);
    assert.ok(path.isAbsolute(provider.installRoot));
  }
}

function primaryRead(endpoint: string, proofs: unknown[]) {
  const observed = spawnSync("gh", ["api", endpoint], {
    cwd: sourceRoot,
    encoding: "utf8",
    timeout: 60_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  const packet = {
    endpoint,
    status: observed.status,
    signal: observed.signal,
    error: observed.error ? wholeProcessError(observed.error) : null,
    stdout: observed.stdout,
    stderr: observed.stderr,
  };
  proofs.push(packet);
  assert.equal(observed.error, undefined, JSON.stringify(packet));
  assert.equal(observed.signal, null, JSON.stringify(packet));
  assert.equal(observed.status, 0, JSON.stringify(packet));
  return JSON.parse(observed.stdout);
}

/** Read actual commit ancestry before touching a consumer or starting its executable. */
export function requireIncludedHtmlRoutes(plan: HtmlCampaignPlan) {
  const graft = spawnSync("git", ["rev-parse", "--git-path", "info/grafts"], {
    cwd: sourceRoot,
    encoding: "utf8",
  });
  assert.equal(graft.status, 0);
  const graftPath = path.resolve(sourceRoot, graft.stdout.trim());
  assert.ok(
    !fs.existsSync(graftPath) || fs.readFileSync(graftPath).length === 0,
    "source ancestry cannot use grafts",
  );
  const proofs: unknown[] = [];
  for (const route of Object.values(plan.includedRoutes)) {
    const result = spawnSync(
      "git",
      ["--no-replace-objects", "merge-base", "--is-ancestor", route.commit, plan.source.C],
      {
        cwd: sourceRoot,
        encoding: "utf8",
      },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(
      result.status,
      0,
      `published cut must contain actual merged #${route.pr}: ${result.stderr}`,
    );
    const pull = primaryRead(`repos/ubugeeei-prod/vize/pulls/${route.pr}`, proofs);
    assert.equal(pull.number, route.pr);
    assert.ok(pull.merged_at);
    assert.equal(
      pull.merge_commit_sha,
      route.commit,
      "source-only branch heads cannot substitute for actual merged routes",
    );
    assert.equal(pull.base.repo.full_name, "ubugeeei-prod/vize");
    const commit = primaryRead(`repos/ubugeeei-prod/vize/commits/${route.commit}`, proofs);
    assert.equal(commit.sha, route.commit);
    assert.equal(commit.commit.verification.verified, true);
    assert.equal(commit.commit.verification.reason, "valid");
  }
  return proofs;
}

function publishedSource(plan: HtmlCampaignPlan) {
  const proofs: unknown[] = [];
  const source = primaryRead(`repos/ubugeeei-prod/vize/pulls/${plan.source.sourcePr}`, proofs);
  assert.equal(String(source.number), plan.source.sourcePr);
  assert.equal(source.head.sha, plan.source.H);
  const run = primaryRead(`repos/ubugeeei-prod/vize/actions/runs/${plan.source.R}`, proofs);
  assert.equal(String(run.id), plan.source.R);
  assert.equal(run.head_sha, plan.source.H);
  assert.equal(run.status, "completed");
  assert.equal(
    run.conclusion,
    "success",
    "public CLI campaign waits for terminal successful Release",
  );
  assert.equal(run.path, ".github/workflows/release.yml");
  const ref = primaryRead(`repos/ubugeeei-prod/vize/git/ref/tags/${plan.source.tag}`, proofs);
  if (ref.object.type === "tag") {
    const tag = primaryRead(`repos/ubugeeei-prod/vize/git/tags/${ref.object.sha}`, proofs);
    assert.equal(tag.tag, plan.source.tag);
    assert.equal(tag.object.type, "commit");
    assert.equal(tag.object.sha, plan.source.H);
  } else {
    assert.equal(ref.object.type, "commit");
    assert.equal(ref.object.sha, plan.source.H);
  }
  const release = primaryRead(`repos/ubugeeei-prod/vize/releases/tags/${plan.source.tag}`, proofs);
  assert.equal(release.tag_name, plan.source.tag);
  assert.equal(release.draft, false);
  assert.equal(release.prerelease, false);
  return proofs;
}

export function htmlInstalledAuthority(plan: HtmlCampaignPlan) {
  installedHtmlPreflight();
  validateHtmlPlan(plan);
  const producer = campaignAuthority();
  assert.equal(producer.sha256, plan.campaignSha256, "reviewed HTML campaign snapshot changed");
  const routeProofs = requireIncludedHtmlRoutes(plan);
  const custody = installedAuthority(
    { ...plan, schema: "vize.n8n.installed-campaign-v1" },
    sourceRoot,
  );
  const publicationProofs = publishedSource(plan);
  const plugin = custody.payload.packages.find(({ name }) => name === "oxlint-plugin-vize");
  assert.ok(
    plugin,
    "the official collector must retain the actual installed oxlint-plugin-vize payload",
  );
  assert.equal(plugin.version, custody.authority.version);
  const manifest = JSON.parse(
    fs.readFileSync(path.join(plugin.packageDirectory, "package.json"), "utf8"),
  );
  assert.equal(manifest.name, plugin.name);
  assert.equal(manifest.bin["oxlint-vize"], "bin/oxlint-vize");
  const binary = exactPath(
    path.join(plugin.packageDirectory, manifest.bin["oxlint-vize"]),
    plugin.packageDirectory,
  );
  const dist = exactPath(
    path.join(plugin.packageDirectory, "dist/cli.mjs"),
    plugin.packageDirectory,
  );
  return { ...custody, plan, producer, plugin, binary, dist, routeProofs, publicationProofs };
}
