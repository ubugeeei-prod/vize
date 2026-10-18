import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assertFullQualification,
  readHandoff,
  type Commands,
} from "../../../tools/support/release/corsa_bootstrap/main.ts";
import {
  crates,
  schema,
  type Candidate,
} from "../../../tools/support/release/corsa_bootstrap/policy.ts";
import { sha256 } from "../../../tools/support/release/corsa_bootstrap/registry.ts";
import { requiredWorkflowJobNames } from "../../../tools/support/compat/github/release-preflight-evidence.mjs";

const source: Candidate = {
  repository: "ubugeeei-prod/vize",
  number: 10001,
  head: "a".repeat(40),
  cut: "b".repeat(40),
  branch: "release/v0.999.0",
  tag: "v0.999.0",
};
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
const carrier = {
  ...qualification,
  id: 99,
  path: ".github/workflows/release-corsa-bootstrap.yml",
  actor: { login: "maintainer" },
  triggering_actor: { login: "maintainer" },
};
const receipt = {
  schema,
  candidate: source,
  run: 99,
  attempt: 1,
  qualification: 77,
  owner: 7,
  ownerLogin: "maintainer",
  repositoryOwner: 42,
  published: crates.map((name, index) => ({
    name,
    version: "0.999.0",
    checksum: "c".repeat(64),
    source: source.head,
    publisher: {
      id: index + 1,
      crate: name,
      repository_owner: "ubugeeei-prod",
      repository_owner_id: 42,
      repository_name: "vize",
      workflow_filename: "release.yml",
      environment: "crates-io",
      created_at: "2026-10-18T00:00:00Z",
    },
  })),
  consumer: { mode: "sdk-only", success: true, receiptSha256: "d".repeat(64) },
};
const states = {
  rerunPending: { run_attempt: 2, status: "in_progress", conclusion: null },
  rerunFailed: { run_attempt: 2, status: "completed", conclusion: "failure" },
  sameAttemptFailed: { run_attempt: 1, status: "completed", conclusion: "failure" },
};
type Row = Record<string, unknown>;
function observer() {
  const events: string[] = [];
  const jobs = requiredWorkflowJobNames("Check", qualification).map((name: string) => ({
    name,
    run_id: 77,
    run_attempt: 1,
    head_sha: source.head,
    status: "completed",
    conclusion: "success",
  }));
  let selected: Row = structuredClone(qualification);
  let exact: Row = structuredClone(qualification);
  let liveCarrier: Row = structuredClone(carrier);
  let phase = "";
  const zip = Buffer.from("controlled whole ZIP transport, inspected through fake unzip IO");
  const changeCarrier = (at: string) => {
    if (phase === at) liveCarrier = { ...liveCarrier, ...states.rerunPending };
  };
  const io: Commands = {
    git: () => "",
    read: () => "",
    exists: () => false,
    run: (program, args) => {
      events.push(`${program} ${args.join(" ")}`);
      if (program === "gh") {
        changeCarrier("download");
        return zip;
      }
      if (args[0] === "-Z1") return Buffer.from("receipt.json\n");
      if (args[0] === "-p") return Buffer.from(JSON.stringify(receipt));
      throw new Error("Unexpected subprocess in pure IO law");
    },
    api: (resource) => {
      events.push(resource);
      if (resource.includes("/workflows/release-corsa-bootstrap.yml/"))
        return { total_count: 1, workflow_runs: [structuredClone(carrier)] };
      if (resource.includes("/artifacts?"))
        return {
          total_count: 1,
          artifacts: [
            {
              id: 9,
              name: "corsa-first-publication",
              expired: false,
              digest: `sha256:${sha256(zip)}`,
              workflow_run: { id: 99, head_sha: source.head },
            },
          ],
        };
      if (resource.includes("/77/jobs?")) {
        changeCarrier("jobs");
        return { total_count: jobs.length, jobs: structuredClone(jobs) };
      }
      if (resource.includes("/workflows/check.yml/")) {
        changeCarrier("inventory");
        return { total_count: 1, workflow_runs: [structuredClone(selected)] };
      }
      if (resource.endsWith("/77")) {
        changeCarrier("qualification");
        return structuredClone(exact);
      }
      if (resource.endsWith("/99")) return structuredClone(liveCarrier);
      if (resource === `repos/${source.repository}`) {
        changeCarrier("repository");
        return { owner: { id: 42 } };
      }
      throw new Error(`Unexpected API read: ${resource}`);
    },
  };
  return {
    io,
    events,
    jobs,
    inventory: (change: Row) => {
      selected = { ...selected, ...change };
    },
    current: (change: Row) => {
      exact = { ...exact, ...change };
    },
    rerunAt: (at: string) => {
      phase = at;
    },
  };
}

await test("unchanged full Check and handoff remain accepted by actual production helpers", () => {
  const full = observer();
  assert.equal(assertFullQualification(structuredClone(qualification), source, full.io), 77);
  const handoff = observer();
  assert.deepEqual(readHandoff(source, handoff.io), receipt);
});
for (const [name, change] of Object.entries(states)) {
  await test(`full Check refuses same-ID final inventory drift: ${name}`, () => {
    const state = observer();
    state.inventory(change);
    assert.throws(() => assertFullQualification(structuredClone(qualification), source, state.io));
  });
  await test(`full Check refuses same-ID final exact-run drift: ${name}`, () => {
    const state = observer();
    state.current(change);
    assert.throws(() => assertFullQualification(structuredClone(qualification), source, state.io));
  });
}
for (const phase of ["download", "qualification", "jobs", "inventory", "repository"]) {
  await test(`handoff refuses a newer live carrier attempt during ${phase}`, () => {
    const state = observer();
    state.rerunAt(phase);
    assert.throws(() => readHandoff(source, state.io));
  });
}
await test("handoff requires the complete real full-Check job contract", () => {
  const state = observer();
  state.jobs.pop();
  assert.throws(() => readHandoff(source, state.io));
});
