import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

// P3-17: the production-reach gate rides the required clippy-and-test job,
// its floors live in reach-budgets.toml [reach] for exactly the measured shapes,
// and the plan record names every shape the Rust gate measures.
const reachCommand =
  "cargo test -p vize_atelier_sfc --features davinci-dom-differential --test davinci_production_reach -- --nocapture";
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
  const clippyJob = workflowJobBody(workflow, "clippy-and-test");
  const manifest = readRepoFile("crates", "vize_atelier_sfc", "Cargo.toml");
  const budgets = readRepoFile("davinci-road", "plan", "reach-budgets.toml");
  const record = readRepoFile("davinci-road", "plan", "phase-3-records", "p3-17.md");
  const shapesSource = readRepoFile(
    "crates",
    "vize_atelier_sfc",
    "tests",
    "davinci_production_reach",
    "shapes.rs",
  );

  assert.ok(clippyJob.includes(reachCommand), "the reach gate must run in clippy-and-test");
  assert.match(
    manifest,
    /^\[\[test\]\]\nname = "davinci_production_reach"\nrequired-features = \["davinci-dom-differential"\]$/mu,
  );
  assert.match(
    manifest,
    /^davinci-dom-differential = \["vize_atelier_dom\/davinci-differential"\]$/mu,
  );

  const section = reachSection(budgets);
  const entries = [
    ...section.matchAll(
      /^(?<id>[a-z_]+) = \{ accepted_min = (?<accepted>\d+), templates_min = (?<templates>\d+) \}$/gmu,
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
