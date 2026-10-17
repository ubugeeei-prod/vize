import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import { withoutPackedConfigExtension } from "./support/native-packed-config-extension.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const original = fs.readFileSync(
  path.join(
    root,
    "tests/_fixtures/differential/vite-native-image-identity/native-smoke-before.yml",
  ),
);
type Job = {
  if?: string;
  steps: Array<{
    if?: string;
    name?: string;
    run?: string;
    uses?: string;
    "working-directory"?: string;
    with?: Record<string, unknown>;
  }>;
  [key: string]: unknown;
};
const before = parse(original.toString()) as {
  jobs: Record<string, Job>;
  on: { schedule: unknown };
};
const after = parse(
  withoutPackedConfigExtension(
    fs.readFileSync(path.join(root, ".github/workflows/native-smoke.yml"), "utf8"),
  ),
) as {
  jobs: Record<string, Job>;
  on: { schedule: unknown; workflow_dispatch: { inputs: Record<string, { default: unknown }> } };
};
const extraNames = [
  "Test native image token producer",
  "Capture selected native image authority",
  "Preserve selected native image",
];
const moonSelector =
  "github.event_name == 'workflow_dispatch' && inputs.host_only && inputs.target == 'darwin-arm64'";

function authenticateDefaultJob(name: string, configured: Job) {
  const { if: condition, ...job } = configured;
  assert.equal(
    condition,
    "github.event_name != 'workflow_dispatch' || !inputs.host_only || inputs.target != 'darwin-arm64'",
  );
  const steps = job.steps.filter((step) => !extraNames.includes(step.name ?? ""));
  assert.equal(
    steps.some((step) => step.uses === "./.github/actions/setup-moonbit"),
    false,
    "the token laws run on Node, so no host row installs MoonBit",
  );
  if (name === "host-native-smoke") {
    const laws = configured.steps.find((entry) => entry.name === extraNames[0])?.if ?? "";
    assert.equal(laws, moonSelector);
    for (const event of ["push", "schedule", "workflow_dispatch"])
      for (const hostOnly of [undefined, false, true])
        for (const target of [
          undefined,
          "all",
          "linux-x64-gnu",
          "linux-arm64-gnu",
          "darwin-x64",
          "darwin-arm64",
          "win32-x64-msvc",
          "win32-arm64-msvc",
        ])
          assert.equal(
            Boolean(
              runInNewContext(laws, {
                github: { event_name: event },
                inputs: { host_only: hostOnly, target },
              }),
            ),
            event === "workflow_dispatch" && hostOnly === true && target === "darwin-arm64",
          );
    // Restore the original setup that #8840 removed with its only Moon consumer.
    const msvc = steps.findIndex((step) => step.name === "Setup MSVC toolchain (Windows)");
    assert.ok(msvc >= 0);
    steps.splice(msvc + 1, 0, {
      uses: "./.github/actions/setup-moonbit",
      if: "matrix.target != 'darwin-x64'",
    });
  }
  assert.deepEqual(
    { ...job, steps },
    before.jobs[name],
    `every original ${name} field remains exact`,
  );
}

test("the full original six-host and twelve-fresh native smoke contract stays intact", () => {
  assert.equal(
    createHash("sha256").update(original).digest("hex"),
    "0eefad4e29b3851daa472fd3d035d4dfd8f2dfb862f4d9e6beb45c36b7acfa31",
  );
  assert.deepEqual(after.on.schedule, before.on.schedule);
  assert.equal(after.on.workflow_dispatch.inputs.target.default, "all");
  assert.equal(after.on.workflow_dispatch.inputs.host_only.default, false);
  for (const name of ["host-native-smoke", "fresh-install-smoke"])
    authenticateDefaultJob(name, after.jobs[name]);
  for (const changed of ["reinstalled-moon", "unknown-msvc-field", "removed-cli-command"]) {
    const altered = structuredClone(after.jobs["host-native-smoke"]);
    if (changed === "reinstalled-moon")
      altered.steps.splice(1, 0, { uses: "./.github/actions/setup-moonbit" });
    else if (changed === "unknown-msvc-field")
      Object.assign(
        altered.steps.find((step) => step.name === "Setup MSVC toolchain (Windows)")!,
        { unreviewedField: true },
      );
    else altered.steps.find((step) => step.name === "Build CLI")!.run = "removed-command";
    assert.throws(() => authenticateDefaultJob("host-native-smoke", altered), {
      code: "ERR_ASSERTION",
    });
  }
});

test("the explicitly selected producer reuses every host step and is not full matrix credit", () => {
  const selected = after.jobs["selected-host-native-producer"];
  assert.equal(
    selected.if,
    "github.event_name == 'workflow_dispatch' && inputs.host_only && inputs.target == 'darwin-arm64'",
  );
  assert.deepEqual(selected.steps, after.jobs["host-native-smoke"].steps);
  assert.equal(selected["runs-on"], "blacksmith-12vcpu-macos-15");
  assert.equal(selected["timeout-minutes"], before.jobs["host-native-smoke"]["timeout-minutes"]);
  assert.deepEqual(selected.env, before.jobs["host-native-smoke"].env);
  const extra = selected.steps.filter((step) => extraNames.includes(step.name ?? ""));
  assert.equal(extra.length, 3);
  assert.equal(extra[0].run, "node --test tests/tooling/native-build-token.test.ts");
  assert.equal(extra[0]["working-directory"], undefined);
  assert.equal(extra[1].run, "node npm/native/scripts/native-build-receipt.mjs");
  assert.match(extra[2].uses ?? "", /^actions\/upload-artifact@/);
  assert.equal(extra[2].with?.["include-hidden-files"], true);
  assert.equal(extra[2].with?.["if-no-files-found"], "error");
});
