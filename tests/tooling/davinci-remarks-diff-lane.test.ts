import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

// TS-32 (P3-13): the corpus remarks-diff rides the required clippy-and-test
// job beside the sibling feature-gated S1-to-S2 corpus lanes, and the plan's
// suite row names exactly the command CI runs.
const remarksCorpusCommand =
  "cargo test -p vize_s1_to_s2 --features davinci-differential --test davinci_remarks_corpus -- --nocapture";

test("the TS-32 remarks-diff lane is wired and documented", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const clippyJob = workflowJobBody(workflow, "clippy-and-test");
  const manifest = readRepoFile("crates", "vize_s1_to_s2", "Cargo.toml");
  const suites = readRepoFile("docs/davinci", "plan", "test-suites.md");
  const baseline = readRepoFile("tests", "_fixtures", "davinci-remarks-baseline.folio");

  assert.match(
    manifest,
    /^\[\[test\]\]\nname = "davinci_remarks_corpus"\nrequired-features = \["davinci-differential"\]$/m,
  );
  assert.ok(
    clippyJob.includes(remarksCorpusCommand),
    "the TS-32 remarks corpus must run explicitly after cargo test --workspace",
  );
  assert.match(
    suites,
    /^\| TS-32 \| Remarks diff\s+\| `cargo test -p vize_s1_to_s2 --features davinci-differential --test davinci_remarks_corpus`: /m,
  );
  assert.match(suites, /`tests\/_fixtures\/davinci-remarks-baseline\.folio`/u);
  assert.match(baseline, /^\[remarks-corpus\]\n\n\[remarks-corpus\.files\]\n/u);
});
