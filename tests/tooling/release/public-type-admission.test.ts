import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import {
  buildEligibility,
  type Api,
} from "../../../tools/support/release/type-admission-eligibility.ts";

const required = "83e3ff318fb772340663b5688624dac2ad37e2b1";
const candidate = "9".repeat(40);
const syntheticVersion = "0.999.0";
const proofName = "Prove the first containing type-admission publication";
const observerName = "Qualify independent published template type admission";
type Step = {
  name?: string;
  if?: string;
  run?: string;
  env?: Record<string, string>;
  with?: { path?: string };
};
type Job = { if?: string; steps: Step[] };
type Workflow = {
  on: { workflow_dispatch: { inputs: Record<string, unknown> } };
  jobs: Record<string, Job>;
};
const workflows = ["public-native-events.yml", "release-contracts.yml"].map(
  (name) =>
    parse(
      readFileSync(new URL("../../../.github/workflows/" + name, import.meta.url), "utf8"),
    ) as Workflow,
);
function job(workflow: Workflow, name: string): Job {
  const value = workflow.jobs[name];
  assert.ok(value, name);
  return value;
}
function comparison(sha: string, contains = false) {
  return {
    base_commit: { sha: required },
    merge_base_commit: { sha: contains ? required : "0".repeat(40) },
    behind_by: contains ? 0 : 1,
    ahead_by: 7,
    status: contains ? "ahead" : "diverged",
    html_url: "https://github.com/ubugeeei-prod/vize/compare/" + required + "..." + sha,
    independentUnknownField: { retained: true },
  };
}
function fixture(priorContains = false) {
  const records = new Map<string, unknown>();
  const releases = [
    { tag_name: "v0.602.0", draft: false, published_at: "2026-01-01T00:00:00Z" },
    { tag_name: "v0.603.0", draft: false, published_at: "2026-01-02T00:00:00Z" },
    { tag_name: "v0.998.0", draft: false, published_at: "2026-01-04T00:00:00Z" },
    { tag_name: "v0.999.0", draft: false, published_at: "2026-01-05T00:00:00Z" },
  ];
  const tags = releases.map((release, index) => ({
    name: release.tag_name,
    commit: { sha: index === 3 ? candidate : String(index + 2).repeat(40) },
    node_id: "synthetic-tag-" + index,
  }));
  records.set("/pulls/10950", {
    number: 10950,
    merge_commit_sha: required,
    merged_at: "2026-01-03T00:00:00Z",
  });
  for (const release of releases) records.set("/releases/tags/" + release.tag_name, release);
  records.set("/tags?per_page=100&page=1", tags);
  records.set("/graphql", {
    data: {
      repository: {
        releases: {
          nodes: releases.map((release) => ({
            tagName: release.tag_name,
            isDraft: false,
            publishedAt: release.published_at,
          })),
          pageInfo: { hasNextPage: false, endCursor: null },
        },
      },
    },
  });
  for (const [index, tag] of tags.entries())
    records.set(
      "/compare/" + required + "..." + tag.commit.sha,
      comparison(tag.commit.sha, index === 3 || (index === 2 && priorContains)),
    );
  const calls: string[] = [];
  const api: Api = async (endpoint) => {
    calls.push(endpoint);
    assert.ok(records.has(endpoint), endpoint);
    return structuredClone(records.get(endpoint));
  };
  return { records, releases, tags, calls, api };
}

test("eligibility retains whole public source, tag and publication evidence", async () => {
  const f = fixture();
  const result = await buildEligibility(syntheticVersion, candidate, f.api);
  assert.equal(result.firstContainingTag, "v0.999.0");
  assert.equal(result.requiredSource, required);
  assert.deepEqual(result.tag, f.tags[3]);
  assert.deepEqual(result.firstContainingPublication, f.releases[3]);
  assert.deepEqual(result.sourceComparison, comparison(candidate, true));
  assert.equal(result.priorPublicationChecks.length, 3);
  assert.deepEqual(
    result.priorPublicationChecks.map((item) => item.tag),
    f.tags.slice(0, 3),
  );
});

test("historical, malformed and incompatible requests reject before SDK or network access", async () => {
  for (const [version, target, phase, source] of [
    ["0.602.0", candidate, "after", candidate],
    ["0.603.0", candidate, "after", candidate],
    ["0.601.0", candidate, "after", candidate],
    [syntheticVersion, "", "after", ""],
    [syntheticVersion, candidate, "before", candidate],
    [syntheticVersion, candidate, "release-contracts", candidate],
    [syntheticVersion, candidate, "after", "0".repeat(40)],
  ]) {
    assert.ok(
      version !== undefined && target !== undefined && phase !== undefined && source !== undefined,
    );
    const f = fixture();
    await assert.rejects(buildEligibility(version, target, f.api, phase, source));
    assert.deepEqual(f.calls, []);
  }
});

test("an earlier containing publication, incomplete listing, or mismatched tag cannot qualify", async () => {
  await assert.rejects(buildEligibility(syntheticVersion, candidate, fixture(true).api));
  for (const mode of [
    "draft",
    "wrong-tag",
    "wrong-merge-base",
    "missing-target",
    "graphql-error",
    "missing-prior-tag",
  ]) {
    const f = fixture();
    if (mode === "draft")
      f.records.set("/releases/tags/v0.999.0", { ...f.releases[3], draft: true });
    if (mode === "wrong-tag")
      f.records.set(
        "/tags?per_page=100&page=1",
        f.tags.map((tag) =>
          tag.name === "v0.999.0" ? { ...tag, commit: { sha: "a".repeat(40) } } : tag,
        ),
      );
    if (mode === "wrong-merge-base")
      f.records.set("/compare/" + required + "..." + candidate, comparison(candidate, false));
    if (mode === "missing-target")
      f.records.set("/graphql", {
        data: { repository: { releases: { nodes: [], pageInfo: { hasNextPage: false } } } },
      });
    if (mode === "graphql-error")
      f.records.set("/graphql", { errors: [{ message: "incomplete" }] });
    if (mode === "missing-prior-tag") f.records.set("/tags?per_page=100&page=1", f.tags.slice(1));
    await assert.rejects(buildEligibility(syntheticVersion, candidate, f.api), mode);
  }
});

test("all four observer flag combinations preserve 602 and select explicit type admission", () => {
  for (const workflow of workflows) {
    assert.equal(
      (workflow.on.workflow_dispatch.inputs.type_admission as { default: boolean }).default,
      false,
    );
    const standard = job(workflow, "public-release-contracts");
    const cpp = job(workflow, "public-custom-properties");
    for (const additional of [false, true])
      for (const admission of [false, true])
        for (const custom of [false, true])
          for (const phase of ["before", "after", "release-contracts", ""])
            for (const value of ["", candidate])
              for (const repository of ["ubugeeei-prod/vize", "example/fork"])
                for (const ref of ["refs/heads/main", "refs/heads/topic"]) {
                  const context = {
                    github: { repository, ref },
                    inputs: {
                      phase,
                      candidate: value,
                      custom_properties: custom,
                      additional_observers: additional,
                      type_admission: admission,
                    },
                  };
                  const main = repository === "ubugeeei-prod/vize" && ref === "refs/heads/main";
                  assert.equal(
                    runInNewContext(standard.if ?? "", context),
                    main &&
                      (admission ||
                        phase === "release-contracts" ||
                        (phase === "after" && (value !== "" || additional))),
                  );
                  assert.equal(
                    runInNewContext(cpp.if ?? "", context),
                    main && (admission || (phase === "after" && custom)),
                  );
                }
    const proof = cpp.steps.findIndex((step) => step.name === proofName);
    const original = cpp.steps.findIndex(
      (step) =>
        step.name === "Qualify fresh public custom properties and authored directive ownership",
    );
    const observer = cpp.steps.findIndex((step) => step.name === observerName);
    assert.ok(proof >= 0 && proof < original && observer > original);
    assert.equal(cpp.steps[proof]?.env?.GH_TOKEN, "${{ github.token }}");
    assert.equal(cpp.steps[observer]?.env?.GH_TOKEN, undefined);
    assert.equal(cpp.steps[proof]?.if, "inputs.type_admission == true");
    assert.equal(cpp.steps[observer]?.if, "inputs.type_admission == true");
    assert.ok(
      cpp.steps.some((step) =>
        step.with?.path?.includes("public-custom-properties-consumer/type-admission/"),
      ),
    );
  }
});

test("the actual verifier command removes credentials while retaining runtime override detection", () => {
  for (const workflow of workflows) {
    const observer = job(workflow, "public-custom-properties").steps.find(
      (step) => step.name === observerName,
    );
    assert.ok(observer?.run);
    const tokens = observer.run.trim().split(/\s+/);
    const nodeAt = tokens.indexOf("node");
    assert.ok(nodeAt > 0);
    assert.deepEqual(tokens.slice(nodeAt + 1), [
      "tools/support/release/public_type_admission/run.ts",
      '"$RUNNER_TEMP/public-custom-properties-consumer"',
      '"$PUBLIC_VERSION"',
      '"$RUNNER_TEMP/public-custom-properties-consumer/type-admission"',
      '"$RUNNER_TEMP/public-type-admission-eligibility.json"',
    ]);
    const probe = spawnSync(
      tokens[0] ?? "",
      [
        ...tokens.slice(1, nodeAt),
        process.execPath,
        "-e",
        "process.stdout.write(JSON.stringify([process.env.GH_TOKEN,process.env.GITHUB_TOKEN,process.env.NODE_AUTH_TOKEN,process.env.NPM_TOKEN,process.env.CORSA_PATH]))",
      ],
      {
        encoding: "utf8",
        env: {
          PATH: process.env.PATH,
          GH_TOKEN: "inert-fixture",
          GITHUB_TOKEN: "inert-fixture",
          NODE_AUTH_TOKEN: "inert-fixture",
          NPM_TOKEN: "inert-fixture",
          CORSA_PATH: "must-remain-visible",
        },
      },
    );
    assert.equal(probe.status, 0, probe.stderr);
    assert.deepEqual(JSON.parse(probe.stdout) as unknown, [
      null,
      null,
      null,
      null,
      "must-remain-visible",
    ]);
  }
});
