import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";
import { traceCompiledBackend } from "./support/davinci-runtime-trace.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const formalRoot = path.join(repoRoot, "formal", "impeto");
const fixtureRoot = path.join(formalRoot, "fixtures");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

type Step = {
  uses?: string;
  run?: string;
  if?: unknown;
  "continue-on-error"?: unknown;
  "working-directory"?: string;
  with?: Record<string, unknown>;
};
type Workflow = {
  on: Record<"push" | "pull_request", { paths: string[] }>;
  jobs: Record<string, { steps: Step[]; if?: unknown; "continue-on-error"?: unknown }>;
};

function assertLeanWorkflow(workflow: Workflow): void {
  const job = workflow.jobs["impeto-reference"];
  assert.ok(job);
  assert.equal(job.if, undefined);
  assert.ok(job["continue-on-error"] === undefined || job["continue-on-error"] === false);
  const lean = job.steps.find((step) => step.uses?.startsWith("leanprover/lean-action@"));
  assert.equal(lean?.uses, "leanprover/lean-action@50fcf42d2e460296f1a34b402e990d1b24f8b596");
  assert.equal(lean?.with?.["lake-package-directory"], "formal/impeto");
  assert.equal(lean?.with?.build, "true");
  const vp = job.steps.find((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"));
  assert.equal(vp?.uses, "voidzero-dev/setup-vp@ca1c46663915d6c1042ae23bd39ab85718bfb0fa");
  assert.equal(vp?.with?.["node-version-file"], "package.json");
  assert.equal(vp?.with?.["run-install"], false);
  for (const step of job.steps) {
    assert.equal(step.if, undefined);
    assert.ok(step["continue-on-error"] === undefined || step["continue-on-error"] === false);
  }
  assert.deepEqual(
    job.steps.flatMap((step) =>
      step.run ? [[step["working-directory"] ?? ".", step.run.trim()]] : [],
    ),
    [
      [".", "vp install --frozen-lockfile --prefer-offline"],
      ["formal/impeto", "lake exe impetoRef --check-fixtures"],
      ["formal/impeto", "lake exe impetoRef --check-backend-fixtures"],
      ["formal/impeto", "lake exe impetoRef --check-stateful-fixtures"],
      [".", "cargo test -p vize_s2_to_s3 --test lean_reference_fixture"],
      [".", "cargo test -p vize_atelier_vapor --test davinci_s3_compiled_trace"],
      [".", "cargo test -p vize_atelier_vapor --test davinci_mounted_behavior"],
    ],
  );
  for (const event of ["push", "pull_request"] as const) {
    for (const file of ["davinci-runtime-trace.mjs", "davinci-mounted-trace.mjs"]) {
      assert.ok(workflow.on[event].paths.includes(`tests/tooling/support/${file}`));
    }
  }
}

test("TS-28 pins the Lean toolchain and CI package directory", () => {
  assert.equal(
    readRepoFile("formal", "impeto", "lean-toolchain").trim(),
    "leanprover/lean4:v4.33.1",
  );

  assertLeanWorkflow(parseYaml(readRepoFile(".github", "workflows", "davinci-lean.yml")));
});

test("TS-28 rejects commented, moved and disabled workflow commands", () => {
  const source = readRepoFile(".github", "workflows", "davinci-lean.yml");
  const command = "lake exe impetoRef --check-stateful-fixtures";
  const commented = source.replace(`run: ${command}`, `run: "true" # ${command}`);
  assert.throws(() => assertLeanWorkflow(parseYaml(commented)), assert.AssertionError);
  const moved: Workflow = parseYaml(source);
  const steps = moved.jobs["impeto-reference"].steps;
  const index = steps.findIndex((step) => step.run === command);
  assert.notEqual(index, -1);
  moved.jobs.unrelated = { steps: steps.splice(index, 1) };
  assert.throws(() => assertLeanWorkflow(moved), assert.AssertionError);
  for (const field of ["if", "continue-on-error"] as const) {
    const disabled: Workflow = parseYaml(source);
    disabled.jobs["impeto-reference"].steps.find((step) => step.run === command)![field] =
      field !== "if";
    assert.throws(() => assertLeanWorkflow(disabled), assert.AssertionError);
  }
});

test("TS-28 rejects failure-ignore settings at job and step scopes", () => {
  const source = readRepoFile(".github", "workflows", "davinci-lean.yml");
  for (const scope of ["job", "step"] as const) {
    for (const value of [true, "${{ true }}", "${{ false }}", "false", null]) {
      const workflow: Workflow = parseYaml(source);
      const job = workflow.jobs["impeto-reference"];
      const target = scope === "job" ? job : job.steps.find((step) => step.run)!;
      target["continue-on-error"] = value;
      assert.throws(() => assertLeanWorkflow(workflow), assert.AssertionError);
    }
  }
  const explicitFalse: Workflow = parseYaml(source);
  const job = explicitFalse.jobs["impeto-reference"];
  job["continue-on-error"] = false;
  for (const step of job.steps) step["continue-on-error"] = false;
  assertLeanWorkflow(explicitFalse);
});

test("TS-28 fixture ladder is declared and non-vacuous", () => {
  const main = readRepoFile("formal", "impeto", "Main.lean");
  const folios = fs
    .readdirSync(fixtureRoot)
    .filter((file) => file.endsWith(".s3.folio"))
    .sort();

  assert.ok(folios.length >= 3, "TS-28 must start with a non-vacuous fixture ladder");
  assert.ok(
    folios.some((folio) => folio.startsWith("rust-lowered-")),
    "TS-28 must include at least one fixture emitted by the Rust S2->S3 lowering path",
  );
  for (const folio of folios) {
    const stem = folio.replace(/\.s3\.folio$/u, "");
    const traces = [`${stem}.trace`, `${stem}.vdom.trace`, `${stem}.vapor.trace`];
    for (const trace of traces) {
      assert.ok(fs.existsSync(path.join(fixtureRoot, trace)), `${trace} is missing`);
      assert.match(main, new RegExp(`fixtures/${trace.replaceAll(".", "\\.")}`, "u"));
    }
    assert.match(main, new RegExp(`fixtures/${folio.replaceAll(".", "\\.")}`, "u"));
  }
  assert.match(main, /--check-backend-fixtures/u);
  assert.match(main, /--trace-vdom/u);
  assert.match(main, /--trace-vapor/u);
  assert.match(main, /--check-stateful-fixtures/u);
});

test("TS-28 Rust lowering bridge is covered by an ordinary cargo test", () => {
  const bridge = readRepoFile("crates", "vize_s2_to_s3", "tests", "lean_reference_fixture.rs");
  assert.match(bridge, /rust_lowered_fixtures_match_impeto_reference_inputs/u);
  assert.match(bridge, /formal\/impeto\/fixtures\/rust-lowered-static-dynamic\.s3\.folio/u);
  assert.match(bridge, /formal\/impeto\/fixtures\/rust-lowered-control-slots\.s3\.folio/u);
  assert.match(bridge, /S3Folio::of\(&lowered\.program\)\.print_to_string\(FolioMode::Full\)/u);
  assert.match(bridge, /reference_trace_text\(&lowered\.program\)/u);
  assert.match(bridge, /backend_trace_text\(TraceBackend::Vdom, &lowered\.program\)/u);
  assert.match(bridge, /backend_trace_text\(TraceBackend::Vapor, &lowered\.program\)/u);
  assert.match(
    bridge,
    /S3ValuesFolio::of\(&lowered\.program\)\.print_to_string\(FolioMode::Full\)/u,
  );
  assert.match(bridge, /rust-lowered-static-dynamic\.values\.folio/u);
});

test("TS-28 stateful reference and mounted backends share full observations and steps", () => {
  const stem = "rust-lowered-static-dynamic";
  const scenario = JSON.parse(
    fs.readFileSync(path.join(fixtureRoot, `${stem}.scenario.json`), "utf8"),
  );
  const trace = JSON.parse(
    fs.readFileSync(path.join(fixtureRoot, `${stem}.behavior.json`), "utf8"),
  );
  assert.equal(trace.length, scenario.steps.length + 2);
  assert.deepEqual(trace[0].events, []);
  assert.deepEqual(trace.at(-1), { tree: [], events: ["save", "save"] });
  assert.deepEqual(
    trace.slice(0, -1).map((snapshot) => snapshot.tree[0].children[0].disabled),
    [true, true, false, false, false, false, false, false, false, true, true],
  );
  assert.deepEqual(
    trace.slice(0, -1).map((snapshot) => snapshot.tree[0].children[0].children),
    [
      ["Save"],
      ["Save"],
      ["Publish"],
      ["Publish"],
      [],
      ["42"],
      ["true"],
      ['\u96ea\n"ready"'],
      ['\u96ea\n"ready"'],
      ["Saved"],
      ["Saved"],
    ],
  );
  const mounted = readRepoFile(
    "crates",
    "vize_atelier_vapor",
    "tests",
    "davinci_mounted_behavior.rs",
  );
  assert.match(mounted, /rust-lowered-static-dynamic\.scenario\.json/u);
  assert.match(mounted, /rust-lowered-static-dynamic\.behavior\.json/u);
  assert.match(mounted, /assert_eq!\(\s*trace, expected,/u);
  const main = readRepoFile("formal", "impeto", "Main.lean");
  assert.match(main, /BehaviorTests\.check/u);
  assert.match(main, /Behavior\.check "fixtures\/rust-lowered-static-dynamic"/u);
});

test("TS-28 compiled backend trace gate executes both emitted backends", async () => {
  const gate = readRepoFile(
    "crates",
    "vize_atelier_vapor",
    "tests",
    "davinci_s3_compiled_trace.rs",
  );
  assert.match(gate, /compiled_backend_runtime_traces_match_s3_reference_ladder/u);
  assert.match(gate, /runtime_backend_trace/u);
  assert.match(gate, /compile_template_with_options/u);
  assert.match(gate, /compile_vapor/u);
  assert.match(gate, /davinci-runtime-trace\.mjs/u);

  assert.deepEqual(
    await traceCompiledBackend({
      backend: "vdom",
      code: `
        import { createElementBlock, openBlock, toDisplayString } from "vue";
        export function render(_ctx, _cache) {
          return (
            openBlock(),
            createElementBlock("p", { id: "msg", textContent: toDisplayString(label) }, null, 9, ["id", "textContent"])
          );
        }
      `,
      context: { label: "ready" },
    }),
    ["create-element", "patch-prop", "set-text"],
  );
  assert.deepEqual(
    await traceCompiledBackend({
      backend: "vapor",
      code: `
        import { child, renderEffect, setText, template } from "vue";
        const t0 = template("<p></p>");
        export function render(_ctx) {
          const n0 = t0();
          const c0 = child(n0, 0);
          renderEffect(() => setText(c0, label));
          return n0;
        }
      `,
      context: { label: "ready" },
    }),
    ["create-node", "text-effect"],
  );
  await assert.rejects(
    traceCompiledBackend({
      backend: "vapor",
      code: `
        import { createForStatic } from "vue";
        export function render(_ctx) {
          return createForStatic(1, () => {});
        }
      `,
    }),
    /unsupported vue runtime helper: createForStatic/u,
  );
  assert.match(gate, /STATIC_DYNAMIC_VAPOR_COMPILED_KNOWN_GAP/u);
  assert.doesNotMatch(gate, /CONTROL_SLOTS_VAPOR_COMPILED_KNOWN_GAP/u);
  assert.match(gate, /rust-lowered-static-dynamic\.vdom\.trace/u);
  assert.match(gate, /rust-lowered-static-dynamic\.vapor\.trace/u);
  assert.match(gate, /rust-lowered-control-slots\.vdom\.trace/u);
  assert.match(gate, /rust-lowered-control-slots\.vapor\.trace/u);
});

test("TS-28 command in the suite registry names the executable runner", () => {
  const suites = readRepoFile("davinci-road", "plan", "test-suites.md");
  const rows = suites
    .split("\n")
    .filter((line) => line.startsWith("|"))
    .map((line) =>
      line
        .split("|")
        .slice(1, -1)
        .map((cell) => cell.trim()),
    )
    .filter((cells) => cells[0] === "TS-28");
  assert.deepEqual(rows, [
    [
      "TS-28",
      "Lean reference differential",
      "`cd formal/impeto && lake exe impetoRef --check-fixtures && lake exe impetoRef --check-backend-fixtures && lake exe impetoRef --check-stateful-fixtures && cd ../.. && cargo test -p vize_s2_to_s3 --test lean_reference_fixture && cargo test -p vize_atelier_vapor --test davinci_s3_compiled_trace --test davinci_mounted_behavior`",
      "exact agreement on observable semantics; stateful subset and remaining operation-order gaps are explicit",
      "P3-4",
    ],
  ]);
});
