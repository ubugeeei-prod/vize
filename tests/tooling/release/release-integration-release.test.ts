import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { qualifyIntegration } from "../../../tools/support/release/qualify_check/integration.ts";
import type { Row, ReadApi } from "../../../tools/support/release/qualify_check/check-state.ts";
import {
  completedReleaseJobIndex,
  qualifyIntegrationRelease,
  releaseJobDigest,
  type Readiness,
} from "../../../tools/support/release/qualify_check/integration-release.ts";
import { fixture, context, repository, H, C, created, clone } from "./integration-fixture.ts";

// This injected callback models the existing Rust adapter boundary; Rust's predicate is tested separately.
async function readiness(jobs: Row[]): Promise<Readiness> {
  const matches = jobs.filter((job) => job.name === "Release candidate ready");
  assert.ok(matches.length <= 1, "original Rust ambiguity");
  if (!matches.length || matches[0].status !== "completed") return { ready: false };
  assert.equal(matches[0].conclusion, "success", "original Rust readiness failure");
  return { ready: true, readyJob: clone(matches[0]) };
}
async function releaseFixture(legacy = false) {
  const f = fixture(legacy);
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
    sha: createHash("sha1").update(`blob ${workflow.length}\0`).update(workflow).digest("hex"),
  };
  const qualification = await qualifyIntegration(context, f.options);
  const author = { login: "maintainer", type: "User" };
  const run: Row = {
    id: 8,
    run_attempt: 2,
    head_sha: H,
    head_branch: `release/${context.tag}`,
    display_title: `Pinned Release ${context.tag} PR #42 @ ${H}`,
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
    run_id: 8,
    run_attempt: 2,
    head_sha: H,
    name,
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
  const api: ReadApi = {
    read: async (resource) => {
      if (resource === "actions/runs/8") return clone(run);
      return f.options.api.read(resource);
    },
    pages: async (resource, collection, query) => {
      if (resource === "actions/workflows/release.yml/runs") {
        assert.equal(collection, "workflow_runs");
        assert.deepEqual(query, { event: "workflow_dispatch", head_sha: H });
        return clone(runs);
      }
      if (resource === "actions/runs/8/jobs") {
        assert.equal(collection, "jobs");
        assert.deepEqual(query, { filter: "latest" });
        return clone(jobs);
      }
      return f.options.api.pages(resource, collection, query);
    },
  };
  const options = { ...f.options, api, readiness };
  return { f, qualification, run, runs, jobs, options };
}
test("R readiness preserves whole Rust packet and whole-job custody while publication is pending", async () => {
  const f = await releaseFixture();
  let submitted: Row[] = [];
  const receipt = await qualifyIntegrationRelease(f.qualification, {
    ...f.options,
    readiness: async (jobs) => {
      submitted = clone(jobs);
      return readiness(jobs);
    },
  });
  assert.deepEqual(submitted, f.jobs);
  assert.deepEqual(receipt.qualification, f.qualification);
  assert.deepEqual(receipt.readyJob, f.jobs[1]);
  assert.equal(receipt.completedJobs.length, 2);
  assert.equal(receipt.completedJobs[0].sha256, releaseJobDigest(f.jobs[0]));
  assert.ok(Buffer.byteLength(JSON.stringify(receipt)) < 16_384);
  f.jobs.push({ ...clone(f.jobs[0]), id: 204, name: "New downstream publication" });
  const verified = await qualifyIntegrationRelease(receipt, {
    ...f.options,
    mode: "verify",
    readiness: async () => {
      assert.fail("final verification must never invoke Rust");
    },
  });
  assert.deepEqual(verified, receipt);
  assert.deepEqual(f.f.sleeps, []);
});
test("whole packet digest ignores object insertion order and preserves unknown nested values", () => {
  assert.equal(
    releaseJobDigest({ b: { z: 2, a: 1 }, a: [2, 1] }),
    releaseJobDigest({ a: [2, 1], b: { a: 1, z: 2 } }),
  );
  assert.notEqual(releaseJobDigest({ future: { x: 1 } }), releaseJobDigest({ future: { x: 2 } }));
});
test("pending R waits within the original Q deadline and rechecks all19 Q jobs", async () => {
  const f = await releaseFixture();
  f.jobs[1].status = "queued";
  f.jobs[1].conclusion = null;
  f.f.onSleep = () => {
    f.jobs[1].status = "completed";
    f.jobs[1].conclusion = "success";
  };
  const receipt = await qualifyIntegrationRelease(f.qualification, f.options);
  assert.equal(receipt.release.id, 8);
  assert.deepEqual(f.f.pendingDeadlines, [f.f.originalDeadline]);
  assert.deepEqual(f.f.sleeps, [15_000]);
  assert.ok(f.f.calls.filter((call) => call === "PAGES actions/runs/7/jobs").length >= 4);
});
test("legacy allows already-ready R and a distinct authorized resume maintainer", async () => {
  const f = await releaseFixture(true);
  f.run.actor = f.run.triggering_actor = { login: "resumer", type: "User" };
  await qualifyIntegrationRelease(f.qualification, f.options);
  assert.deepEqual(f.f.sleeps, []);
  f.jobs[1].status = "queued";
  f.jobs[1].conclusion = null;
  await assert.rejects(qualifyIntegrationRelease(f.qualification, f.options), /legacy/);
});
test("original rerun semantics retain successful earlier-attempt jobs", async () => {
  const f = await releaseFixture();
  f.jobs[0].run_attempt = f.jobs[1].run_attempt = 1;
  const receipt = await qualifyIntegrationRelease(f.qualification, f.options);
  await qualifyIntegrationRelease(receipt, { ...f.options, mode: "verify" });
});
const invalid: [string, (f: Awaited<ReturnType<typeof releaseFixture>>) => void][] = [
  ["missing R", (f) => f.runs.splice(0)],
  [
    "missing workflow id",
    (f) => {
      delete f.run.workflow_id;
    },
  ],
  [
    "foreign workflow id",
    (f) => {
      f.run.workflow_id = 90;
    },
  ],
  [
    "workflow metadata",
    (f) => {
      f.f.records["actions/workflows/release.yml"].path = "other.yml";
    },
  ],
  [
    "source blob",
    (f) => {
      f.f.records[`contents/.github/workflows/release.yml?ref=${H}`].sha = H;
    },
  ],
  [
    "source encoding",
    (f) => {
      f.f.records[`contents/.github/workflows/release.yml?ref=${H}`].encoding = "utf8";
    },
  ],
  [
    "future R",
    (f) => {
      f.run.created_at = new Date(created + 2_000).toISOString();
    },
  ],
  [
    "R predates Q",
    (f) => {
      f.run.created_at = new Date(created - 1).toISOString();
    },
  ],
  [
    "R predates O",
    (f) => {
      f.f.records["actions/runs/5"].created_at = new Date(created + 500).toISOString();
    },
  ],
  [
    "wrong R source",
    (f) => {
      f.run.head_branch = "main";
    },
  ],
  [
    "foreign repository",
    (f) => {
      f.run.head_repository = { full_name: "outsider/fork" };
    },
  ],
  [
    "wrong O actor",
    (f) => {
      f.run.actor = { login: "resumer", type: "User" };
    },
  ],
  [
    "bot actor",
    (f) => {
      f.run.actor = { login: "maintainer", type: "Bot" };
    },
  ],
  [
    "numeric actor",
    (f) => {
      f.run.actor = { login: 42, type: "User" };
    },
  ],
  [
    "empty actor",
    (f) => {
      f.run.actor = { login: "", type: "User" };
    },
  ],
  [
    "numeric triggering actor",
    (f) => {
      f.run.triggering_actor = { login: 42, type: "User" };
    },
  ],
  [
    "empty triggering actor",
    (f) => {
      f.run.triggering_actor = { login: "", type: "User" };
    },
  ],
  [
    "string R ID",
    (f) => {
      f.run.id = "7";
    },
  ],
  [
    "failed R",
    (f) => {
      f.run.status = "completed";
      f.run.conclusion = "failure";
    },
  ],
  [
    "cancelled R",
    (f) => {
      f.run.status = "completed";
      f.run.conclusion = "cancelled";
    },
  ],
  [
    "skipped readiness",
    (f) => {
      f.jobs[1].conclusion = "skipped";
    },
  ],
  [
    "ambiguous readiness",
    (f) => {
      f.jobs.push({ ...clone(f.jobs[1]), id: 205 });
    },
  ],
  [
    "foreign job",
    (f) => {
      f.jobs[0].run_id = 9;
    },
  ],
  [
    "job source",
    (f) => {
      f.jobs[0].head_sha = C;
    },
  ],
  [
    "future attempt",
    (f) => {
      f.jobs[0].run_attempt = 3;
    },
  ],
  [
    "failed retained attempt",
    (f) => {
      f.jobs[0].run_attempt = 1;
      f.jobs[0].conclusion = "failure";
    },
  ],
  [
    "duplicate job ID",
    (f) => {
      f.jobs.push(clone(f.jobs[0]));
    },
  ],
];
for (const [name, mutate] of invalid)
  test(`R join refuses ${name}`, async () => {
    const f = await releaseFixture();
    mutate(f);
    await assert.rejects(qualifyIntegrationRelease(f.qualification, f.options));
  });
for (const mutate of [
  (f: Awaited<ReturnType<typeof releaseFixture>>) => {
    f.run.run_attempt = 3;
  },
  (f: Awaited<ReturnType<typeof releaseFixture>>) => {
    f.runs.push({ ...clone(f.run), id: 9 });
  },
  (f: Awaited<ReturnType<typeof releaseFixture>>) => {
    (f.jobs[0].future as Row).preserved = ["changed"];
  },
  (f: Awaited<ReturnType<typeof releaseFixture>>) => {
    f.jobs[1].steps = [];
  },
  (f: Awaited<ReturnType<typeof releaseFixture>>) => {
    f.jobs.splice(0, 1);
  },
])
  test("final verification refuses changed R identity or whole terminal packet", async () => {
    const f = await releaseFixture();
    const receipt = await qualifyIntegrationRelease(f.qualification, f.options);
    mutate(f);
    await assert.rejects(qualifyIntegrationRelease(receipt, { ...f.options, mode: "verify" }));
  });
test("Q failure during pending R refuses before admission", async () => {
  const f = await releaseFixture();
  f.jobs[1].status = "queued";
  f.jobs[1].conclusion = null;
  f.f.onSleep = () => {
    f.f.jobs[0].conclusion = "failure";
  };
  await assert.rejects(qualifyIntegrationRelease(f.qualification, f.options));
});
test("pending R cannot renew the original deadline", async () => {
  const f = await releaseFixture();
  f.jobs[1].status = "queued";
  f.jobs[1].conclusion = null;
  f.f.time = f.f.originalDeadline - 1;
  await assert.rejects(qualifyIntegrationRelease(f.qualification, f.options), /deadline expired/);
  assert.deepEqual(f.f.sleeps, [1]);
});
test("expired already-ready R uses no renewed transport deadline", async () => {
  const f = await releaseFixture();
  f.f.time = f.f.originalDeadline + 1;
  await qualifyIntegrationRelease(f.qualification, f.options);
  assert.deepEqual(f.f.pendingDeadlines, []);
});
test("full ready packet exceeding same-run output bound refuses", async () => {
  const f = await releaseFixture();
  f.jobs[1].large = "x".repeat(16_384);
  await assert.rejects(qualifyIntegrationRelease(f.qualification, f.options), /output bound/);
});

test("actual failed v0.462.0 R packet retains complete transport custody below the output cap", async () => {
  // https://github.com/ubugeeei-prod/vize/actions/runs/39035187768 (transport control, never green evidence).
  const bytes = fs.readFileSync(
    new URL("../../_fixtures/release/early-integration-462-release-jobs.json", import.meta.url),
  );
  assert.equal(bytes.length, 123_979);
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    "c0dea55802b112902c1ac0e4e607fbed66d5eb6cf95d6f9f6a03473ea65c06c8",
  );
  const packet = JSON.parse(bytes.toString("utf8"));
  assert.equal(packet.total_count, 50);
  assert.equal(packet.jobs.length, 50);
  for (const job of packet.jobs) {
    assert.equal(job.run_id, 39035187768);
    assert.equal(job.head_sha, "7150fbdfde4b984b7bcd4efe59f98f6b5541668d");
  }
  const index = completedReleaseJobIndex(packet.jobs);
  assert.equal(index.length, 50);
  assert.deepEqual(
    ["success", "skipped", "failure"].map(
      (status) => packet.jobs.filter((job: Row) => job.conclusion === status).length,
    ),
    [23, 26, 1],
  );
  assert.ok(Buffer.byteLength(JSON.stringify(index)) < 16_384);
  const changed = clone(packet.jobs[0]);
  changed.future = { opaque: "changed" };
  assert.notEqual(
    releaseJobDigest(changed),
    index.find((entry) => entry.id === changed.id)!.sha256,
  );
  await assert.rejects(readiness(packet.jobs), /original Rust readiness failure/);
});
