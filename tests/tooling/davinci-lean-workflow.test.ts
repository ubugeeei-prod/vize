import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const formalRoot = path.join(repoRoot, "formal", "impeto");

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
  on: { schedule: Array<{ cron: string }>; workflow_dispatch: unknown };
  jobs: Record<string, { steps: Step[]; if?: unknown; "continue-on-error"?: unknown }>;
};

const proofEscapes = ["sorry", "admit", "axiom", "native_decide"];
const proofEscapeScan = [
  "status=0",
  `grep -rnwE --include='*.lean' --exclude-dir=.lake '${proofEscapes.join("|")}' . || status=$?`,
  'test "$status" -eq 1',
].join("\n");

function leanSources(root: string = formalRoot): string[] {
  return fs.readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const absolute = path.join(root, entry.name);
    if (entry.isDirectory()) return entry.name === ".lake" ? [] : leanSources(absolute);
    return entry.name.endsWith(".lean") ? [absolute] : [];
  });
}

function assertLeanWorkflow(workflow: Workflow): void {
  assert.equal(Object.hasOwn(workflow.on, "push"), false);
  assert.deepEqual(workflow.on.schedule, [{ cron: "11 5 * * *" }]);
  assert.ok(Object.hasOwn(workflow.on, "workflow_dispatch"));
  assert.ok(!Object.hasOwn(workflow.on, "pull_request"));
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
      ["formal/impeto", proofEscapeScan],
      ["formal/impeto", "lake exe impetoRef --check-lattice-fixtures"],
      ["formal/impeto", "lake exe impetoRef --check-schedule-fixtures"],
      ["formal/impeto", "lake exe impetoRef --check-folios"],
      [".", "cargo test -p vize_impeto --test lattice_reference_fixture"],
      [".", "cargo test -p vize_s2_to_s3 --test lean_reference_fixture"],
      [".", "cargo test -p vize_atelier_vapor --test davinci_s3_compiled_trace"],
      [".", "cargo test -p vize_atelier_vapor --test davinci_mounted_behavior"],
      ["tests", "vize-ci-apt-retry vp exec playwright install --with-deps chromium"],
      [
        ".",
        "cargo test -p vize_atelier_vapor --test davinci_event_handlers -- --ignored --nocapture",
      ],
    ],
  );
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

test("TS-30 Chromium contract cannot silently skip its ignored Rust test", () => {
  const source = readRepoFile(".github", "workflows", "davinci-lean.yml");
  const command =
    "cargo test -p vize_atelier_vapor --test davinci_event_handlers -- --ignored --nocapture";
  const skipped = source.replace(command, command.replace(" -- --ignored --nocapture", ""));
  assert.throws(() => assertLeanWorkflow(parseYaml(skipped)), assert.AssertionError);
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

test("P3-15 theorems are audited and the lattice differential is wired", () => {
  const sources = leanSources();
  assert.ok(sources.length >= 10, "the Lean package scan must not be vacuous");
  for (const file of sources) {
    const text = fs.readFileSync(file, "utf8");
    for (const escape of proofEscapes) {
      assert.doesNotMatch(
        text,
        new RegExp(`(^|[^A-Za-z0-9_])${escape}(?![A-Za-z0-9_])`, "u"),
        `${path.relative(repoRoot, file)} uses ${escape}`,
      );
    }
  }
  const theorems = readRepoFile("formal", "impeto", "Impeto", "Theorems.lean");
  assert.match(theorems, /^import Impeto\.LatticeLaws$/mu);
  assert.match(theorems, /^import Impeto\.ScheduleLaws$/mu);
  assert.match(theorems, /^import Impeto\.IncrementalLaws$/mu);
  assert.match(theorems, /^#audit_impeto_theorems \d+$/mu);
  const main = readRepoFile("formal", "impeto", "Main.lean");
  assert.match(main, /^import Impeto\.Theorems$/mu);
  assert.match(
    main,
    /"--check-lattice-fixtures"\] => LatticeFixture\.checkFile "fixtures\/reactivity-lattice\.folio"/u,
  );
  const laws = readRepoFile("formal", "impeto", "Impeto", "LatticeLaws.lean");
  for (const name of [
    "join_le_iff",
    "effectsFloor_eq_tier",
    "classify_spec",
    "classify_monotone",
    "classify_verdict_orthogonal",
    "provideInject_capped",
  ]) {
    assert.match(laws, new RegExp(`^theorem ${name}\\b`, "mu"), `missing theorem ${name}`);
  }
  const schedule = readRepoFile("formal", "impeto", "Impeto", "ScheduleLaws.lean");
  for (const name of [
    "accepted_schedule_orders_edges",
    "accepted_edges_stay_in_scope",
    "accepted_regrouping_preserves_edges",
  ]) {
    assert.match(schedule, new RegExp(`^theorem ${name}\\b`, "mu"), `missing theorem ${name}`);
  }
  assert.match(main, /"--check-schedule-fixtures"\] => ScheduleFixture\.check/u);
  assert.match(main, /"--check-folios"\] => CheckerTests\.check/u);
  const checker = readRepoFile("formal", "impeto", "Impeto", "Checker.lean");
  assert.doesNotMatch(checker, /\bpartial\b|\bunsafe\b/u, "the folio checker must stay total");
  assert.match(
    readRepoFile("formal", "impeto", "Impeto", "CheckerLaws.lean"),
    /^theorem violations_nil_iff\b/mu,
  );
  const incremental = readRepoFile("formal", "impeto", "Impeto", "IncrementalLaws.lean");
  for (const name of [
    "reconcile_erase",
    "update_tree",
    "siblings_identity",
    "reconcile_allocations",
    "update_without_insertions",
  ]) {
    assert.match(incremental, new RegExp(`^theorem ${name}\\b`, "mu"), `missing theorem ${name}`);
  }
  assert.match(
    readRepoFile("crates", "vize_s2_to_s3", "tests", "lean_reference_fixture.rs"),
    /^    mod schedule;$/mu,
  );
  const bridge = readRepoFile("crates", "vize_impeto", "tests", "lattice_reference_fixture.rs");
  assert.match(bridge, /formal\/impeto\/fixtures\/reactivity-lattice\.folio/u);
  assert.match(bridge, /ReactivityFolio::of\(&facts\)\.print_to_string\(FolioMode::Full\)/u);
});
