import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";
import { digest } from "../../tools/support/release/vapor_acceptance/fixtures.ts";
import {
  assertNamespaceTrace,
  campaignDefinition,
  materializeNamespaceFixtures,
  type PhysicalRow,
  type PhysicalTrace,
} from "../../tools/support/release/vapor_acceptance/namespace.ts";

const ids = [
  "svg-a",
  "svg-filter",
  "svg-blur",
  "svg-view",
  "svg-switch",
  "foreign-html-a",
  "html-a",
];
const html = "http://www.w3.org/1999/xhtml",
  svg = "http://www.w3.org/2000/svg";
const namespaces = [html, ...Array(9).fill(svg), html, html];
const colors = ["rgb(200, 0, 0)", "rgb(0, 0, 200)"];
const classes = ["node array active", "node selected", "node"];
// This inert validator model never executes an SFC and grants no browser/product credit.
const model = {
  name: "inherited-svg-and-html-integration-bindings",
  context: { classes: ["array", { active: true }], style: { color: colors[0] } },
  steps: [{ patch: { classes: { selected: true } } }, { patch: { classes: null, style: null } }],
  boundIds: ids,
  expectedTrees: [["fixed mount"], ["fixed update"], ["fixed clear"], []],
  expectedNamespacePhases: [namespaces, namespaces, namespaces, []],
  expectedClassPhases: [...classes.map((value) => Array(7).fill(value)), []],
  expectedColorPhases: colors.map((value) => Array(7).fill(value)),
};
function fixture() {
  const prefix = "tests/_fixtures/differential/compiler/vapor-svg-namespace-8624";
  const files = new Map([
    [`${prefix}/App.vue.txt`, Buffer.from("<template>whole inert custody model</template>\n")],
    [`${prefix}/Child.vue.txt`, Buffer.from("<template>whole child custody model</template>\n")],
    [`${prefix}/controls.json`, Buffer.from(JSON.stringify([model]) + "\n")],
  ]);
  const entries = [...files].map(([file, bytes]) => ({
    path: path.basename(file),
    sha256: digest(bytes),
  }));
  files.set(
    `${prefix}.manifest.json`,
    Buffer.from(
      JSON.stringify({
        schema: "vize.differential.runtime-pack",
        version: 1,
        product: "compiler",
        issue: "https://github.com/ubugeeei-prod/vize/issues/8624",
        cases: [
          {
            id: "compiler/vapor-svg-namespace-8624",
            inputs: { root: "vapor-svg-namespace-8624", files: entries },
            reference: {
              path: "vapor-svg-namespace-8624/controls.json",
              sha256: entries[2].sha256,
            },
          },
        ],
      }) + "\n",
    ),
  );
  return files;
}
const read = (files: Map<string, Buffer>) => (file: string) => {
  const bytes = files.get(file);
  assert.ok(bytes, `missing immutable H authority: ${file}`);
  return bytes;
};

test("separate namespace campaign retains all raw H blobs and cannot substitute reporter counts", () => {
  assert.deepEqual(campaignDefinition("reported"), {
    name: "reported",
    schema: "vize-public-reported-vapor-v1",
    original: 6,
    authored: 2,
    rows: 8,
    observations: 16,
  });
  assert.deepEqual(campaignDefinition("svg-namespace"), {
    name: "svg-namespace",
    schema: "vize-public-svg-namespace-v1",
    original: 0,
    authored: 1,
    rows: 1,
    observations: 2,
  });
  for (const name of [undefined, "", "all", { original: 6 }, "reported,svg-namespace"])
    assert.throws(() => campaignDefinition(name));
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-namespace-custody-law-"));
  try {
    const files = fixture();
    const result = materializeNamespaceFixtures(root, read(files));
    assert.equal(result.rows.length, 1);
    assert.equal(result.authorities.length, 4);
    for (const authority of result.authorities) {
      const source = files.get(authority.file)!;
      const retained = fs.readFileSync(
        path.join(
          root,
          "original",
          authority.file.slice("tests/_fixtures/differential/compiler/".length),
        ),
      );
      assert.ok(source.equals(retained));
      assert.equal(digest(retained), authority.sha256);
    }
  } finally {
    fs.rmSync(root, { recursive: true });
  }
  for (const mutation of [
    "missing",
    "source",
    "manifest",
    "count",
    "bindings",
    "classes",
    "colors",
    "namespace",
  ]) {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-namespace-refusal-law-"));
    try {
      const files = fixture(),
        prefix = "tests/_fixtures/differential/compiler/vapor-svg-namespace-8624";
      if (mutation === "missing") files.delete(`${prefix}/Child.vue.txt`);
      else if (mutation === "source")
        files.set(`${prefix}/App.vue.txt`, Buffer.from("changed source"));
      else if (mutation === "manifest") {
        const value = JSON.parse(files.get(`${prefix}.manifest.json`)!.toString());
        value.cases[0].inputs.files.pop();
        files.set(`${prefix}.manifest.json`, Buffer.from(JSON.stringify(value)));
      } else {
        const value = JSON.parse(files.get(`${prefix}/controls.json`)!.toString());
        if (mutation === "count") value.push(value[0]);
        if (mutation === "bindings") value[0].boundIds.pop();
        if (mutation === "classes") value[0].expectedClassPhases[1][0] = "node";
        if (mutation === "colors") value[0].expectedColorPhases[1][0] = colors[0];
        if (mutation === "namespace") value[0].expectedNamespacePhases[1][10] = svg;
        const bytes = Buffer.from(JSON.stringify(value)),
          manifest = JSON.parse(files.get(`${prefix}.manifest.json`)!.toString());
        files.set(`${prefix}/controls.json`, bytes);
        manifest.cases[0].inputs.files[2].sha256 = digest(bytes);
        manifest.cases[0].reference.sha256 = digest(bytes);
        files.set(`${prefix}.manifest.json`, Buffer.from(JSON.stringify(manifest)));
      }
      assert.throws(() => materializeNamespaceFixtures(directory, read(files)), mutation);
    } finally {
      fs.rmSync(directory, { recursive: true });
    }
  }
});

test("physical namespace ownership, update/removal and complete phases refuse false greens", () => {
  const row: PhysicalRow = {
    ...model,
    issue: 8624,
    directory: "vapor-svg-namespace-8624",
    app: "whole inert model",
    child: "whole child",
  };
  const trace: PhysicalTrace[] = model.expectedTrees.map((tree, phase) => ({
    tree,
    namespaces: model.expectedNamespacePhases[phase],
    events: [],
    bindings:
      phase === 3
        ? []
        : ids.map((id, index) => ({
            id,
            namespace: index < 5 ? svg : html,
            classNameType: index < 5 ? "[object SVGAnimatedString]" : "[object String]",
            classAttribute: classes[phase],
            classProperty: classes[phase],
            styleAttribute: phase < 2 ? `color: ${colors[phase]};` : null,
            styleColor: phase < 2 ? colors[phase] : "",
            computedColor: phase < 2 ? colors[phase] : "inherited default",
          })),
  }));
  assertNamespaceTrace(row, trace);
  for (const mutation of [
    "phase",
    "tree",
    "namespace",
    "svg-type",
    "html-type",
    "class-property",
    "update-class",
    "update-color",
    "clear-attribute",
    "clear-property",
    "binding",
    "unmount",
  ]) {
    const bad = structuredClone(trace);
    if (mutation === "phase") bad.pop();
    if (mutation === "tree") bad[1].tree = ["shortened"];
    if (mutation === "namespace") bad[1].namespaces[10] = svg;
    if (mutation === "svg-type") bad[0].bindings![0].classNameType = "[object String]";
    if (mutation === "html-type") bad[0].bindings![5].classNameType = "[object SVGAnimatedString]";
    if (mutation === "class-property") bad[0].bindings![0].classProperty = "";
    if (mutation === "update-class") bad[1].bindings![0].classAttribute = classes[0];
    if (mutation === "update-color") bad[1].bindings![0].computedColor = colors[0];
    if (mutation === "clear-attribute") bad[2].bindings![0].styleAttribute = "";
    if (mutation === "clear-property") bad[2].bindings![0].styleColor = colors[1];
    if (mutation === "binding") bad[0].bindings!.pop();
    if (mutation === "unmount") bad[3].bindings = bad[2].bindings;
    assert.throws(() => assertNamespaceTrace(row, bad), mutation);
  }
});

test("namespace workflow uses a separate exact selector and guards available runner contexts", () => {
  const workflow = parse(
    fs.readFileSync(
      new URL("../../.github/workflows/release-vapor-namespace-acceptance.yml", import.meta.url),
      "utf8",
    ),
  );
  const job = workflow.jobs["public-browser"];
  assert.equal(job.defaults.run.shell, "bash");
  for (const value of Object.values(job.env))
    assert.doesNotMatch(String(value), /\$\{\{[^}]*\brunner[.[]/u);
  for (const step of job.steps.filter((entry: { run?: string }) => entry.run))
    assert.ok(!step.shell || step.shell === "bash");
  const prepare = job.steps.find((entry: { run?: string }) => entry.run?.includes("prepare.ts"));
  assert.match(prepare.run, /prepare\.ts svg-namespace 2>&1 \| tee/u);
  const guard = job.steps.find(
    (entry: { name?: string }) =>
      entry.name === "Require reviewed main and absent runtime overrides",
  );
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-namespace-workflow-law-"));
  try {
    const environment = path.join(root, "env");
    fs.writeFileSync(environment, "");
    const env = {
      PATH: process.env.PATH,
      GITHUB_REPOSITORY: "ubugeeei-prod/vize",
      GITHUB_REF: "refs/heads/main",
      RUNNER_TEMP: root,
      GITHUB_RUN_ID: "321",
      GITHUB_RUN_ATTEMPT: "2",
      GITHUB_ENV: environment,
    };
    const accepted = spawnSync("bash", ["-eo", "pipefail", "-c", guard.run], {
      env,
      encoding: "utf8",
    });
    assert.equal(accepted.status, 0, accepted.stderr);
    assert.equal(
      fs.readFileSync(environment, "utf8"),
      `PUBLIC_OUTPUT=${root}/vize-public-svg-namespace-evidence-321-2\n`,
    );
    fs.writeFileSync(environment, "");
    const refused = spawnSync("bash", ["-eo", "pipefail", "-c", guard.run], {
      env: { ...env, VIZE_BINDING_PATH: "override.node" },
      encoding: "utf8",
    });
    assert.equal(refused.status, 1);
    assert.equal(fs.readFileSync(environment, "utf8"), "");
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});
