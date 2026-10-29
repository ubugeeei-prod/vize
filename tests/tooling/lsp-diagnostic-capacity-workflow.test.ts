import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
  with?: Record<string, unknown>;
};
type Workflow = {
  on: { pull_request: { paths: string[] } };
  jobs: { coverage: { env: Record<string, string>; steps: Step[] } };
};

test("native capacity admission requires real source, nonzero laws, both native cases and retained evidence", () => {
  const workflow = parse(
    readRepoFile(".github/workflows/lsp-native-diagnostic-coverage.yml"),
  ) as Workflow;
  const job = workflow.jobs.coverage;
  assert.equal(job.env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(job.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(job.env.VIZE_CAPACITY_FILES, "129");
  for (const path of [
    "crates/vize_maestro/src/server/initial_diagnostics/**",
    "crates/vize_maestro/src/server/open_document.rs",
    "crates/vize_maestro/src/server/diagnostic_publishing/**",
  ])
    assert.ok(workflow.on.pull_request.paths.includes(path));
  const build = job.steps.find(
    (step) => step.name === "Build the exact source once in its own target",
  );
  assert.ok(build?.run);
  assert.match(build.run, /test ! -e "\$target_dir"/u);
  assert.match(build.run, /export CARGO_TARGET_DIR="\$target_dir"/u);
  assert.match(build.run, /build-receipt[.]ts/u);
  assert.ok(build.run.includes("VIZE_LSP_BIN=$GITHUB_WORKSPACE/target/ci/vize"));
  const laws = job.steps.find(
    (step) => step.name === "Preserve scheduler/version/cancellation laws",
  );
  assert.ok(laws?.run);
  assert.ok(laws.run.includes("server::initial_diagnostics::"));
  assert.ok(laws.run.includes("nonzero scheduler laws with zero ignored tests are required"));
  const native = job.steps.find(
    (step) => step.name === "Observe all native errors, complete repairs and checked idle",
  );
  assert.ok(native?.run);
  assert.ok(native.run.includes("tests/tooling/lsp-diagnostic-capacity.test.ts"));
  assert.ok(native.run.includes("tests/tooling/lsp-diagnostic-capacity-reopen.test.ts"));
  const upload = job.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(upload?.if, "always()");
  const name = upload?.with?.name;
  assert.ok(
    typeof name === "string" &&
      name.includes("github.run_id") &&
      name.includes("github.run_attempt") &&
      name.includes("github.event.pull_request.head.sha"),
  );
  assert.equal(upload?.with?.["if-no-files-found"], "error");
  assert.equal(
    upload?.with?.path,
    "target/ci/vize.differential-build.json\n" +
      "target/vize-tests/diagnostic-capacity-129/\n" +
      "target/differential/lsp-sessions/\n",
  );
});
