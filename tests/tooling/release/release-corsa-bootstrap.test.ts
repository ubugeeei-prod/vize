import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  assertPublisher,
  assertReceipt,
  candidate,
  crates,
  decision,
  hasPause,
  namespaceResponse,
  pauseMarker,
  schema,
  withPause,
  type Candidate,
  type Published,
} from "../../../tools/support/release/corsa_bootstrap/policy.ts";
import {
  authenticate,
  assertFullQualification,
  assertQualification,
  checkpointPath,
  policyFiles,
  preRelease,
  readCheckpoint,
  type Commands,
} from "../../../tools/support/release/corsa_bootstrap/main.ts";
import {
  assertConsumer,
  bootstrap,
  type Steps,
} from "../../../tools/support/release/corsa_bootstrap/carrier.ts";
import { sha256 } from "../../../tools/support/release/corsa_bootstrap/registry.ts";
import { requiredWorkflowJobNames } from "../../../tools/support/compat/github/release-preflight-evidence.mjs";

const source: Candidate = {
  repository: "ubugeeei-prod/vize",
  number: 10001,
  branch: "release/v0.999.0",
  head: "a".repeat(40),
  cut: "b".repeat(40),
  tag: "v0.999.0",
};
const publisher = (name: string) => ({
  id: crates.indexOf(name as (typeof crates)[number]) + 1,
  crate: name,
  repository_owner: "ubugeeei-prod",
  repository_owner_id: 42,
  repository_name: "vize",
  workflow_filename: "release.yml",
  environment: "crates-io",
  created_at: "2026-10-18T00:00:00Z",
});
const published: Published[] = crates.map((name) => ({
  name,
  version: "0.999.0",
  checksum: "c".repeat(64),
  source: source.head,
  publisher: publisher(name),
}));
const qualification = {
  id: 77,
  name: "Check",
  run_attempt: 1,
  display_title: `Check full @ ${source.head}`,
  head_sha: source.head,
  head_branch: source.branch,
  path: ".github/workflows/check.yml",
  event: "workflow_dispatch",
  status: "completed",
  conclusion: "success",
  repository: { full_name: source.repository },
};
const receipt = () => ({
  schema,
  candidate: source,
  run: 99,
  attempt: 1,
  qualification: 77,
  owner: 7,
  ownerLogin: "maintainer",
  repositoryOwner: 42,
  published,
  consumer: { mode: "sdk-only" as const, success: true as const, receiptSha256: "d".repeat(64) },
});
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

test("immutable tuple and single sticky pause reject changed H, C, PR, branch, tag and repository", () => {
  assert.deepEqual(candidate(source), source);
  const body = withPause("Original operator and Q evidence\n", source);
  assert.ok(hasPause(body, source));
  assert.equal(withPause(body, source), body);
  for (const change of [
    { head: "e".repeat(40) },
    { cut: "f".repeat(40) },
    { number: 10002 },
    { repository: "other/vize" },
    { tag: "v0.1000.0", branch: "release/v0.1000.0" },
  ])
    assert.throws(() => hasPause(body, { ...source, ...change }));
  assert.throws(() => hasPause(`${body}\n${pauseMarker(source)}`, source));
});
test("namespace presence never substitutes for authenticated first-handoff authority", () => {
  const existing = crates.map(() => "existing");
  assert.equal(decision(existing, false), "pause");
  assert.equal(decision(existing, false), "pause");
  assert.equal(decision(["missing", ...existing.slice(1)], false), "pause");
  assert.equal(decision(existing, true), "verified-handoff");
  assert.throws(() => decision(["missing", ...existing.slice(1)], true));
  assert.throws(() => decision(["unknown", ...existing.slice(1)], false));
});
test("registry absence never swallows outage, auth failure, wrong identity or malformed schema", () => {
  assert.equal(namespaceResponse(crates[0], 404, { errors: [{ detail: "not found" }] }), "missing");
  for (const status of [401, 403, 429, 500])
    assert.throws(() => namespaceResponse(crates[0], status, {}));
  assert.throws(() => namespaceResponse(crates[0], 404, {}));
  assert.throws(() => namespaceResponse(crates[0], 200, { crate: { name: crates[1] } }));
});
test("all six identities and actual server trust fields are mandatory in a handoff", () => {
  assertReceipt(receipt(), source, 99, 1);
  for (const change of [
    { repository_owner: "other" },
    { repository_owner_id: 43 },
    { repository_name: "fork" },
    { workflow_filename: "release-corsa-bootstrap.yml" },
    { environment: null },
    { crate: crates[1] },
    { id: 0 },
    { created_at: "invalid" },
  ])
    assert.throws(() => assertPublisher({ ...publisher(crates[0]), ...change }, crates[0], 42));
  for (const change of [
    { run: 100 },
    { attempt: 2 },
    { qualification: 0 },
    { owner: 0 },
    { published: published.slice(1) },
    { published: [...published].reverse() },
    { published: published.map((row) => ({ ...row, source: source.cut })) },
    { published: published.map((row) => ({ ...row, version: "0.998.0" })) },
    { consumer: { mode: "sdk-only", success: false, receiptSha256: "d".repeat(64) } },
  ])
    assert.throws(() => assertReceipt({ ...receipt(), ...change }, source, 99, 1));
});
function operator(paused = false) {
  let body = paused ? withPause("Original custody", source) : "Original custody";
  const events: string[] = [];
  const io: Commands = {
    git: (...args) => {
      const key = args.join(" ");
      if (key === "rev-parse HEAD") return source.head;
      if (key === "rev-parse HEAD^") return source.cut;
      if (key.includes(`refs/heads/${source.branch}`)) return `${source.head}\tref`;
      if (key.includes("refs/heads/main")) return `${source.cut}\tref`;
      return "";
    },
    exists: () => false,
    read: (file) =>
      file === "Cargo.toml"
        ? '[workspace.package]\nversion = "0.999.0"\n'
        : `name = "vize_${file.split("/")[1]}"\nversion.workspace = true\n`,
    run: (_program, args) => {
      events.push(args.join(" "));
      return Buffer.from("");
    },
    api: (resource, patch) => {
      if (resource.includes("release-corsa-bootstrap.yml"))
        return { workflow_runs: [], total_count: 0 };
      if (resource.includes("release.yml")) return { workflow_runs: [] };
      if (patch) {
        body = String((patch as { body: string }).body);
        events.push("pause-persisted");
      }
      return {
        number: source.number,
        state: "open",
        draft: true,
        merged: false,
        body,
        head: { sha: source.head, ref: source.branch, repo: { full_name: source.repository } },
        base: { sha: source.cut, ref: "main", repo: { full_name: source.repository } },
      };
    },
  };
  return { io, events, body: () => body };
}
test("new R pauses only after full exact-H Q and preserves original source custody", async () => {
  const state = operator();
  await assert.rejects(
    preRelease(source, ".", Date.now() + 30_000, {
      io: state.io,
      catalogue: crates,
      request: async () => json({ errors: [{ detail: "not found" }] }, 404),
    }),
    /paused before NEW-R/u,
  );
  assert.equal(state.events.length, 2);
  assert.equal(
    state.events[0],
    `tools/support/release/qualify_check/main.ts ${source.repository} ${source.branch} ${source.head} ${source.cut}`,
  );
  assert.equal(state.events[1], "pause-persisted");
  assert.ok(state.body().startsWith("Original custody"));
  assert.ok(hasPause(state.body(), source));
  assert.ok(state.events.every((event) => !event.includes("workflow run release.yml")));
});
test("partial first publication cannot lose its pause when all six names appear", async () => {
  const state = operator(true);
  await assert.rejects(
    preRelease(source, ".", Date.now() + 30_000, {
      io: state.io,
      catalogue: crates,
      request: async (url) => json({ crate: { name: url.split("/").at(-1) } }),
    }),
    /paused before NEW-R/u,
  );
  assert.equal(state.events.length, 1, "no duplicate pause marker or release dispatch");
});
test("unmarked foreign or partially claimed six names still pause before any new R", async () => {
  const state = operator();
  await assert.rejects(
    preRelease(source, ".", Date.now() + 30_000, {
      io: state.io,
      catalogue: crates,
      request: async (url) => json({ crate: { name: url.split("/").at(-1) } }),
    }),
    /paused before NEW-R/u,
  );
  assert.ok(hasPause(state.body(), source));
  assert.equal(state.events.at(-1), "pause-persisted");
});
test("carrier rejects historical tags, changed source/main, an existing R and non-full Q", () => {
  for (const replacement of [source.cut, "", "foreign"]) {
    const state = operator();
    const original = state.io.git;
    state.io.git = (...args) =>
      args.join(" ") === "rev-parse HEAD" ? replacement : original(...args);
    assert.throws(() => authenticate(source, state.io));
  }
  const state = operator();
  const original = state.io.api;
  state.io.api = (resource, body) =>
    resource.includes("release.yml") ? { workflow_runs: [{ id: 22 }] } : original(resource, body);
  assert.throws(() => authenticate(source, state.io));
  assert.equal(assertQualification(qualification, source), 77);
  for (const change of [
    { conclusion: "failure" },
    { event: "pull_request" },
    { head_sha: source.cut },
    { path: ".github/workflows/pr-check.yml" },
    { display_title: `Check iteration @ ${source.head}` },
  ])
    assert.throws(() => assertQualification({ ...qualification, ...change }, source));
});
test("bootstrap requires the whole ordinary full-Check vector, same jobs, and latest full run", () => {
  const io = operator().io;
  const jobs = requiredWorkflowJobNames("Check", qualification).map((name: string) => ({
    name,
    run_id: 77,
    run_attempt: 1,
    head_sha: source.head,
    status: "completed",
    conclusion: "success",
  }));
  const fixture = (selected = jobs, runs: unknown[] = [qualification]) => {
    io.api = (resource) =>
      resource.includes("/workflows/")
        ? { total_count: runs.length, workflow_runs: runs }
        : resource.endsWith("/77")
          ? structuredClone(qualification)
          : { total_count: selected.length, jobs: selected };
    return io;
  };
  assert.equal(assertFullQualification(qualification, source, fixture()), 77);
  assert.throws(() => assertFullQualification(qualification, source, fixture(jobs.slice(1))));
  assert.throws(() => assertFullQualification(qualification, source, fixture([...jobs, jobs[0]])));
  assert.throws(() =>
    assertFullQualification(
      qualification,
      source,
      fixture(jobs.map((job) => ({ ...job, head_sha: source.cut }))),
    ),
  );
  assert.throws(() =>
    assertFullQualification(
      qualification,
      source,
      fixture(jobs, [qualification, { ...qualification, id: 78 }]),
    ),
  );
});
test("both original existing-R returns precede the new gate and new Q dispatch", () => {
  for (const [file, resume] of [
    ["pr_pin_runs.rs", "if let Some(id) = find()? {\n        return Ok(id);\n    }"],
    [
      "pr_watch.rs",
      "if let Some(id) = find_run(repository, head, &title, root)? {\n        return Ok(Some(id));\n    }",
    ],
  ]) {
    const text = readFileSync(
      new URL(`../../../tools/support/release/${file}`, import.meta.url),
      "utf8",
    );
    const start = text.indexOf("fn find_or_dispatch(");
    const body = text.slice(start);
    const early = body.indexOf(resume),
      gate = body.indexOf("pr_ci::corsa_first_publication(");
    assert.ok(early >= 0 && early < gate && gate < body.indexOf("pr_ci::start_check("));
    assert.equal(body.split("pr_ci::corsa_first_publication(").length, 2);
  }
});
test("serial first publication authenticates partial ownership and stops at every failed barrier", async () => {
  const events: string[] = [];
  const steps: Steps = {
    owner: async () => {
      events.push("owner");
    },
    inventory: async (name) => {
      events.push(`inventory:${name}`);
      return name === crates[0] ? "existing" : "missing";
    },
    adopt: async (name) => {
      events.push(`adopt:${name}`);
    },
    publish: async (name) => {
      events.push(`publish:${name}`);
    },
    verify: async (name) => {
      events.push(`verified:${name}`);
      return published.find((row) => row.name === name)!;
    },
    consumer: async () => {
      events.push("consumer");
      return receipt().consumer;
    },
  };
  const result = await bootstrap(steps);
  assert.deepEqual(result.published, published);
  assert.ok(events.indexOf(`adopt:${crates[0]}`) < events.indexOf(`publish:${crates[1]}`));
  for (let index = 1; index < crates.length; index++)
    assert.ok(
      events.indexOf(`verified:${crates[index - 1]}`) < events.indexOf(`publish:${crates[index]}`),
    );
  events.length = 0;
  await assert.rejects(
    bootstrap({
      ...steps,
      verify: async (name) => {
        events.push(`verify:${name}`);
        throw new Error("checksum or registry barrier failed");
      },
    }),
  );
  assert.ok(!events.some((event) => event.startsWith("publish:") || event === "consumer"));
});
test("SDK acceptance never claims Canon or full seven-crate public acceptance", () => {
  const value = {
    sourceSha: source.head,
    version: "0.999.0",
    sdkOnlyRegistryAcceptance: true,
    publicRegistryAcceptance: false,
    sixDefaultAndDistributedCompiledAndTested: true,
    canonDefaultNativeCompiled: false,
    sourcePatches: false,
    sourceReplacement: false,
    archives: published.map((row) => ({ ...row, vcs: { git: { sha1: source.head } } })),
  };
  assertConsumer(value, source, published);
  for (const change of [
    { canonDefaultNativeCompiled: true },
    { publicRegistryAcceptance: true },
    { sourceReplacement: true },
    { archives: value.archives.slice(1) },
  ])
    assert.throws(() => assertConsumer({ ...value, ...change }, source, published));
});

test("durable authority binds original H, actual signed main tree, successful carrier and closed policy", () => {
  const next = {
    ...source,
    head: "e".repeat(40),
    cut: "f".repeat(40),
    tag: "v0.1000.0",
    branch: "release/v0.1000.0",
  };
  const original = receipt();
  const main = "1".repeat(40);
  const packet = {
    schema: "vize-corsa-completed-checkpoint-v1",
    receipt: original,
    mainCommit: main,
    releaseRun: 88,
    policy: Object.fromEntries(policyFiles.map((file) => [file, sha256(`policy:${file}`)])),
  };
  const carrier = {
    ...qualification,
    id: 99,
    run_attempt: 1,
    path: ".github/workflows/release-corsa-bootstrap.yml",
    actor: { login: "maintainer" },
    triggering_actor: { login: "maintainer" },
  };
  const fixture = () => {
    const io = operator().io;
    io.exists = () => true;
    io.read = (file) => (file === checkpointPath ? JSON.stringify(packet) : `policy:${file}`);
    io.run = (_program, args) =>
      Buffer.from(args.includes("show") ? `policy:${args.at(-1)!.slice(41)}` : "");
    io.git = (...args) => (args[0] === "rev-parse" ? "whole-tree" : "");
    io.api = (resource) => {
      if (resource.includes("/commits/"))
        return { sha: main, commit: { verification: { verified: true } } };
      if (resource.includes("/attempts/"))
        return { total_count: 1, jobs: [{ name: "bootstrap", conclusion: "success" }] };
      if (resource.endsWith("/99")) return carrier;
      if (resource.endsWith("/77")) return qualification;
      if (resource.includes("/77/jobs?")) {
        const jobs = requiredWorkflowJobNames("Check", qualification).map((name: string) => ({
          name,
          run_id: 77,
          run_attempt: 1,
          head_sha: source.head,
          status: "completed",
          conclusion: "success",
        }));
        return { total_count: jobs.length, jobs };
      }
      if (resource.includes("/workflows/check.yml/"))
        return { total_count: 1, workflow_runs: [qualification] };
      if (resource.endsWith("/88"))
        return {
          ...qualification,
          path: ".github/workflows/release.yml",
          display_title: `Pinned Release ${source.tag} PR #${source.number} @ ${source.head}`,
        };
      return { owner: { id: 42 } };
    };
    return io;
  };
  assert.deepEqual(readCheckpoint(next, fixture()), original);
  const unsigned = fixture(),
    base = unsigned.api;
  unsigned.api = (resource) =>
    resource.includes("/commits/")
      ? { sha: main, commit: { verification: { verified: false } } }
      : base(resource);
  assert.throws(() => readCheckpoint(next, unsigned));
  const changed = fixture(),
    read = changed.read;
  changed.read = (file) => (file === policyFiles[0] ? "changed policy" : read(file));
  assert.throws(() => readCheckpoint(next, changed));
  const unrelated = fixture();
  unrelated.git = (...args) => {
    if (args[0] === "merge-base") throw new Error("M is not an ancestor of C");
    return "whole-tree";
  };
  assert.throws(() => readCheckpoint(next, unrelated));
  const wrongTree = fixture();
  wrongTree.git = (...args) => (args[1]?.startsWith(main) ? "other-tree" : "whole-tree");
  assert.throws(() => readCheckpoint(next, wrongTree));
  const skipped = fixture(),
    api = skipped.api;
  skipped.api = (resource) =>
    resource.includes("/attempts/")
      ? { total_count: 1, jobs: [{ name: "bootstrap", conclusion: "skipped" }] }
      : api(resource);
  assert.throws(() => readCheckpoint(next, skipped));
});
