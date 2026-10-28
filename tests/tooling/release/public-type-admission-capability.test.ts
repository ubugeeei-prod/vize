import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

type Step = {
  name?: string;
  if?: string;
  run?: string;
  env?: Record<string, string>;
  with?: { path?: string };
};
type Job = { if?: string; steps: Step[] };
type Workflow = { jobs: Record<string, Job> };
const observerName = "Qualify independent published template type admission";
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

test("the isolated host bootstrap stays after original CPP restoration and before admission", () => {
  for (const workflow of workflows) {
    const cpp = job(workflow, "public-custom-properties");
    const restore = cpp.steps.findIndex(
      (step) =>
        step.name === "Qualify fresh public custom properties and authored directive ownership",
    );
    const host = cpp.steps.findIndex(
      (step) => step.name === "Install the isolated native TypeScript mapper host",
    );
    const observer = cpp.steps.findIndex((step) => step.name === observerName);
    assert.ok(restore >= 0 && host > restore && observer > host);
    assert.equal(cpp.steps[host]?.if, "inputs.type_admission == true");
    assert.equal(cpp.steps[host]?.env?.GH_TOKEN, undefined);
    assert.match(
      cpp.steps[host]?.run ?? "",
      /env -u GH_TOKEN -u GITHUB_TOKEN -u NODE_AUTH_TOKEN -u NPM_TOKEN node/,
    );
    assert.ok(
      cpp.steps.some((step) =>
        step.with?.path?.includes("public-type-admission-mapper-host-proof/"),
      ),
    );
  }
});

test("capability mode is manual source proof and preserves every ordinary unit selection", () => {
  const wrapper = workflows[1];
  assert.ok(wrapper);
  const capability = job(wrapper, "type-admission-source-capability");
  const units = job(wrapper, "contracts");
  for (const phase of ["", "before", "after", "release-contracts", "type-admission-capability"])
    for (const event of ["pull_request", "push", "workflow_dispatch"])
      for (const repository of ["ubugeeei-prod/vize", "example/fork"]) {
        const context = { github: { repository, event_name: event }, inputs: { phase } };
        assert.equal(
          runInNewContext(capability.if ?? "", context),
          repository === "ubugeeei-prod/vize" &&
            event === "workflow_dispatch" &&
            phase === "type-admission-capability",
        );
        assert.equal(
          runInNewContext(units.if ?? "", context),
          event !== "workflow_dispatch" ||
            !["release-contracts", "type-admission-capability"].includes(phase),
        );
      }
  const proof = capability.steps.findIndex(
    (step) => step.name === "Prove the known public source for the isolated host capability",
  );
  const auth = capability.steps.findIndex(
    (step) => step.name === "Authenticate the existing public CLI and native TypeScript provider",
  );
  assert.ok(proof >= 0 && auth > proof);
  for (const [index, step] of capability.steps.entries())
    if (step.env?.GH_TOKEN) assert.equal(index, proof);
  for (const step of capability.steps.slice(auth).filter((step) => step.run))
    assert.match(
      step.run ?? "",
      /^env -u GH_TOKEN -u GITHUB_TOKEN -u NODE_AUTH_TOKEN -u NPM_TOKEN node /,
    );
});

test("the actual source-capability proof rejects mixed or false public identities before install", async () => {
  const wrapper = workflows[1];
  assert.ok(wrapper);
  const proof = job(wrapper, "type-admission-source-capability").steps.find(
    (step) => step.name === "Prove the known public source for the isolated host capability",
  );
  assert.ok(proof?.run);
  const match = /node --input-type=module <<'NODE'\n([\s\S]+?)\nNODE/.exec(proof.run);
  assert.ok(match?.[1]);
  const script = match[1].replace(/^import .*;\n/gm, "");
  const source = "3778a06863bc70bf63b435ae816cca134d2d438e";
  const base = "7c2f6f46172404a8f95335a79ee2b95094f6b7ee";
  const baseline = {
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/test/public-type-admission-release",
    CAPABILITY_PHASE: "type-admission-capability",
    CAPABILITY_ADMISSION: "false",
    CAPABILITY_OBSERVERS: "false",
    CAPABILITY_CPP: "false",
    GH_TOKEN: "inert-fixture",
    PUBLIC_VERSION: "0.603.0",
    PUBLIC_SOURCE_SHA: source,
    PUBLIC_CANDIDATE: source,
    RUNNER_TEMP: "/proof",
    GITHUB_SHA: "9".repeat(40),
    GITHUB_RUN_ID: "123",
  };
  for (const mode of [
    "valid",
    "version",
    "candidate",
    "phase",
    "branch",
    "admission",
    "observers",
    "cpp",
    "tag",
    "draft",
    "ancestry",
  ]) {
    const env = { ...baseline };
    if (mode === "version") env.PUBLIC_VERSION = "0.604.0";
    if (mode === "candidate") env.PUBLIC_CANDIDATE = "0".repeat(40);
    if (mode === "phase") env.CAPABILITY_PHASE = "after";
    if (mode === "branch") env.GITHUB_REF = "refs/heads/main";
    if (mode === "admission") env.CAPABILITY_ADMISSION = "true";
    if (mode === "observers") env.CAPABILITY_OBSERVERS = "true";
    if (mode === "cpp") env.CAPABILITY_CPP = "true";
    const writes: { name: string; value: unknown }[] = [];
    const fetch = async (url: string) => ({
      status: 200,
      json: async (): Promise<unknown> =>
        url.includes("/releases/tags/")
          ? { tag_name: "v0.603.0", draft: mode === "draft", published_at: "2026-01-01T00:00:00Z" }
          : url.includes("/tags?")
            ? [{ name: "v0.603.0", commit: { sha: mode === "tag" ? "0".repeat(40) : source } }]
            : {
                base_commit: { sha: base },
                merge_base_commit: { sha: mode === "ancestry" ? "0".repeat(40) : base },
                behind_by: 0,
                status: "ahead",
              },
    });
    const run = () =>
      runInNewContext(
        "(async () => { " + script + " })()",
        {
          assert,
          path,
          AbortSignal,
          process: { env },
          fetch,
          fs: {
            writeFileSync: (name: string, bytes: string, options: unknown) => {
              assert.deepEqual(JSON.parse(JSON.stringify(options)), { flag: "wx" });
              writes.push({ name, value: JSON.parse(bytes) as unknown });
            },
          },
        },
        { timeout: 1000 },
      ) as Promise<void>;
    if (mode === "valid") {
      await run();
      const plan = writes.find((item) => item.name.endsWith("capability-plan.json"))
        ?.value as Record<string, unknown>;
      assert.equal(plan.schema, "vize.typeAdmission.sourceCapabilityEligibility.v1");
      assert.equal(plan.coreReleaseEligible, false);
      assert.equal(plan.scope, "source-capability-only");
      assert.equal(plan.publicationConfirmed, undefined);
      assert.equal(plan.requiredSource, undefined);
      assert.equal(plan.firstContainingTag, undefined);
      assert.deepEqual(plan.mapperHost, {
        route: "hosted-content-mapper",
        sdkVersion: "7.1.0-beta",
        workspace: "/proof/public-type-admission-capability-host",
        setupReceipt: "/proof/public-type-admission-capability-host-proof/setup.json.txt",
      });
    } else {
      await assert.rejects(run(), mode);
      assert.deepEqual(writes, []);
    }
  }
});

test("full admission and source preflight preserve original CPP custody even on failure", () => {
  for (const workflow of workflows) {
    const cpp = job(workflow, "public-custom-properties");
    const before = cpp.steps.findIndex(
      (step) => step.name === "Preserve the original CPP consumer before template admission",
    );
    const observer = cpp.steps.findIndex((step) => step.name === observerName);
    const after = cpp.steps.findIndex(
      (step) => step.name === "Verify the original CPP consumer after template admission",
    );
    assert.ok(before >= 0 && observer > before && after > observer);
    assert.equal(cpp.steps[before]?.if, "inputs.type_admission == true");
    assert.equal(cpp.steps[after]?.if, "always() && inputs.type_admission == true");
    for (const index of [before, after])
      assert.match(
        cpp.steps[index]?.run ?? "",
        /type-admission-cpp-custody.ts "\$RUNNER_TEMP\/public-custom-properties-consumer" "\$PUBLIC_VERSION" (?:before|after) "\$RUNNER_TEMP\/public-type-admission-cpp-custody.json"/,
      );
  }
  const wrapper = workflows[1];
  assert.ok(wrapper);
  const capability = job(wrapper, "type-admission-source-capability");
  const before = capability.steps.findIndex(
    (step) => step.name === "Preserve the original CPP consumer before source capability",
  );
  const probe = capability.steps.findIndex(
    (step) =>
      step.name === "Verify actual hosted mapper capability and its physical public provider",
  );
  const after = capability.steps.findIndex(
    (step) => step.name === "Verify the original CPP consumer after source capability",
  );
  assert.ok(before >= 0 && probe > before && after > probe);
  assert.equal(capability.steps[after]?.if, "always()");
});
