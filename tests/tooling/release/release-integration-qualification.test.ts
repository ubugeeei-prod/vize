import assert from "node:assert/strict";
import { test } from "node:test";
import {
  qualifyIntegration,
  validateIntegrationContext,
} from "../../../tools/support/release/qualify_check/integration.ts";
import { readRepoFile } from "../support/github-workflows.ts";
import type { Row } from "../../../tools/support/release/qualify_check/check-state.ts";
import {
  fixture,
  context,
  repository,
  H,
  C,
  candidate,
  leaseSha,
  pinSha,
  created,
  clone,
} from "./integration-fixture.ts";

test("actual read-only helper authenticates whole source/pin/O and all19 before final revalidation", async () => {
  const f = fixture();
  assert.equal(f.jobs.length, 19);
  assert.equal(
    f.jobs.filter((job) => String(job.name).startsWith("cargo-semver-checks (")).length,
    12,
  );
  const receipt = await qualifyIntegration(context, f.options);
  assert.deepEqual(receipt.context, context);
  assert.equal(receipt.leaseSha, leaseSha);
  assert.equal(receipt.checkId, 7);
  assert.equal(receipt.checkAttempt, 3);
  assert.equal(receipt.operatorRunId, 5);
  assert.equal(receipt.operatorRunAttempt, 2);
  assert.equal(receipt.originalDeadline, f.originalDeadline);
  assert.equal(receipt.qualificationDeadline, f.originalDeadline);
  assert.ok(f.snapshots.length > 0);
  assert.deepEqual(f.sleeps, []);
  assert.ok(Buffer.byteLength(JSON.stringify(receipt)) < 16_384);
  assert.deepEqual(await qualifyIntegration(receipt, { ...f.options, mode: "verify" }), receipt);
  assert.equal(f.calls.filter((call) => call === "PAGES actions/runs/7/jobs").length, 2);
});

test("fresh v2 waits only under the original Budget and existing Q cap", async () => {
  const f = fixture();
  f.runs[0].status = "in_progress";
  f.runs[0].conclusion = null;
  f.onSleep = () => {
    f.runs[0].status = "completed";
    f.runs[0].conclusion = "success";
  };
  const receipt = await qualifyIntegration(context, f.options);
  assert.ok(f.calls.length <= 66, "one pending poll keeps a bounded read-only API cost");
  assert.equal(f.calls.filter((call) => call.startsWith("GET git/commits/")).length, 4);
  assert.equal(f.calls.filter((call) => call.includes("contents/.github/workflows")).length, 1);
  assert.deepEqual(f.sleeps, [15_000]);
  assert.deepEqual(f.pendingDeadlines, [f.originalDeadline]);
  assert.equal(receipt.qualificationDeadline, f.originalDeadline);
  const capped = fixture();
  capped.records[`git/commits/${leaseSha}`].message = String(
    capped.records[`git/commits/${leaseSha}`].message,
  ).replace(String(capped.originalDeadline), String(created + 500 * 60_000));
  assert.equal(
    (await qualifyIntegration(context, capped.options)).qualificationDeadline,
    created + 345 * 60_000,
  );
});

for (const legacy of [false, true])
  test(`${legacy ? "legacy" : "v2"} expired green Q gets fresh immediate all19 verification without renewed sleep`, async () => {
    const f = fixture(legacy);
    f.time = created + 400 * 60_000;
    const receipt = await qualifyIntegration(context, f.options);
    await qualifyIntegration(receipt, { ...f.options, mode: "verify" });
    assert.deepEqual(f.sleeps, []);
    assert.equal(receipt.originalDeadline, legacy ? null : f.originalDeadline);
    assert.deepEqual(f.pendingDeadlines, []);
  });

for (const legacy of [false, true])
  test(`${legacy ? "legacy/resume" : "expired v2"} pending Q refuses without waiting or dispatch`, async () => {
    const f = fixture(legacy);
    f.runs[0].status = "in_progress";
    f.runs[0].conclusion = null;
    if (!legacy) f.time = f.originalDeadline;
    await assert.rejects(
      qualifyIntegration(context, f.options),
      legacy ? /cannot wait/ : /expired/,
    );
    assert.deepEqual(f.sleeps, []);
  });

const badContexts: Array<[string, string, unknown]> = [
  ["ordinary PR", "event", "pull_request"],
  ["wrong candidate", "candidate", H],
  ["missing metadata", "metadataExact", false],
  ["missing catalog", "catalogEqual", false],
  ["unknown field", "operatorDeadline", created + 99999],
  ["source is integration", "source", 99],
  ["missing body", "integrationBody", undefined],
  ["empty body", "integrationBody", ""],
  ["non-string body", "integrationBody", 42],
];
for (const [label, key, value] of badContexts)
  test(`validated context refuses ${label} before any API read`, async () => {
    const f = fixture(),
      invalid = JSON.parse(JSON.stringify({ ...context, [key]: value }));
    assert.throws(() => validateIntegrationContext(invalid, repository, candidate));
    await assert.rejects(qualifyIntegration(invalid, f.options));
    assert.deepEqual(f.calls, []);
  });

const badAuthority: Array<[string, string, string, unknown]> = [
  ["foreign source", "pulls/42", "head.repo.full_name", "foreign/repo"],
  ["integration head", "pulls/99", "head.sha", H],
  ["nonmaintainer", "collaborators/maintainer/permission", "role_name", "write"],
  ["pin parent", `git/commits/${pinSha}`, "parents", [{ sha: C }]],
  ["pin tree", `git/commits/${pinSha}`, "tree.sha", C],
  ["pin marker", `git/commits/${pinSha}`, "message", "foreign"],
  ["H parent", `git/commits/${H}`, "parents", [{ sha: C }, { sha: C }]],
  ["candidate parent", `git/commits/${candidate}`, "parents", [{ sha: H }]],
  ["lease parent", `git/commits/${leaseSha}`, "parents", [{ sha: C }]],
  ["lease tree", `git/commits/${leaseSha}`, "tree.sha", C],
  ["duplicate lease field", `git/commits/${leaseSha}`, "append", "Operator-run: 5\n"],
  ["malformed deadline", `git/commits/${leaseSha}`, "deadline", "01"],
  ["wrong O ID", "actions/runs/5", "id", 6],
  ["wrong O attempt", "actions/runs/5", "run_attempt", 3],
  ["wrong O source", "actions/runs/5", "head_sha", H],
  ["wrong O branch", "actions/runs/5", "head_branch", "foreign"],
  ["wrong O workflow", "actions/runs/5", "workflow_id", 91],
  ["wrong O event", "actions/runs/5", "event", "push"],
  ["wrong O actor", "actions/runs/5", "triggering_actor.login", "foreign"],
  ["completed O", "actions/runs/5", "status", "completed"],
  ["missing workflow ID", "actions/workflows/release-operator.yml", "id", undefined],
  ["missing workflow name", "actions/workflows/release-operator.yml", "name", undefined],
  [
    "wrong source policy",
    `contents/.github/workflows/release-operator.yml?ref=${C}`,
    "content",
    "",
  ],
];
for (const [label, resource, pointer, value] of badAuthority)
  test(`authority refuses ${label} without Q waiting`, async () => {
    const f = fixture();
    if (pointer === "append") f.records[resource].message += String(value);
    else if (pointer === "deadline")
      f.records[resource].message = String(f.records[resource].message).replace(
        String(f.originalDeadline),
        String(value),
      );
    else {
      const keys = pointer.split(".");
      let target = f.records[resource];
      for (const key of keys.slice(0, -1)) target = target[key] as Row;
      target[keys.at(-1)!] = value;
    }
    await assert.rejects(qualifyIntegration(context, f.options));
    assert.deepEqual(f.sleeps, []);
  });
for (const name of ["pin", "operator"])
  test(`absent ${name} refuses immediately`, async () => {
    const f = fixture();
    delete f.records[`git/ref/heads/release-${name}/${context.tag}`];
    await assert.rejects(qualifyIntegration(context, f.options));
    assert.deepEqual(f.sleeps, []);
  });

for (const field of ["head_sha", "head_branch", "path", "event", "display_title"])
  test(`Q refuses a foreign ${field}`, async () => {
    const f = fixture();
    f.runs[0][field] = "foreign";
    await assert.rejects(qualifyIntegration(context, f.options), /missing/);
    assert.deepEqual(f.sleeps, []);
  });
for (const state of ["failure", "cancelled", "skipped", "timed_out", "neutral"])
  test(`Q refuses terminal ${state}`, async () => {
    const f = fixture();
    f.runs[0].conclusion = state;
    await assert.rejects(qualifyIntegration(context, f.options));
    assert.deepEqual(f.sleeps, []);
  });
for (const [status, conclusion] of [
  ["unknown", null],
  ["queued", "success"],
])
  test(`Q refuses inconsistent ${status}/${conclusion}`, async () => {
    const f = fixture();
    f.runs[0].status = status;
    f.runs[0].conclusion = conclusion;
    await assert.rejects(qualifyIntegration(context, f.options));
    assert.deepEqual(f.sleeps, []);
  });
for (const flaw of ["missing", "duplicate", "failed", "foreign-source", "future-attempt"])
  test(`all19 validation refuses ${flaw} semver evidence`, async () => {
    const f = fixture(),
      index = f.jobs.findIndex((job) => String(job.name).startsWith("cargo-semver-checks ("));
    if (flaw === "missing") f.jobs.splice(index, 1);
    else if (flaw === "duplicate") f.jobs.push(clone(f.jobs[index]));
    else if (flaw === "failed") f.jobs[index].conclusion = "failure";
    else if (flaw === "foreign-source") f.jobs[index].head_sha = C;
    else f.jobs[index].run_attempt = 4;
    await assert.rejects(qualifyIntegration(context, f.options));
    assert.deepEqual(f.sleeps, []);
  });

test("pending failure in a required job refuses before its first sleep", async () => {
  const f = fixture();
  f.runs[0].status = "in_progress";
  f.runs[0].conclusion = null;
  f.jobs[0].conclusion = "failure";
  await assert.rejects(qualifyIntegration(context, f.options));
  assert.deepEqual(f.sleeps, []);
});
test("Q attempt/run/ref and lease changes during the wait cannot borrow earlier green proof", async () => {
  for (const change of ["attempt", "run", "ref", "lease"]) {
    const f = fixture();
    f.runs[0].status = "queued";
    f.runs[0].conclusion = null;
    f.onSleep = () => {
      f.runs[0].status = "completed";
      f.runs[0].conclusion = "success";
      if (change === "attempt") f.runs[0].run_attempt = 4;
      if (change === "run") f.runs[0].id = 8;
      if (change === "ref")
        (f.records[`git/ref/heads/release/${context.tag}`].object as Row).sha = C;
      if (change === "lease")
        (f.records[`git/ref/heads/release-operator/${context.tag}`].object as Row).sha = C;
    };
    await assert.rejects(qualifyIntegration(context, f.options), change);
    assert.deepEqual(f.sleeps, [15_000]);
  }
});
test("required final verification refuses changed latest Q or original authority and never waits", async () => {
  for (const change of ["attempt", "newer-run", "pending", "lease", "pin", "O", "head"]) {
    const f = fixture(),
      receipt = await qualifyIntegration(context, f.options);
    if (change === "attempt") f.runs[0].run_attempt = 4;
    if (change === "newer-run")
      f.runs.unshift({ ...f.runs[0], id: 8, created_at: new Date(created + 500).toISOString() });
    if (change === "pending") {
      f.runs[0].status = "queued";
      f.runs[0].conclusion = null;
    }
    if (change === "lease")
      (f.records[`git/ref/heads/release-operator/${context.tag}`].object as Row).sha = C;
    if (change === "pin")
      (f.records[`git/ref/heads/release-pin/${context.tag}`].object as Row).sha = C;
    if (change === "O") f.records["actions/runs/5"].run_attempt = 3;
    if (change === "head") (f.records["pulls/99"].head as Row).sha = C;
    await assert.rejects(qualifyIntegration(receipt, { ...f.options, mode: "verify" }), change);
    assert.deepEqual(f.sleeps, []);
  }
});
test("verify requires the original bounded catalog receipt rather than another caller context", async () => {
  const f = fixture();
  await assert.rejects(
    qualifyIntegration(context, { ...f.options, mode: "verify" }),
    /prior catalog/,
  );
  const receipt = await qualifyIntegration(context, f.options);
  await assert.rejects(
    qualifyIntegration({ ...receipt, extra: true }, { ...f.options, mode: "verify" }),
    /changed/,
  );
});

test("legacy prerelease and distinct-maintainer resume verify green Q without a new wait budget", async () => {
  const legacy = { ...context, tag: "v0.500.0-beta.2" };
  const f = fixture(true, legacy.tag, "resumer", "0.500.0-beta.1");
  legacy.integrationBody = String(f.records["pulls/99"].body);
  const receipt = await qualifyIntegration(legacy, f.options);
  assert.equal(receipt.originalDeadline, null);
  await qualifyIntegration(receipt, { ...f.options, mode: "verify" });
  assert.ok(f.calls.includes("GET collaborators/resumer/permission"));
  f.records["collaborators/resumer/permission"].role_name = "write";
  await assert.rejects(qualifyIntegration(receipt, { ...f.options, mode: "verify" }), /maintainer/);
  assert.deepEqual(f.sleeps, []);
});

test("candidate context retains the same complete body accepted by the original Rust policy", () => {
  const native = readRepoFile("tools/support/release/pr_pin_candidate.rs");
  assert.ok(native.includes('let integration_body = pr_contract::field(&pr, "/body")?;'));
  assert.ok(
    native.includes("super::body::qualify(integration_body, &canonical, &repository, number)?;"),
  );
  assert.ok(native.includes('"integrationBody":integration_body'));
  assert.ok(
    native.indexOf("super::body::qualify(integration_body") <
      native.indexOf('"integrationBody":integration_body'),
  );
});

test("realistic complete integration body survives the bounded context and Q receipt unchanged", async () => {
  const f = fixture();
  assert.ok(Buffer.byteLength(context.integrationBody) >= 2_000);
  assert.ok(Buffer.byteLength(JSON.stringify(context)) < 16_384);
  assert.equal(f.records["pulls/99"].body, context.integrationBody);
  const receipt = await qualifyIntegration(context, f.options);
  assert.equal(receipt.context.integrationBody, context.integrationBody);
  assert.ok(Buffer.byteLength(JSON.stringify(receipt)) < 16_384);
  assert.deepEqual(await qualifyIntegration(receipt, { ...f.options, mode: "verify" }), receipt);
});

for (const mutation of ["same-marker prose", "bot footer update", "remove body"]) {
  test(`post-catalog ${mutation} refuses the entire validated integration body`, async () => {
    const f = fixture(),
      receipt = await qualifyIntegration(context, f.options);
    f.records["pulls/99"].body =
      mutation === "same-marker prose"
        ? context.integrationBody.replace("never auto-merges", "immediately auto-merges")
        : mutation === "bot footer update"
          ? context.integrationBody.replace(
              /signature=([0-9a-f])/u,
              (_match, first: string) => `signature=${first === "a" ? "b" : "a"}`,
            )
          : undefined;
    assert.notEqual(f.records["pulls/99"].body, context.integrationBody);
    await assert.rejects(
      qualifyIntegration(receipt, { ...f.options, mode: "verify" }),
      /validated integration body changed/,
    );
    assert.deepEqual(f.sleeps, []);
  });
}

test("body changes during the existing Q wait refuse before reusing a later green answer", async () => {
  const f = fixture();
  f.runs[0].status = "queued";
  f.runs[0].conclusion = null;
  f.onSleep = () => {
    f.runs[0].status = "completed";
    f.runs[0].conclusion = "success";
    f.records["pulls/99"].body = context.integrationBody + "\n";
  };
  await assert.rejects(
    qualifyIntegration(context, f.options),
    /validated integration body changed/,
  );
  assert.deepEqual(f.sleeps, [15_000]);
});
