import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import { planAffectedJs } from "../../tools/support/compat/github/plan-affected-js.ts";
import { jsInstallFilters } from "../../tools/support/compat/github/install-js-checks.ts";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const actor = "tools/support/compat/content-mapper/direct-child-props-11809.ts";
const actorInputs = [
  actor,
  "tools/support/compat/content-mapper/direct-child-props-runtime-11809.ts",
  "tests/tooling/content-mapper-direct-child-props.test.ts",
];
const workflowPath = ".github/workflows/pr-js-checks.yml";
const contract = "tests/tooling/content-mapper-source-node22-workflow.test.ts";
const proofName = "Verify packaged direct-child props with source native on Node 22";
type Step = {
  name?: string;
  if?: string;
  run?: string;
  shell?: string;
  uses?: string;
  env?: Record<string, string>;
  with?: Record<string, string | number>;
  "continue-on-error"?: boolean;
};
type Workflow = { jobs: Record<string, { steps: Step[]; needs?: string[] }> };
const workflow = parse(fs.readFileSync(path.join(root, workflowPath), "utf8")) as Workflow;
const steps = workflow.jobs["pr-js-package-conformance"]!.steps;
function required(name: string): Step {
  const result = steps.find((step) => step.name === name);
  assert.ok(result, `missing source reporter step: ${name}`);
  return result;
}

test("source reporter inputs select the existing native UI dependency closure", () => {
  const workspace = [
    { name: "@vizejs/native", path: "npm/native", dependencies: [] },
    { name: "@vizejs/ui", path: "npm/ui", dependencies: ["@vizejs/native"] },
    { name: "@vizejs/fresco-native", path: "npm/fresco-native", dependencies: [] },
  ];
  for (const file of actorInputs) {
    const selected = planAffectedJs(workspace, [file]);
    assert.equal(selected.scope, "affected", file);
    assert.equal(selected.native, true, file);
    assert.deepEqual(selected.conformance, { fresco: false, ui: true }, file);
  }
  assert.deepEqual(planAffectedJs(workspace, ["README.md"]).conformance, {
    fresco: false,
    ui: false,
  });
});

test("the frozen UI installer keeps its native and UI dependencies and adds the source CLI", () => {
  const filters = jsInstallFilters("ui-conformance");
  for (const pkg of ["native", "ui", "cli"]) assert.ok(filters.includes(`{./npm/${pkg}}...`));
  assert.equal(filters.length, new Set(filters).size);
  assert.ok(!jsInstallFilters("fresco-conformance").includes("{./npm/cli}..."));
});

test("one existing UI step builds only CLI JavaScript and calls the strict source actor", () => {
  const proof = required(proofName);
  assert.equal(proof.if, required("Check UI package conformance").if);
  assert.equal(proof.shell, "bash");
  assert.equal(proof["continue-on-error"], undefined);
  assert.equal(proof.env?.PATH, undefined);
  assert.ok(proof.run);
  assert.ok(steps.indexOf(proof) > steps.indexOf(required("Check UI package conformance")));
  assert.ok(
    steps.indexOf(proof) >
      steps.indexOf(required("Verify whole source CSS API maps with the built native image")),
  );
  assert.doesNotMatch(proof.run!, /build:ci|cargo |setup-node|\|\| true/);
  assert.equal(proof.run!.split("\n").filter((line) => line.includes(".mjs")).length, 1);
});

test("source reporter raw archives and receipts survive failure and cancellation", () => {
  const upload = required("Retain source reporter archives and whole outcomes");
  assert.equal(
    upload.if,
    "${{ always() && inputs.run-js && matrix.surface == 'ui' && fromJSON(inputs.js-plan).conformance.ui }}",
  );
  assert.match(upload.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/u);
  assert.deepEqual(String(upload.with?.path).trim().split("\n"), [
    "${{ runner.temp }}/direct-child-props-source-11809/",
    "!${{ runner.temp }}/direct-child-props-source-11809/**/node_modules",
    "!${{ runner.temp }}/direct-child-props-source-11809/**/node_modules/**",
  ]);
  assert.equal(upload.with?.["if-no-files-found"], "error");
  assert.equal(upload["continue-on-error"], undefined);
  const source = parse(
    fs.readFileSync(path.join(root, ".github/workflows/pr-source-checks.yml"), "utf8"),
  ) as Workflow;
  assert.ok(source.jobs["source-report"]!.needs?.includes("pr-js-packages"));
});

test("both actor sources and their workflow select this source routing law", () => {
  for (const input of [workflowPath, ...actorInputs]) {
    assert.ok(planToolingTests([input], { cwd: root }).tests.includes(contract), input);
  }
});

test("the actual workflow command preserves a failed pack and a failed consumer", () => {
  const proof = required(proofName);
  assert.ok(proof.run);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reporter-route-"));
  try {
    const bin = path.join(directory, "bin");
    fs.mkdirSync(bin);
    fs.mkdirSync(path.join(directory, "npm/cli"), { recursive: true });
    fs.writeFileSync(
      path.join(bin, "vp"),
      '#!/bin/bash\nprintf "%s\\n" "$@" > "$MOCK_PACK"\nprintf "controlled pack stdout\\n"\nprintf "controlled pack stderr\\n" >&2\nexit "$PACK_STATUS"\n',
      { mode: 0o755 },
    );
    fs.writeFileSync(
      path.join(bin, "node"),
      '#!/bin/bash\nif [ "$1" = "npm/native/scripts/native-build-receipt.mjs" ]; then printf "producer receipt\\n" > "$MOCK_PRODUCER"; exit 0; fi\nmkdir -p "$3"\nprintf "%s\\n" "$@" > "$MOCK_ARGS"\nprintf "failed outcome\\n" > "$3/receipt.json"\nexit 23\n',
      { mode: 0o755 },
    );
    const args = path.join(directory, "args.txt");
    const pack = path.join(directory, "pack.txt");
    const producer = path.join(directory, "producer.txt");
    const env = {
      ...process.env,
      PATH: `${bin}:/usr/bin:/bin`,
      RUNNER_TEMP: directory,
      MOCK_ARGS: args,
      MOCK_PACK: pack,
      MOCK_PRODUCER: producer,
      PACK_STATUS: "0",
    };
    const run = () =>
      spawnSync("/bin/bash", ["-e", "-o", "pipefail", "-c", proof.run!], {
        cwd: directory,
        env,
        encoding: "utf8",
      });
    assert.equal(run().status, 23);
    assert.equal(fs.readFileSync(producer, "utf8"), "producer receipt\n");
    assert.equal(fs.readFileSync(pack, "utf8"), "pack\n");
    const output = path.join(directory, "direct-child-props-source-11809");
    assert.equal(
      fs.readFileSync(path.join(output, "cli-pack.stdout"), "utf8"),
      "controlled pack stdout\n",
    );
    assert.equal(
      fs.readFileSync(path.join(output, "cli-pack.stderr"), "utf8"),
      "controlled pack stderr\n",
    );
    assert.deepEqual(fs.readFileSync(args, "utf8").trim().split("\n"), [
      actor,
      "--capture",
      output,
    ]);
    assert.equal(fs.readFileSync(path.join(output, "receipt.json"), "utf8"), "failed outcome\n");
    fs.unlinkSync(args);
    env.PACK_STATUS = "17";
    assert.equal(run().status, 17);
    assert.equal(fs.existsSync(args), false);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
