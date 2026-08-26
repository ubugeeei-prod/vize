import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const ts20LoweringCorpusCommand =
  "cargo test -p vize_ricalco --features davinci-differential --test davinci_lowering_corpus -- --nocapture";

test("TS-20 lowering corpus lane rides the required clippy-and-test job", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const clippyJob = workflowJobBody(workflow, "clippy-and-test");
  const testReportJob = workflowJobBody(workflow, "test-report");
  const manifest = readRepoFile("crates", "vize_ricalco", "Cargo.toml");
  const suites = readRepoFile("davinci-road", "plan", "test-suites.md");

  assert.match(suites, /\| TS-20 \| Lowering totality fuzz\s+\| `cargo test -p vize_ricalco`/);
  assert.match(
    manifest,
    /^\[\[test\]\]\nname = "davinci_lowering_corpus"\nrequired-features = \["davinci-differential"\]$/m,
  );

  assert.doesNotMatch(
    clippyJob,
    /^ {4}if:/m,
    "clippy-and-test must stay unconditional so TS-20 runs on pull requests",
  );
  assert.match(testReportJob, /- clippy-and-test\b/);
  assert.match(clippyJob, /run: cargo test --workspace && /);
  assert.ok(
    clippyJob.includes(ts20LoweringCorpusCommand),
    "the feature-gated TS-20 corpus entry must run explicitly after cargo test --workspace",
  );
});
