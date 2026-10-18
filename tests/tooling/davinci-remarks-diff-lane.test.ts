import assert from "node:assert/strict";
import { test } from "node:test";
import { differentialSteps } from "../../tools/support/compat/github/rust-differential-lanes.ts";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

// TS-32 (P3-13): the corpus remarks-diff rides the required rust-native-differential
// job beside the sibling feature-gated L1-to-L2 corpus lanes, and the plan's
// suite row names exactly the command CI runs.
test("the TS-32 remarks-diff lane is wired and documented", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const nativeJob = workflowJobBody(workflow, "rust-native-differential");
  const recipe = readRepoFile(
    ".github",
    "actions",
    "test-rust-workspace-differential",
    "action.yml",
  );
  const manifest = readRepoFile("davinci", "vize_l1_to_l2", "Cargo.toml");
  const suites = readRepoFile("docs/davinci", "plan", "test-suites.md");
  const baseline = readRepoFile("tests", "_fixtures", "davinci-remarks-baseline.folio");

  assert.match(
    manifest,
    /^\[\[test\]\]\nname = "davinci_remarks_corpus"\nrequired-features = \["legacy-differential"\]$/m,
  );
  assert.match(nativeJob, /uses: \.\/\.github\/actions\/test-rust-workspace-differential/);
  assert.match(
    recipe,
    /run: node tools\/support\/compat\/github\/rust-differential-lanes\.ts --all/,
  );
  const lowering = differentialSteps("lowering", ["vize_l1_to_l2"]);
  assert.ok(
    lowering.some(
      (step) =>
        step.args.includes("davinci_remarks_corpus") && step.args.includes("legacy-differential"),
    ),
    "the TS-32 remarks corpus must run explicitly after cargo test --workspace",
  );
  assert.match(
    suites,
    /^\| TS-32 \| Remarks diff\s+\| `cargo test -p vize_l1_to_l2 --features legacy-differential --test davinci_remarks_corpus`: /m,
  );
  assert.match(suites, /`tests\/_fixtures\/davinci-remarks-baseline\.folio`/u);
  assert.match(baseline, /^\[remarks-corpus\]\n\n\[remarks-corpus\.files\]\n/u);
});
