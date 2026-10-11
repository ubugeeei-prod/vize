import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { verifyGithub } from "../public_acceptance/github.ts";

export const repository = "ubugeeei-prod/vize";
export const apiRoot = `https://api.github.com/repos/${repository}`;
export const fixedMerge = "fd20178fa311bcd5d78d1f0241a671d7d1753185";
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const gitBlob = (bytes) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
export const overrides = [
  "NODE_OPTIONS",
  "VIZE_BINDING_PATH",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "LD_PRELOAD",
  "DYLD_INSERT_LIBRARIES",
  "VIZE_PUBLIC_NATIVE_CUSTODY",
  "VIZE_OXLINT_NATIVE_CUSTODY",
];

export function optionsFrom(env) {
  for (const key of overrides) assert(!env[key], `runtime override must be absent: ${key}`);
  const options = {
    cut: env.PUBLIC_CUT,
    head: env.PUBLIC_HEAD,
    tag: env.PUBLIC_TAG,
    sourcePr: env.PUBLIC_SOURCE_PR,
    run: env.PUBLIC_RELEASE_RUN,
  };
  for (const key of ["cut", "head"]) assert.match(options[key] ?? "", /^[0-9a-f]{40}$/);
  assert.match(options.tag ?? "", /^v0\.(0|[1-9][0-9]*)\.0$/);
  for (const key of ["sourcePr", "run"]) {
    assert.match(options[key] ?? "", /^[1-9][0-9]*$/);
    assert(Number.isSafeInteger(Number(options[key])));
  }
  assert.notEqual(options.cut, options.head);
  return options;
}

export function githubReader(output, token = process.env.GH_TOKEN) {
  fs.mkdirSync(path.join(output, "github"), { recursive: true });
  let sequence = 0;
  return async (url) => {
    assert(url.startsWith(`${apiRoot}/`), "own repository API required");
    const response = await fetch(url, {
      headers: {
        accept: "application/vnd.github+json",
        ...(token ? { authorization: `Bearer ${token}` } : {}),
      },
      signal: AbortSignal.timeout(60_000),
    });
    assert(response.ok, `GitHub read failed: HTTP ${response.status}`);
    const bytes = Buffer.from(await response.arrayBuffer());
    assert(
      bytes.length > 0 && bytes.length <= 32 * 1024 * 1024,
      "bounded GitHub response required",
    );
    fs.writeFileSync(
      path.join(
        output,
        "github",
        `${String(sequence++).padStart(3, "0")}-${encodeURIComponent(url.slice(apiRoot.length + 1))}.json`,
      ),
      bytes,
      { flag: "wx" },
    );
    return JSON.parse(bytes.toString("utf8"));
  };
}

export async function authenticate(options, json) {
  const official = await verifyGithub(options, json);
  const commit = await json(`${apiRoot}/git/commits/${options.head}`);
  assert.equal(commit.sha, options.head);
  assert.deepEqual(
    commit.parents.map((parent) => parent.sha),
    [options.cut],
    "actual frozen H must directly version C",
  );
  const fix = await json(`${apiRoot}/pulls/8565`);
  assert.equal(fix.merged, true);
  assert.equal(fix.merge_commit_sha, fixedMerge, "actual typed-default correction merge required");
  assert.equal(fix.base.repo.full_name, repository);
  assert.equal(fix.head.repo.full_name, repository);
  const ancestry = await json(`${apiRoot}/compare/${fixedMerge}...${options.head}`);
  assert.equal(ancestry.base_commit.sha, fixedMerge);
  assert.equal(
    ancestry.merge_base_commit.sha,
    fixedMerge,
    "released source must contain the actual fix",
  );
  assert(["ahead", "identical"].includes(ancestry.status));
  return { ...official, commit, fix, ancestry };
}

export function consumerPaths(env, checkout) {
  assert(env.PUB_CONSUMER && env.PUBLIC_OUTPUT, "exclusive consumer and evidence paths required");
  const canonicalChild = (value) =>
    path.join(fs.realpathSync(path.dirname(path.resolve(value))), path.basename(value));
  const consumer = canonicalChild(env.PUB_CONSUMER);
  const output = canonicalChild(env.PUBLIC_OUTPUT);
  const runner = fs.realpathSync(env.RUNNER_TEMP ?? "");
  checkout = fs.realpathSync(checkout);
  const inside = (parent, child) => child.startsWith(`${parent}${path.sep}`);
  assert(
    inside(runner, consumer) && inside(runner, output),
    "consumer and evidence must be under RUNNER_TEMP",
  );
  assert(
    !inside(checkout, consumer) && !inside(checkout, output),
    "public consumer and packets must be outside checkout",
  );
  assert(!inside(consumer, output) && !inside(output, consumer) && consumer !== output);
  assert(!fs.existsSync(consumer) && !fs.existsSync(output), "fresh public paths required");
  return { consumer, output };
}
