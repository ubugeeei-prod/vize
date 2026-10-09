import assert from "node:assert/strict";
import type { TestContext } from "node:test";
import { publicReader } from "../../../tools/support/release/public_acceptance/http.ts";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { verifyPublication as verify } from "../../../tools/support/release/public_acceptance/registry.ts";
import { derivePublicationPlan } from "../../../tools/support/release/public_acceptance/plan.ts";
import { marketplaceQueryUrl } from "../../../tools/support/release/marketplace_query.ts";
import {
  marketplaceEnvelope,
  marketplaceInventory,
} from "./release-retirement-marketplace-fixtures.ts";

/** Inert registry responses and temporary Git objects; never public execution evidence. */
export const version = "0.438.0";
export const tag = `v${version}`;
export const repository = "ubugeeei-prod/vize";
export const github = `https://api.github.com/repos/${repository}`;
export const gh = (resource: string) => github + resource;
export const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");

export function sourceFixture(context: { after: (callback: () => void) => void }) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-acceptance-"));
  context.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  git("init", "-q");
  git("config", "user.name", "Release fixture");
  git("config", "user.email", "fixture@example.invalid");
  git("commit", "--allow-empty", "-qm", "source C");
  const cut = git("rev-parse", "HEAD");
  const write = (name: string, content: string) => {
    fs.mkdirSync(path.dirname(path.join(root, name)), { recursive: true });
    fs.writeFileSync(path.join(root, name), content);
  };
  const manifest = (name: string, extra = {}) => JSON.stringify({ name, version, ...extra });
  write(
    "Cargo.toml",
    `[workspace.package]\nversion = "${version}"\nrepository = "https://github.com/${repository}"\n`,
  );
  write(
    ".github/workflows/release.yml",
    "jobs:\n  publish:\n    steps:\n      - run: moon run --target native tools/moon/cmd/publish_npm_package -- npm/native --provenance\n      - run: moon run --target native tools/moon/cmd/publish_npm_package -- npm/mcp-musea --provenance\n      - run: moon run --target native tools/moon/cmd/publish_npm_package_dirs -- npm/native/npm --provenance\n    path: |\n      zed-vize-extension.tar.gz\n",
  );
  write(
    "npm/native/package.json",
    manifest("@vizejs/native", {
      optionalDependencies: {
        "@vizejs/native-darwin-arm64": "catalog:native-binaries",
        "@vizejs/native-linux-x64-gnu": "catalog:native-binaries",
      },
    }),
  );
  write("npm/mcp-musea/package.json", manifest("@vizejs/mcp-musea"));
  write(
    "pnpm-workspace.yaml",
    `catalogs:\n  native-binaries:\n    "@vizejs/native-darwin-arm64": "${version}"\n    "@vizejs/native-linux-x64-gnu": "${version}"\n`,
  );
  write(
    "tools/moon/cmd/publish_crates/main.mbt",
    'let published_crates : Array[String] = [\n  "vize_l0",\n  "vize_musea",\n]\n',
  );
  write("editors/vscode/package.json", manifest("vize", { publisher: "ubugeeei" }));
  git("add", ".");
  git("commit", "-qm", "version H");
  const head = git("rev-parse", "HEAD");
  return { root, cut, head, git, write, plan: derivePublicationPlan(root, head) };
}

export function publicFixture(context: Parameters<typeof sourceFixture>[0]) {
  const source = sourceFixture(context);
  const { plan, cut, head } = source;
  const sourcePr = 42;
  const run = 77;
  const tagOid = "a".repeat(40);
  const responses = new Map<string, unknown>();
  const calls: { url: string; init: RequestInit | undefined }[] = [];
  responses.set(gh(`/git/ref/tags/${tag}`), {
    ref: `refs/tags/${tag}`,
    object: { type: "tag", sha: tagOid },
  });
  responses.set(gh(`/git/tags/${tagOid}`), {
    sha: tagOid,
    tag,
    object: { type: "commit", sha: head },
    message: `Release ${tag}\n\nRelease-Mode: pinned\nRelease-PR: #${sourcePr}\nValidated-run: ${run}\nValidated-head: ${head}\nValidated-base: ${cut}\nIntegration-PR: #43\n`,
  });
  responses.set(gh(`/pulls/${sourcePr}`), {
    number: sourcePr,
    draft: true,
    state: "open",
    merged: false,
    merged_at: null,
    body: `<!-- vize-release-pin: immutable-v1 -->\n<!-- vize-release-pin-cut: ${cut} -->\n<!-- vize-release-pin-head: ${head} -->\n<!-- vize-release-integration: 43 -->\n`,
    base: { ref: "main", repo: { full_name: repository } },
    head: { sha: head, ref: `release/${tag}`, repo: { full_name: repository } },
  });
  responses.set(gh(`/actions/runs/${run}`), {
    id: run,
    run_attempt: 1,
    repository: { full_name: repository },
    head_repository: { full_name: repository },
    head_sha: head,
    head_branch: `release/${tag}`,
    event: "workflow_dispatch",
    display_title: `Pinned Release ${tag} PR #${sourcePr} @ ${head}`,
    path: ".github/workflows/release.yml",
    status: "completed",
    conclusion: "success",
  });
  responses.set(gh(`/actions/runs/${run}/jobs?filter=latest&per_page=100&page=1`), {
    total_count: 2,
    jobs: ["success", "skipped"].map((conclusion, index) => ({
      id: index + 1,
      name: `job fixture ${index}`,
      run_id: run,
      run_attempt: 1,
      head_sha: head,
      status: "completed",
      conclusion,
    })),
  });
  responses.set(gh(`/releases/tags/${tag}`), {
    id: 123,
    tag_name: tag,
    target_commitish: "main",
    draft: false,
    prerelease: false,
    published_at: "2026-10-09T09:00:00Z",
  });
  for (const target of plan.npm) {
    const bytes = Buffer.from(`public npm fixture ${target.name}@${target.version}`);
    const url = `https://registry.npmjs.org/${encodeURIComponent(target.name)}/-/${target.version}.tgz`;
    const integrity = `sha512-${createHash("sha512").update(bytes).digest("base64")}`;
    responses.set(
      `https://registry.npmjs.org/${encodeURIComponent(target.name)}/${target.version}`,
      { ...target, dist: { tarball: url, integrity } },
    );
    responses.set(url, bytes);
  }
  for (const target of plan.crates) {
    const bytes = Buffer.from(`public crate fixture ${target.name}@${target.version}`);
    responses.set(`https://crates.io/api/v1/crates/${target.name}/${target.version}`, {
      version: { crate: target.name, num: target.version, yanked: false, checksum: sha256(bytes) },
    });
    responses.set(
      `https://static.crates.io/crates/${target.name}/${target.name}-${target.version}.crate`,
      bytes,
    );
  }
  const assetName = "zed-vize-extension.tar.gz";
  const assetUrl = `https://github.com/${repository}/releases/download/${tag}/${assetName}`;
  const asset = Buffer.from("own GitHub Zed archive fixture");
  const release = responses.get(gh(`/releases/tags/${tag}`)) as Record<string, unknown>;
  release.assets = [assetName, assetName + ".sha256"].map((name) => ({
    name,
    state: "uploaded",
    browser_download_url: `https://github.com/${repository}/releases/download/${tag}/${name}`,
  }));
  responses.set(assetUrl, asset);
  responses.set(assetUrl + ".sha256", `${sha256(asset)}  ${assetName}\n`);
  const editor = plan.editor;
  const marketplaceUrl = marketplaceQueryUrl;
  const openVsxUrl = `https://open-vsx.org/api/${editor.publisher}/${editor.name}/${version}`;
  responses.set(
    marketplaceUrl,
    marketplaceEnvelope({
      ...marketplaceInventory(editor.publisher, editor.name),
      versions: [{ version }],
    }),
  );
  responses.set(openVsxUrl, {
    namespace: editor.publisher,
    name: editor.name,
    version,
    files: {
      download: `https://open-vsx.org/api/${editor.publisher}/${editor.name}/${version}/file/extension.vsix`,
    },
  });
  const fetch: typeof globalThis.fetch = async (input, init) => {
    const url = fixtureUrl(input);
    calls.push({ url, init });
    assert.ok(responses.has(url), `unplanned fixture request: ${url}`);
    const value = responses.get(url);
    if (value instanceof Response) return value.clone();
    const response = new Response(
      Buffer.isBuffer(value)
        ? new Uint8Array(value)
        : typeof value === "string"
          ? value
          : JSON.stringify(value),
      {
        status: 200,
        headers: { "content-type": "application/json" },
      },
    );
    Object.defineProperty(response, "url", { value: url });
    return response;
  };
  return {
    ...source,
    sourcePr,
    run,
    responses,
    calls,
    marketplaceUrl,
    openVsxUrl,
    options: { tag, cut, head, sourcePr, run, fetch },
  };
}

export async function refuse(
  fixture: ReturnType<typeof publicFixture>,
  url: string,
  fields: Record<string, unknown>,
  reason: RegExp,
) {
  for (const [key, value] of Object.entries(fields)) {
    const keys = key.split(".");
    let packet = fixture.responses.get(url) as Record<string, unknown>;
    for (const field of keys.slice(0, -1)) packet = packet[field] as Record<string, unknown>;
    const field = keys.at(-1)!;
    const original = packet[field];
    packet[field] = value;
    try {
      await assert.rejects(verify(fixture.plan, fixture.options), reason);
    } finally {
      packet[field] = original;
    }
  }
}

/** Manifest-only consumer fixture: no native module or package code is executed. */
export function consumerFixture(context: { after: (callback: () => void) => void }) {
  const previous = process.cwd();
  // Manifest-only unit owns its environment; live public override checks stay strict.
  const warningOptions = process.env.NODE_OPTIONS;
  delete process.env.NODE_OPTIONS;
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-consumer-"));
  context.after(() => {
    if (warningOptions === undefined) delete process.env.NODE_OPTIONS;
    else process.env.NODE_OPTIONS = warningOptions;
    process.chdir(previous);
    fs.rmSync(root, { recursive: true, force: true });
  });
  const names = [
    "@vizejs/native",
    "@vizejs/native-linux-x64-gnu",
    "@vizejs/vite-plugin",
    "@vizejs/vite-plugin-musea",
  ];
  const dependencies = Object.fromEntries(names.map((name) => [name, version]));
  const entry = (name: string) => ({
    version,
    resolved: `https://registry.npmjs.org/${name}/-/fixture.tgz`,
    integrity: `sha512-${Buffer.alloc(64).toString("base64")}`,
    optional: false,
  });
  const packages = Object.fromEntries(names.map((name) => ["node_modules/" + name, entry(name)]));
  const optionalLocation = "node_modules/@vizejs/native-darwin-arm64";
  packages[optionalLocation] = { ...entry("@vizejs/native-darwin-arm64"), optional: true };
  const writeLock = () =>
    fs.writeFileSync(
      path.join(root, "package-lock.json"),
      JSON.stringify({ lockfileVersion: 3, packages: { "": { dependencies }, ...packages } }),
    );
  fs.writeFileSync(
    path.join(root, "package.json"),
    JSON.stringify({ private: true, dependencies }),
  );
  for (const name of names) {
    const directory = path.join(root, "node_modules", name);
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(path.join(directory, "package.json"), JSON.stringify({ name, version }));
  }
  writeLock();
  process.chdir(root);
  return {
    names,
    packages,
    optionalLocation,
    writeLock,
    remove: (name: string) => fs.rmSync(path.join(root, "node_modules", name), { recursive: true }),
  };
}

export async function stalledBodyFixture(t: TestContext) {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  let bodyStarted = false;
  let signal: AbortSignal | null | undefined;
  const response = new Response("{}");
  Object.defineProperty(response, "json", {
    value: () => {
      bodyStarted = true;
      return new Promise(() => {});
    },
  });
  const fetch: typeof globalThis.fetch = async (_url, init) => {
    signal = init?.signal;
    return response;
  };
  const pending = publicReader(fetch).json("https://registry.npmjs.org/timeout-fixture/0.438.0");
  await new Promise<void>((resolve) => process.nextTick(resolve));
  assert.equal(bodyStarted, true);
  t.mock.timers.tick(30_000);
  await assert.rejects(pending, /timed out after 30000ms/);
  assert.equal(signal?.aborted, true);
}

export function fixtureUrl(input: Parameters<typeof globalThis.fetch>[0]): string {
  return typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
}
