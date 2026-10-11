import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import {
  assertTrace,
  campaigns,
  digest,
  materializeFixtures,
  type Row,
} from "../../../tools/support/release/vapor_acceptance/fixtures.ts";

type Workflow = {
  jobs: Record<
    string,
    {
      env: Record<string, string>;
      defaults: { run: { shell: string } };
      steps: { name?: string; run?: string; shell?: string }[];
    }
  >;
};

test("public Vapor workflow starts with available contexts and refuses runtime overrides", () => {
  const workflow = parse(
    fs.readFileSync(
      new URL("../../../.github/workflows/release-vapor-acceptance.yml", import.meta.url),
      "utf8",
    ),
  ) as Workflow;
  const validate = (value: Workflow) => {
    for (const job of Object.values(value.jobs)) {
      for (const entry of Object.values(job.env)) {
        assert.doesNotMatch(
          entry,
          /\$\{\{[^}]*\brunner(?:\.|\[)/u,
          "runner is unavailable in job.env",
        );
      }
      assert.equal(job.defaults.run.shell, "bash", "piped consumer failures require pipefail");
      for (const step of job.steps.filter((entry) => entry.run)) {
        assert.ok(!step.shell || step.shell === "bash", "consumer steps retain pipefail");
      }
    }
  };
  validate(workflow);
  const invalid = structuredClone(workflow);
  const job = workflow.jobs["public-browser"];
  const invalidJob = invalid.jobs["public-browser"];
  assert.ok(job && invalidJob);
  invalidJob.env.PUBLIC_OUTPUT = "${{ runner.temp }}/evidence";
  assert.throws(() => validate(invalid), /runner is unavailable/u);
  const guard = job.steps.find(
    (entry) => entry.name === "Require reviewed main and absent runtime overrides",
  );
  const guardScript = guard?.run;
  assert.ok(guardScript);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-vapor-workflow-law-"));
  try {
    const environment = path.join(root, "github-env");
    fs.writeFileSync(environment, "");
    const env = {
      PATH: process.env.PATH,
      GITHUB_REPOSITORY: "ubugeeei-prod/vize",
      GITHUB_REF: "refs/heads/main",
      GITHUB_RUN_ID: "123",
      GITHUB_RUN_ATTEMPT: "2",
      GITHUB_ENV: environment,
      RUNNER_TEMP: root,
    };
    const execute = (extra: Record<string, string>) =>
      spawnSync("bash", ["-eo", "pipefail", "-c", guardScript], {
        env: { ...env, ...extra },
        encoding: "utf8",
      });
    const accepted = execute({});
    assert.equal(accepted.status, 0, accepted.stdout + accepted.stderr);
    assert.equal(
      fs.readFileSync(environment, "utf8"),
      `PUBLIC_OUTPUT=${root}/vize-public-vapor-evidence-123-2\n`,
    );
    fs.writeFileSync(environment, "");
    const refused = execute({ VIZE_BINDING_PATH: "source-override.node" });
    assert.equal(refused.status, 1);
    assert.match(refused.stdout, /Runtime override must be absent: VIZE_BINDING_PATH/u);
    assert.equal(fs.readFileSync(environment, "utf8"), "");
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

test("public Vapor preparation preserves raw authorities and decoded original SFC sources", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-vapor-fixture-law-"));
  const authorities = new Map<string, Buffer>();
  for (const campaign of campaigns) {
    const prefix = `tests/_fixtures/differential/compiler/${campaign.directory}/`;
    const app = `<script setup vapor>\nconst label = '雪🌸';\n</script>\n<template><span>${campaign.issue}</span></template>\n`;
    const child = "<script setup vapor></script>\n<template><span>child</span></template>\n";
    authorities.set(
      prefix + campaign.app,
      Buffer.from(campaign.app.endsWith(".json") ? JSON.stringify(app) + "\n" : app),
    );
    authorities.set(
      prefix + campaign.child,
      Buffer.from(campaign.child.endsWith(".json") ? JSON.stringify(child) + "\n" : child),
    );
    authorities.set(
      prefix + "original-issue.md",
      Buffer.from(`Original issue ${campaign.issue}\n`),
    );
    authorities.set(prefix + "reported.vue.txt", Buffer.from(app));
    authorities.set(
      prefix + "controls.json",
      Buffer.from(
        JSON.stringify(
          Array.from({ length: campaign.count }, (_, index) => ({
            name: `retained-${campaign.issue}-${index}`,
            context: {},
            steps: [],
          })),
        ),
      ),
    );
  }
  try {
    const result = materializeFixtures(root, (file) => {
      const bytes = authorities.get(file);
      assert.ok(bytes, "missing source authority must fail");
      return bytes;
    });
    assert.equal(result.authorities.length, 20);
    assert.equal(result.rows.length, 8);
    for (const authority of result.authorities) {
      const suffix = authority.file.slice("tests/_fixtures/differential/compiler/".length);
      assert.deepEqual(
        fs.readFileSync(path.join(root, "original", suffix)),
        authorities.get(authority.file),
      );
      assert.equal(authority.sha256, digest(authorities.get(authority.file)!));
    }
    const encoded = result.rows.find((row) => row.issue === 8625)!;
    assert.equal(
      encoded.app,
      JSON.parse(
        authorities
          .get(`tests/_fixtures/differential/compiler/vapor-root-fallthrough-8625/App.vue.json`)!
          .toString(),
      ),
    );
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

test("public Vapor refuses a reduced original denominator before executing a consumer", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-vapor-denominator-law-"));
  try {
    assert.throws(
      () =>
        materializeFixtures(root, (file) =>
          Buffer.from(file.endsWith("controls.json") ? "[]" : "<template><span /></template>"),
        ),
      /original control denominator/u,
    );
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

test("public Vapor exact original namespace, event update and disposal oracles reject corruption", () => {
  const row: Row = {
    issue: 8627,
    name: "typed-original",
    directory: "fixture",
    app: "source",
    child: "source",
    context: {},
    steps: [{ click: "#typed-click" }],
    expectedNamespaces: ["http://www.w3.org/1999/xhtml"],
    expectedTrees: [["none"], ["click"], []],
    expectedEvents: ["click"],
  };
  const trace = [
    { tree: ["none"], namespaces: ["http://www.w3.org/1999/xhtml"], events: [] },
    { tree: ["click"], namespaces: ["http://www.w3.org/1999/xhtml"], events: ["click"] },
    { tree: [], namespaces: [], events: ["click"] },
  ];
  assert.doesNotThrow(() => assertTrace(row, trace));
  for (const change of [
    (value: typeof trace) => {
      value[0].namespaces[0] = "http://www.w3.org/2000/svg";
    },
    (value: typeof trace) => {
      value[1].tree = ["none"];
    },
    (value: typeof trace) => {
      value[1].events = [];
    },
    (value: typeof trace) => {
      value[2].tree = ["retained"];
    },
  ]) {
    const changed = structuredClone(trace);
    change(changed);
    assert.throws(() => assertTrace(row, changed), assert.AssertionError);
  }
});
