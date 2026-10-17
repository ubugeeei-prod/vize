import assert from "node:assert/strict";
import { test } from "node:test";
import { differentialSteps } from "../../tools/support/compat/github/rust-differential-lanes.ts";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

// P3-17: the production-reach gate rides the required rust-native-differential job,
// its floors live in reach-budgets.toml [reach] for exactly the measured shapes,
// and the plan record names every shape the Rust gate measures.
const reachCommand =
  "test --locked -p vize_atelier_sfc --features legacy-dom-differential --test davinci_production_reach -- --nocapture";
const shapes = ["dom_inline", "dom_module", "ssr", "vapor"];

function reachSection(budgets: string): string {
  const start = budgets.indexOf("\n[reach]\n");
  assert.notEqual(start, -1, "reach-budgets.toml must carry a [reach] section");
  const rest = budgets.slice(start + "\n[reach]\n".length);
  const next = rest.search(/^\[/mu);
  return next === -1 ? rest : rest.slice(0, next);
}

test("the P3-17 production-reach gate is wired, budgeted and recorded", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const nativeJob = workflowJobBody(workflow, "rust-native-differential");
  const recipe = readRepoFile(
    ".github",
    "actions",
    "test-rust-workspace-differential",
    "action.yml",
  );
  const manifest = readRepoFile("crates", "vize_atelier_sfc", "Cargo.toml");
  const budgets = readRepoFile("docs/davinci", "plan", "reach-budgets.toml");
  const record = readRepoFile("docs/davinci", "plan", "phase-3-records", "p3-17.md");
  const shapesSource = readRepoFile(
    "crates",
    "vize_atelier_sfc",
    "tests",
    "davinci_production_reach",
    "shapes.rs",
  );

  assert.match(nativeJob, /uses: \.\/\.github\/actions\/test-rust-workspace-differential/);
  assert.match(
    recipe,
    /run: node tools\/support\/compat\/github\/rust-differential-lanes\.ts --all/,
  );
  assert.ok(
    differentialSteps("sfc", ["vize_atelier_sfc"]).some(
      (step) => step.args.join(" ") === reachCommand,
    ),
    "the shared recipe must run the feature-gated reach test",
  );
  assert.match(
    manifest,
    /^\[\[test\]\]\nname = "davinci_production_reach"\nrequired-features = \["legacy-dom-differential"\]$/mu,
  );
  assert.match(
    manifest,
    /^legacy-dom-differential = \["vize_atelier_dom\/legacy-differential"\]$/mu,
  );

  const section = reachSection(budgets);
  const entries = [
    ...section.matchAll(
      /^(?<id>[a-z_]+) = \{ accepted_min = (?<accepted>\d+), ready_min = (?<ready>\d+), templates_min = (?<templates>\d+) \}$/gmu,
    ),
  ];
  assert.deepEqual(
    entries.map((entry) => entry.groups!.id),
    shapes,
    "[reach] must list exactly the measured shapes, in gate order",
  );
  assert.ok(
    budgets
      .split("\n")
      .includes(
        "# ratchet: numbers may only tighten; loosening requires budget-loosen: <charter ref> in the commit body",
      ),
    "reach-budgets.toml must carry the ratchet header",
  );
  for (const entry of entries) {
    assert.ok(Number(entry.groups!.templates) > 0, `${entry.groups!.id} templates_min`);
  }

  for (const shape of shapes) {
    assert.ok(shapesSource.includes(`=> "${shape}"`), `shapes.rs must measure ${shape}`);
    assert.ok(record.includes(`\`${shape}\``), `p3-17.md must report ${shape}`);
  }
});
