import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { requiredWorkflowJobNames } from "../../../tools/support/compat/github/release-preflight-evidence.mjs";
import { fullCheckJobNames } from "../../../tools/support/compat/github/full-check-policy.ts";
import { qualifiedSourceReport } from "../../../tools/support/compat/github/canonical-corpus-selection.mjs";
import type { ReadApi, Row } from "../../../tools/support/release/qualify_check/check-state.ts";
import { qualifyIntegration } from "../../../tools/support/release/qualify_check/integration.ts";
import {
  qualifyIntegrationRelease,
  releaseJobDigest,
  type Readiness,
} from "../../../tools/support/release/qualify_check/integration-release.ts";
import {
  qualifyIntegrationProducts,
  type IntegrationProductOptions,
} from "../../../tools/support/release/qualify_check/integration-product.ts";
import {
  fixture,
  context,
  repository,
  H,
  C,
  candidate,
  base,
  tree,
  created,
  clone,
} from "./integration-fixture.ts";

const gitHash = (kind: string, body: Uint8Array) =>
  createHash("sha1").update(`${kind} ${body.length}\0`).update(body).digest("hex");
const emptyTree = gitHash("tree", Buffer.alloc(0));
const blob = Buffer.from("different actual tree entry\n");
const otherTree = gitHash(
  "tree",
  Buffer.concat([Buffer.from("100644 entry\0"), Buffer.from(gitHash("blob", blob), "hex")]),
);
function commit(treeId: string, parents: string[], extra = "", role = "head") {
  const raw = Buffer.from(
    [
      `tree ${treeId}`,
      ...parents.map((parent) => `parent ${parent}`),
      ...(extra ? [extra] : []),
      "author Protocol Control <control@example.invalid> 1 +0000",
      "committer Protocol Control <control@example.invalid> 1 +0000",
      "",
      `source-bound product qualification control ${role}`,
      "",
    ].join("\n"),
  );
  return { id: gitHash("commit", raw), raw };
}
async function readiness(jobs: Row[]): Promise<Readiness> {
  const matches = jobs.filter((job) => job.name === "Release candidate ready");
  assert.equal(matches.length, 1, "original readiness identity");
  assert.equal(matches[0].status, "completed");
  assert.equal(matches[0].conclusion, "success");
  return { ready: true, readyJob: clone(matches[0]) };
}
async function productFixture({
  legacy = false,
  candidateTree = emptyTree,
  headParents = [C],
  headExtra = "",
}: {
  legacy?: boolean;
  candidateTree?: string;
  headParents?: string[];
  headExtra?: string;
} = {}) {
  const f = fixture();
  const head = commit(emptyTree, headParents, headExtra);
  const proposed = commit(candidateTree, [base], "", "candidate");
  const substitute = <T>(value: T): T =>
    JSON.parse(
      JSON.stringify(value)
        .replaceAll(H, head.id)
        .replaceAll(candidate, proposed.id)
        .replaceAll(tree, emptyTree),
    ) as T;
  const originalResource = (value: string) =>
    value.replaceAll(head.id, H).replaceAll(proposed.id, candidate);
  const q = f.runs[0];
  q.name = `Check full @ ${H}`;
  q.opaque = { whole: [1, { value: "retained" }] };
  if (!legacy)
    q.referenced_workflows = [
      {
        path: `${repository}/.github/workflows/full-qualification.yml@${H}`,
        sha: H,
        ref: `refs/heads/release/${context.tag}`,
      },
    ];
  f.jobs.splice(
    0,
    f.jobs.length,
    ...requiredWorkflowJobNames("Check", q).map((name: string, index: number) => ({
      id: 100 + index,
      name,
      run_id: 7,
      run_attempt: 3,
      head_sha: H,
      status: "completed",
      conclusion: "success",
    })),
  );
  f.records["actions/runs/5"].created_at = new Date(created - 1_000).toISOString();
  const workflow = Buffer.from("name: Release\njobs: {}\n");
  f.records["actions/workflows/release.yml"] = {
    id: 91,
    path: ".github/workflows/release.yml",
    name: "Release",
  };
  f.records[`contents/.github/workflows/release.yml?ref=${H}`] = {
    path: ".github/workflows/release.yml",
    encoding: "base64",
    content: workflow.toString("base64"),
    sha: gitHash("blob", workflow),
  };
  (f.records[`git/commits/${candidate}`].tree as Row).sha = candidateTree;
  const author = { login: "maintainer", type: "User" };
  const run: Row = {
    id: 8,
    run_attempt: 2,
    head_sha: head.id,
    head_branch: `release/${context.tag}`,
    display_title: `Pinned Release ${context.tag} PR #42 @ ${head.id}`,
    path: ".github/workflows/release.yml",
    event: "workflow_dispatch",
    created_at: new Date(created).toISOString(),
    status: "in_progress",
    conclusion: null,
    repository: { full_name: repository },
    head_repository: { full_name: repository },
    workflow_id: 91,
    actor: author,
    triggering_actor: author,
  };
  const job = (id: number, name: string): Row => ({
    id,
    name,
    run_id: 8,
    run_attempt: 2,
    head_sha: head.id,
    status: "completed",
    conclusion: "success",
    steps: [{ name: "actual", conclusion: "success" }],
    future: { preserved: [1, { field: "opaque" }] },
  });
  const jobs = [
    job(201, "Build CLI - aarch64-apple-darwin"),
    job(202, "Release candidate ready"),
    { ...job(203, "Release preflight"), status: "in_progress", conclusion: null },
  ];
  const runs = [run];
  const reads: string[] = [];
  const raw = new Map([
    [head.id, head.raw],
    [proposed.id, proposed.raw],
  ]);
  const api: ReadApi = {
    read: async (resource) => {
      reads.push(resource);
      if (resource === "actions/runs/7") return substitute(q);
      if (resource === "actions/runs/8") return clone(run);
      return substitute(await f.options.api.read(originalResource(resource)));
    },
    pages: async (resource, collection, query) => {
      if (resource === "actions/workflows/release.yml/runs") {
        assert.equal(collection, "workflow_runs");
        assert.deepEqual(query, { event: "workflow_dispatch", head_sha: head.id });
        return clone(runs);
      }
      if (resource === "actions/runs/8/jobs") {
        assert.equal(collection, "jobs");
        assert.deepEqual(query, { filter: "latest" });
        return clone(jobs);
      }
      const originalQuery = query
        ? Object.fromEntries(
            Object.entries(query).map(([key, value]) => [key, originalResource(value)]),
          )
        : undefined;
      return substitute(await f.options.api.pages(resource, collection, originalQuery));
    },
  };
  const options = { ...f.options, candidateSha: proposed.id, api, readiness };
  const qualification = await qualifyIntegration(substitute(context), options);
  const release = await qualifyIntegrationRelease(qualification, options);
  const rawReads: string[] = [];
  const productOptions: IntegrationProductOptions = {
    ...options,
    mode: "qualify",
    readiness: async () => {
      assert.fail("product reuse must verify the retained original readiness packet");
    },
    readCommit: (id) => {
      rawReads.push(id);
      assert.ok(raw.has(id), "only the two current actual commit bodies may be read");
      return raw.get(id)!;
    },
  };
  return { f, q, head, proposed, run, runs, jobs, raw, reads, rawReads, release, productOptions };
}
type Fixture = Awaited<ReturnType<typeof productFixture>>;
const qualify = (f: Fixture) =>
  qualifyIntegrationProducts({ qualification: f.release }, f.productOptions);
const verify = (f: Fixture, product: unknown) =>
  qualifyIntegrationProducts(
    { qualification: f.release, product },
    { ...f.productOptions, mode: "verify" },
  );

test("full product reuse authenticates actual raw commits and keeps the original whole R receipt pending", async () => {
  const f = await productFixture();
  const original = clone(f.release);
  const result = await qualify(f);
  assert.equal(result.reuse, true);
  assert.deepEqual(result.product, {
    kind: "vize.release.integration-product-v1",
    qualificationSha256: releaseJobDigest(original),
    profile: "full-qualification-v2",
    checkId: 7,
    checkAttempt: 3,
    head: f.head.id,
    candidate: f.proposed.id,
    headTree: emptyTree,
    candidateTree: emptyTree,
  });
  assert.deepEqual(f.rawReads, [f.head.id, f.proposed.id]);
  assert.deepEqual(f.release, original);
  assert.equal(f.release.kind, "vize.release.integration-release-v1");
  assert.deepEqual(f.release.readyJob, f.jobs[1]);
  assert.equal(f.run.status, "in_progress");
  assert.equal(f.run.conclusion, null);
  assert.deepEqual(f.f.sleeps, []);
  f.jobs.push({ ...clone(f.jobs[0]), id: 204, name: "New downstream publication" });
  assert.deepEqual(await verify(f, result.product), result);
  assert.deepEqual(f.release, original);
});

test("legacy jobs and a different genuine raw candidate tree cannot confer product reuse", async () => {
  for (const options of [{ legacy: true }, { candidateTree: otherTree }]) {
    const f = await productFixture(options);
    assert.deepEqual(await qualify(f), { reuse: false, product: null });
    await assert.rejects(verify(f, null));
    assert.equal(f.run.status, "in_progress");
  }
});

test("raw object hash, exact tree header and complete parent vector must be authentic", async () => {
  const badHash = await productFixture();
  badHash.raw.set(badHash.head.id, Buffer.concat([badHash.head.raw, Buffer.from("changed")]));
  await assert.rejects(qualify(badHash), /raw Git commit object identity changed/);
  for (const options of [
    { headParents: [C, C] },
    { headParents: [base] },
    { headExtra: `tree ${otherTree}` },
  ]) {
    const f = await productFixture(options);
    await assert.rejects(qualify(f), /raw (?:Git parent vector changed|tree required)/);
  }
  const apiTree = await productFixture();
  (apiTree.f.records[`git/commits/${candidate}`].tree as Row).sha = otherTree;
  await assert.rejects(qualify(apiTree), /raw\/API Git tree changed/);
});

test("adjacent qualification requires the unchanged original release-ready receipt and exact input shape", async () => {
  const f = await productFixture();
  for (const input of [
    null,
    [],
    {},
    { qualification: f.release.qualification },
    { qualification: f.release, product: null },
    { qualification: f.release, unknown: true },
  ])
    await assert.rejects(qualifyIntegrationProducts(input, f.productOptions));
});

const invalid: Array<[string, (f: Fixture) => void]> = [
  [
    "Q source",
    (f) => {
      f.q.head_sha = C;
    },
  ],
  [
    "Q attempt",
    (f) => {
      f.q.run_attempt = 4;
    },
  ],
  [
    "Q workflow",
    (f) => {
      f.q.path = ".github/workflows/other.yml";
    },
  ],
  [
    "Q repository",
    (f) => {
      f.q.head_repository = { full_name: "outsider/fork" };
    },
  ],
  [
    "Q profile ref",
    (f) => {
      (f.q.referenced_workflows as Row[])[0].ref = "refs/heads/main";
    },
  ],
  [
    "Q profile source",
    (f) => {
      (f.q.referenced_workflows as Row[])[0].sha = C;
    },
  ],
  [
    "Q duplicate profile",
    (f) => {
      (f.q.referenced_workflows as Row[]).push(clone((f.q.referenced_workflows as Row[])[0]));
    },
  ],
  [
    "Q job failure",
    (f) => {
      f.f.jobs.at(-1)!.conclusion = "failure";
    },
  ],
  [
    "Q job source",
    (f) => {
      f.f.jobs.at(-1)!.head_sha = C;
    },
  ],
  [
    "Q duplicate job",
    (f) => {
      f.f.jobs.push(clone(f.f.jobs.at(-1)!));
    },
  ],
  [
    "R attempt",
    (f) => {
      f.run.run_attempt = 3;
    },
  ],
  [
    "R source",
    (f) => {
      f.run.head_sha = C;
    },
  ],
  [
    "R whole ready packet",
    (f) => {
      f.jobs[1].steps = [];
    },
  ],
  [
    "R opaque terminal value",
    (f) => {
      f.jobs[0].future = { preserved: ["changed"] };
    },
  ],
  [
    "pin ref",
    (f) => {
      (f.f.records[`git/ref/heads/release-pin/${context.tag}`].object as Row).sha = C;
    },
  ],
  [
    "lease ref",
    (f) => {
      (f.f.records[`git/ref/heads/release-operator/${context.tag}`].object as Row).sha = C;
    },
  ],
  [
    "source branch",
    (f) => {
      (f.f.records[`git/ref/heads/release/${context.tag}`].object as Row).sha = C;
    },
  ],
];
for (const [name, mutate] of invalid)
  test(`current product qualification refuses changed ${name}`, async () => {
    const f = await productFixture();
    mutate(f);
    await assert.rejects(qualify(f));
  });

test("every one of the eight actual full-profile product jobs is required", async () => {
  const initial = await productFixture();
  const required = fullCheckJobNames(initial.q);
  assert.equal(required.length, 8);
  for (const name of required) {
    const f = await productFixture();
    const index = f.f.jobs.findIndex((job) => job.name === name);
    assert.ok(index >= 0, name);
    f.f.jobs.splice(index, 1);
    await assert.rejects(qualify(f), name);
  }
});

test("final verification refuses footprint tampering and fresh Q attempt, references or R custody", async () => {
  const f = await productFixture();
  const result = await qualify(f);
  assert.ok(result.product);
  for (const [key, value] of Object.entries({
    kind: "foreign",
    qualificationSha256: "0".repeat(64),
    profile: "legacy",
    checkId: 8,
    checkAttempt: 4,
    head: C,
    candidate: base,
    headTree: otherTree,
    candidateTree: otherTree,
    unknown: true,
  }))
    await assert.rejects(verify(f, { ...result.product, [key]: value }), key);
  await assert.rejects(verify(f, null));
  for (const mutate of [
    (current: Fixture) => {
      current.q.run_attempt = 4;
    },
    (current: Fixture) => {
      (current.q.referenced_workflows as Row[])[0].ref = "refs/heads/main";
    },
    (current: Fixture) => {
      current.jobs[1].future = { opaque: "changed" };
    },
    (current: Fixture) => {
      (current.f.records[`git/ref/heads/release/${context.tag}`].object as Row).sha = C;
    },
  ]) {
    const current = await productFixture();
    const previous = await qualify(current);
    mutate(current);
    await assert.rejects(verify(current, previous.product));
  }
});

test("second original verification refuses Q changes during raw commit reads", async () => {
  const f = await productFixture();
  const read = f.productOptions.readCommit!;
  f.productOptions.readCommit = async (id) => {
    const result = await read(id);
    if (id === f.proposed.id) f.q.run_attempt = 4;
    return result;
  };
  await assert.rejects(qualify(f));
});

test("canonical report permits only Nuxt reuse after the real live product verifier", async () => {
  const f = await productFixture();
  const { product } = await qualify(f);
  const input = { qualification: f.release, product };
  const options = { ...f.productOptions, mode: "verify" as const };
  const needs: Record<string, Row> = {
    "pr-source-plan": { result: "success", outputs: { "canonical-corpus": "true" } },
    "canonical-corpus": { result: "success" },
    "release-integration-catalog": { result: "success" },
    "pr-rust-products": { result: "success" },
    "nuxt-module-qualification": { result: "skipped" },
    unique: { result: "success" },
  };
  const report = () => qualifiedSourceReport(needs, "merge_group", input, options);
  assert.deepEqual(await report(), {
    exitCode: 0,
    message:
      "test-report gate: all 5 needed jobs are accounted for. 4 succeeded; 1 skipped on a pull request by design: nuxt-module-qualification.",
  });
  for (const job of ["release-integration-catalog", "pr-rust-products", "canonical-corpus"])
    for (const result of ["failure", "skipped", undefined]) {
      const saved = needs[job];
      needs[job] = { result };
      await assert.rejects(report());
      needs[job] = saved;
    }
  for (const result of ["failure", "skipped", undefined]) {
    needs.unique = { result };
    if (result === undefined) await assert.rejects(report());
    else assert.equal((await report()).exitCode, 1);
  }
  needs.unique = { result: "success" };
  await assert.rejects(qualifiedSourceReport(needs, "pull_request", input, options));
  const legacy = await productFixture({ legacy: true });
  await assert.rejects(
    qualifiedSourceReport(
      needs,
      "merge_group",
      { qualification: legacy.release, product },
      { ...legacy.productOptions, mode: "verify" },
    ),
  );
  f.q.run_attempt = 4;
  await assert.rejects(report());
});
