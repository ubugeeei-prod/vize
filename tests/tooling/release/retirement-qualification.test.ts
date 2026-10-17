import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { assertPreReleaseQualification } from "../../../tools/support/release/retirement_qualification.ts";
import { requiredWorkflowJobNames } from "../../../tools/support/compat/github/release-preflight-evidence.mjs";

const head = "a".repeat(40);
const cut = "b".repeat(40);
const script = fileURLToPath(
  new URL("../../../tools/support/release/retirement_qualification.ts", import.meta.url),
);
function run(id = 20) {
  return {
    id,
    run_attempt: 2,
    head_sha: head,
    head_branch: "release/v0.8.0",
    path: ".github/workflows/check.yml",
    event: "workflow_dispatch",
    display_title: `Check full @ ${head}`,
    status: "completed",
    conclusion: "failure",
    repository: { id: 10, full_name: "owner/repo" },
    head_repository: { id: 10, full_name: "owner/repo" },
    actor: { login: "maintainer" },
    triggering_actor: { login: "maintainer" },
    run_started_at: "2026-10-17T01:00:00Z",
  };
}
function job(name: string, id: number, source = run()) {
  return {
    id,
    run_id: source.id,
    run_attempt: source.run_attempt,
    head_sha: source.head_sha,
    name,
    status: "completed",
    conclusion: "success",
    steps: [] as unknown[],
  };
}
function guards() {
  const qualification = run();
  const operator = {
    ...run(19),
    head_sha: cut,
    head_branch: "main",
    path: ".github/workflows/release-operator.yml",
    conclusion: "cancelled",
  };
  const jobs = requiredWorkflowJobNames("Check", qualification).map((name: string, index: number) =>
    job(name, index + 100),
  );
  jobs.push({ ...job("check-js", 200), conclusion: "failure" });
  return {
    identity: {
      repository: "owner/repo",
      tag: "v0.8.0",
      head,
      cut,
      sourcePr: 42,
      origin: "pre-release-qualification",
      qualificationRun: 20,
      qualificationAttempt: 2,
      operatorRun: 19,
      operatorAttempt: 2,
    },
    sourcePr: { number: 42, user: { login: "maintainer" } },
    qualificationRun: qualification,
    operatorRun: operator,
    qualificationJobs: jobs,
    originalFailureJobPages: {
      qualification: [{ total_count: jobs.length, jobs }],
      operator: [{ total_count: 1, jobs: [job("operator", 300, operator)] }],
    },
    releaseRunAbsence: {
      pages: [{ total_count: 0, workflow_runs: [] as ReturnType<typeof run>[] }],
      hRunPages: [{ total_count: 1, workflow_runs: [structuredClone(qualification)] }],
    },
  };
}
function refreshJobs(value: ReturnType<typeof guards>) {
  value.originalFailureJobPages.qualification = [
    { total_count: value.qualificationJobs.length, jobs: structuredClone(value.qualificationJobs) },
  ];
}
function receipt() {
  const value = guards();
  return {
    schema: "vize-unpublished-retirement-v3",
    identity: structuredClone(value.identity),
    guards: value,
    originalEvidence: {
      HRunPages: structuredClone(value.releaseRunAbsence.hRunPages),
      observations: [
        {
          run: structuredClone(value.qualificationRun),
          jobPages: structuredClone(value.originalFailureJobPages.qualification),
        },
        {
          run: structuredClone(value.operatorRun),
          jobPages: structuredClone(value.originalFailureJobPages.operator),
        },
      ],
    },
  };
}
function rejects(change: (value: ReturnType<typeof guards>) => void, message?: RegExp) {
  const value = guards();
  change(value);
  assert.throws(() => assertPreReleaseQualification(value), message ?? /Pre-release qualification/);
}
function cli(value: unknown, args: string[] = []) {
  return spawnSync(process.execPath, [script, ...args], {
    input: JSON.stringify(value),
    encoding: "utf8",
    timeout: 10_000,
  });
}

test("whole failed current Q accepts real existing 19-name full Check policy and offline receipt", () => {
  const value = guards();
  assert.equal(requiredWorkflowJobNames("Check", value.qualificationRun).length, 19);
  assert.equal(
    value.qualificationJobs.find((row) => row.name === "check-js")?.conclusion,
    "failure",
  );
  assert.doesNotThrow(() => assertPreReleaseQualification(value));
  assert.doesNotThrow(() => assertPreReleaseQualification(receipt()));
});

test("all-attempt evidence accepts inherited terminal required jobs and a current failed rerun", () => {
  const value = guards();
  for (const row of value.qualificationJobs) if (row.name !== "check-js") row.run_attempt = 1;
  value.qualificationJobs.push({ ...job("check-js", 201), run_attempt: 1, conclusion: "failure" });
  refreshJobs(value);
  assert.doesNotThrow(() => assertPreReleaseQualification(value));
});

test("Q run must be failed or timed out with a current actual job failure", () => {
  for (const conclusion of ["success", "cancelled", "neutral", "skipped"]) {
    rejects((value) => {
      value.qualificationRun.conclusion = conclusion;
    }, /Q must be terminal/);
  }
  rejects((value) => {
    value.qualificationRun.status = "in_progress";
    value.qualificationRun.conclusion = null as unknown as string;
  }, /terminal/);
  rejects((value) => {
    for (const row of value.qualificationJobs) row.conclusion = "cancelled";
    refreshJobs(value);
  }, /current Q attempt/);
  rejects((value) => {
    value.qualificationJobs.find((row) => row.name === "check-js")!.run_attempt = 1;
    refreshJobs(value);
  }, /current Q attempt/);
  const timedOut = guards();
  timedOut.qualificationRun.conclusion = "timed_out";
  timedOut.releaseRunAbsence.hRunPages[0].workflow_runs[0].conclusion = "timed_out";
  timedOut.qualificationJobs.at(-1)!.conclusion = "timed_out";
  refreshJobs(timedOut);
  assert.doesNotThrow(() => assertPreReleaseQualification(timedOut));
});

test("reuse actual existing full title/event/ref qualifier and latest-run ordering", () => {
  for (const field of ["display_title", "event", "path", "head_branch"] as const) {
    rejects((value) => {
      value.qualificationRun[field] = "ordinary";
      value.releaseRunAbsence.hRunPages[0].workflow_runs[0][field] = "ordinary";
    });
  }
  const ignored = guards();
  ignored.releaseRunAbsence.hRunPages[0].workflow_runs.push({ ...run(21), display_title: "Check" });
  ignored.releaseRunAbsence.hRunPages[0].total_count = 2;
  assert.doesNotThrow(() => assertPreReleaseQualification(ignored));
  rejects((value) => {
    value.releaseRunAbsence.hRunPages[0].workflow_runs.push({
      ...run(21),
      run_started_at: "2026-10-17T02:00:00Z",
    });
    value.releaseRunAbsence.hRunPages[0].total_count = 2;
  }, /latest eligible/);
  rejects((value) => {
    value.releaseRunAbsence.hRunPages[0].workflow_runs[0].run_attempt = 3;
  }, /latest eligible/);
});

test("any Release-only row or unfiltered all-H Release path blocks regardless of event/status/title", () => {
  for (const event of ["push", "workflow_dispatch", "schedule"]) {
    rejects((value) => {
      value.releaseRunAbsence.hRunPages[0].workflow_runs.push({
        ...run(21),
        path: ".github/workflows/release.yml",
        event,
        status: "queued",
        conclusion: null as unknown as string,
        display_title: "unrelated",
      });
      value.releaseRunAbsence.hRunPages[0].total_count = 2;
    }, /Release run/);
  }
  rejects((value) => {
    value.releaseRunAbsence.pages = [
      { total_count: 1, workflow_runs: [{ ...run(1), path: "anything" }] },
    ];
  }, /Release-only row/);
});

test("Node CLI scans page two Release evidence in either inventory", () => {
  for (const field of ["pages", "hRunPages"] as const) {
    const value = guards();
    const first = Array.from({ length: 100 }, (_, index) => ({
      ...run(1_000 + index),
      display_title: "Check",
      event: "push",
    }));
    const last = {
      ...run(2_000),
      path: ".github/workflows/release.yml",
      event: "push",
      display_title: "other title",
      status: "queued",
      conclusion: null as unknown as string,
    };
    value.releaseRunAbsence[field] = [
      { total_count: 101, workflow_runs: first },
      { total_count: 101, workflow_runs: [last] },
    ];
    const result = cli(value);
    assert.equal(result.status, 1, result.stderr);
    assert.match(result.stderr, /Release/);
    assert.equal(result.stdout, "");
  }
});

test("complete bounded pagination refuses truncated totals, short intermediate pages and errors", () => {
  rejects((value) => {
    value.releaseRunAbsence.hRunPages[0].total_count = 2;
  }, /incomplete inventory total/);
  rejects((value) => {
    value.originalFailureJobPages.qualification[0].total_count += 1;
  }, /incomplete inventory total/);
  rejects((value) => {
    value.releaseRunAbsence.hRunPages.push({ total_count: 1, workflow_runs: [] });
  }, /intermediate page/);
  rejects((value) => {
    value.releaseRunAbsence.pages[0].total_count = 10_001;
  }, /bounded total/);
  rejects((value) => {
    value.releaseRunAbsence.pages = Array.from({ length: 101 }, () => ({
      total_count: 0,
      workflow_runs: [],
    }));
  }, /bounded array/);
  rejects((value) => {
    Object.assign(value.releaseRunAbsence.pages[0], { message: "API rate limit exceeded" });
  }, /API error/);
  rejects((value) => {
    value.releaseRunAbsence.pages = [];
  }, /complete pages/);
});

test("no duplicate, missing or unsafe row IDs are accepted", () => {
  for (const id of [0, -1, NaN, Number.MAX_SAFE_INTEGER + 1]) {
    rejects((value) => {
      value.releaseRunAbsence.hRunPages[0].workflow_runs[0].id = id;
    }, /positive safe integer/);
  }
  rejects((value) => {
    value.releaseRunAbsence.hRunPages[0].workflow_runs.push(
      structuredClone(value.qualificationRun),
    );
    value.releaseRunAbsence.hRunPages[0].total_count = 2;
  }, /duplicate ID/);
  rejects((value) => {
    value.qualificationJobs.push(structuredClone(value.qualificationJobs[0]));
    refreshJobs(value);
  }, /duplicate ID/);
  rejects((value) => {
    value.originalFailureJobPages.operator[0].jobs[0].id = value.qualificationJobs[0].id;
  }, /cross-run job ID/);
});

test("every full Check required name is required and all jobs must belong to Q/H/attempt", () => {
  for (const name of requiredWorkflowJobNames("Check", run())) {
    rejects((value) => {
      value.qualificationJobs = value.qualificationJobs.filter((row) => row.name !== name);
      refreshJobs(value);
    }, /required Check job missing/);
  }
  rejects((value) => {
    value.qualificationJobs[0].run_id = 99;
    refreshJobs(value);
  }, /run\/head\/attempt/);
  rejects((value) => {
    value.qualificationJobs[0].head_sha = cut;
    refreshJobs(value);
  }, /run\/head\/attempt/);
  rejects((value) => {
    value.qualificationJobs[0].run_attempt = 3;
    refreshJobs(value);
  }, /run\/head\/attempt/);
  rejects((value) => {
    value.qualificationJobs.push({ ...value.qualificationJobs[0], id: 999 });
    refreshJobs(value);
  }, /duplicate job name/);
  rejects((value) => {
    value.qualificationJobs[0].status = "queued";
    value.qualificationJobs[0].conclusion = null as unknown as string;
    refreshJobs(value);
  }, /terminal/);
});

test("guard job list must reconcile every raw job, including jobs outside the required 19", () => {
  rejects((value) => {
    value.qualificationJobs = value.qualificationJobs.slice(0, -1);
  }, /inventory differs/);
  rejects((value) => {
    value.originalFailureJobPages.qualification = structuredClone(
      value.originalFailureJobPages.qualification,
    );
    value.qualificationJobs.at(-1)!.name = "altered";
  }, /inventory differs/);
  rejects((value) => {
    value.qualificationJobs = [
      ...value.qualificationJobs,
      structuredClone(value.qualificationJobs[0]),
    ];
  }, /duplicate ID/);
});

test("Q identity, origin, actors, own repository and all-H source cannot drift", () => {
  rejects((value) => {
    value.identity.qualificationRun += 1;
  }, /identity\/head/);
  rejects((value) => {
    value.identity.qualificationAttempt += 1;
  }, /identity\/head/);
  rejects((value) => {
    value.identity.operatorRun = value.identity.qualificationRun;
  }, /Q and O/);
  rejects((value) => {
    value.identity.origin = "release";
  }, /origin/);
  rejects((value) => {
    Object.assign(value.identity, { releaseRun: 90 });
  }, /authority/);
  rejects((value) => {
    Object.assign(value.identity, { releaseAttempt: 1 });
  }, /authority/);
  rejects((value) => {
    value.qualificationRun.actor.login = "outsider";
  }, /custody/);
  rejects((value) => {
    value.qualificationRun.triggering_actor.login = "outsider";
  }, /custody/);
  rejects((value) => {
    value.qualificationRun.head_repository.full_name = "fork/repo";
  }, /foreign repository/);
  rejects((value) => {
    value.releaseRunAbsence.hRunPages[0].workflow_runs[0].repository.full_name = "fork/repo";
  }, /foreign repository/);
  rejects((value) => {
    value.releaseRunAbsence.hRunPages[0].workflow_runs[0].head_sha = cut;
  }, /foreign H/);
});

test("archived observation and re-read full-H identity changes refuse offline replay", () => {
  const changedIdentity = receipt();
  changedIdentity.identity.qualificationAttempt += 1;
  assert.throws(() => assertPreReleaseQualification(changedIdentity), /receipt\/guard identity/);
  const changedAttempt = receipt();
  changedAttempt.originalEvidence.HRunPages[0].workflow_runs[0].run_attempt += 1;
  assert.throws(() => assertPreReleaseQualification(changedAttempt), /latest eligible/);
  const changedJobs = receipt();
  changedJobs.originalEvidence.observations[0].jobPages[0].jobs.at(-1)!.conclusion = "success";
  assert.throws(
    () => assertPreReleaseQualification(changedJobs),
    /observed jobs inventory differs/,
  );
  const changedRun = receipt();
  changedRun.originalEvidence.observations[0].run.run_attempt += 1;
  assert.throws(() => assertPreReleaseQualification(changedRun), /observation identity differs/);
});

test("every guard and original H row needs unambiguous classification and ordering metadata", () => {
  const invalidFields: Array<[string, unknown]> = [
    ...["path", "event", "head_branch", "display_title"].flatMap((field) =>
      [undefined, null, 1, {}, [], ""].map((value): [string, unknown] => [field, value]),
    ),
    ...[undefined, null, "", "not a date", 1, {}].map((value): [string, unknown] => [
      "run_started_at",
      value,
    ]),
  ];
  for (const location of ["guard", "original"] as const) {
    for (const [field, invalid] of invalidFields) {
      const value = receipt();
      for (const pages of [
        value.guards.releaseRunAbsence.hRunPages,
        value.originalEvidence.HRunPages,
      ]) {
        pages[0].workflow_runs.push({
          ...run(21),
          path: "dynamic/github-code-quality/codeql",
          event: "dynamic",
        });
        pages[0].total_count = 2;
      }
      const pages =
        location === "guard"
          ? value.guards.releaseRunAbsence.hRunPages
          : value.originalEvidence.HRunPages;
      const extra = pages[0].workflow_runs[1] as Record<string, unknown>;
      if (invalid === undefined) delete extra[field];
      else extra[field] = invalid;
      assert.throws(
        () => assertPreReleaseQualification(value),
        /all H.*(?:string required|finite ordering timestamp)/,
        `${location} ${field}: ${JSON.stringify(invalid)}`,
      );
    }
  }
  for (const path of [
    "dynamic/github-code-quality/codeql",
    ".github/workflows/future policy 日本語.yaml",
  ]) {
    for (const fallback of ["created_at", "updated_at"]) {
      const value = guards();
      const extra = { ...run(21), path, event: "dynamic" };
      Object.assign(extra, { run_started_at: null, [fallback]: "2026-10-17T02:00:00Z" });
      value.releaseRunAbsence.hRunPages[0].workflow_runs.push(extra);
      value.releaseRunAbsence.hRunPages[0].total_count = 2;
      assert.doesNotThrow(() => assertPreReleaseQualification(value));
    }
  }
  const invalidPreferred = guards();
  Object.assign(invalidPreferred.releaseRunAbsence.hRunPages[0].workflow_runs[0], {
    run_started_at: "invalid",
    created_at: "2026-10-17T01:00:00Z",
  });
  assert.throws(() => assertPreReleaseQualification(invalidPreferred), /finite ordering timestamp/);
});

test("CLI is silent on valid JSON and returns full errors for invalid evidence/arguments", () => {
  const success = cli(receipt());
  assert.equal(success.status, 0, success.stderr);
  assert.equal(success.stdout, "");
  assert.equal(success.stderr, "");
  const invalid = cli({});
  assert.equal(invalid.status, 1);
  assert.match(invalid.stderr, /Error: Pre-release qualification:/);
  assert.equal(invalid.stdout, "");
  const args = cli(guards(), ["--publish"]);
  assert.equal(args.status, 1);
  assert.match(args.stderr, /no arguments/);
});
