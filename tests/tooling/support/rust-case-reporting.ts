import assert from "node:assert/strict";
import type { DailySourceStep } from "./github-workflows.ts";

export type RustJob = {
  if: string;
  needs: string[];
  env: Record<string, string>;
  steps: Array<DailySourceStep & Record<string, unknown>>;
};

export function restoreRustCaseReporting(job: RustJob, contracts: number) {
  assert.equal(job.env.TEST_CASES_JOB, "Clippy, doctests and feature lanes (${{ matrix.lane }})");
  assert.equal(job.env.VIZE_TEST_CASES, "test-cases-rust-source-${{ matrix.lane }}.json");
  delete job.env.TEST_CASES_JOB;
  delete job.env.VIZE_TEST_CASES;

  const contract = job.steps[contracts];
  const reporter =
    ' --test-reporter=spec --test-reporter-destination=stdout --test-reporter=./tools/support/compat/github/test-case-reporter.ts --test-reporter-destination="$VIZE_TEST_CASES"';
  assert.equal(contract.id, "contract-cases");
  assert.ok(contract.run?.includes(reporter));
  delete contract.id;
  contract.run = contract.run?.replace(reporter, "");

  const named = (name: string) => {
    const index = job.steps.findIndex((step) => step.name === name);
    assert.notEqual(index, -1, name);
    return { index, step: job.steps[index] };
  };
  for (const name of ["Clippy affected crates", "Run affected Rust doctests"]) {
    const { step } = named(name);
    assert.ok(step.env?.VIZE_RUST_CASE_LABEL);
    delete step.env;
  }
  const ratchet = named("Clippy test targets of touched ratchet crates").step;
  assert.match(ratchet.run ?? "", /rust-test-cases\.ts capture/);
  assert.ok(
    ratchet.run?.includes(
      'node tools/support/compat/github/clippy-test-targets.ts touched "$RUNNER_TEMP/rust-plan.json"',
    ),
  );
  ratchet.run =
    'node tools/support/compat/github/clippy-test-targets.ts touched "$RUNNER_TEMP/rust-plan.json"';
  for (const [name, run] of [
    [
      "Native L1 markup parity",
      "cargo test -p vize_armature --features native-lex-parity,legacy --lib --test markup_lex_parity",
    ],
    [
      "Test shared legacy transform capabilities",
      "cargo test --locked --profile ci -p vize_atelier_core --features legacy --lib legacy -- --nocapture",
    ],
    [
      "Test L1 feature-enabled surface corpus",
      "cargo test -p vize_l1 --features legacy-differential --test surface_corpus -- --nocapture",
    ],
  ] as const) {
    const step = named(name).step;
    assert.equal(step["continue-on-error"], undefined);
    assert.match(step.run ?? "", /rust-test-cases\.ts capture/);
    assert.ok(step.run?.includes(run));
    step.run = run;
  }
  const navigation = named("Test affected native Program and Vue navigation").step;
  assert.equal(navigation.id, "navigation");
  delete navigation.id;
  job.steps.splice(named("Record native navigation case").index, 1);
  for (const name of ["Require Rust source test cases", "Upload Rust source test cases"])
    job.steps.splice(named(name).index, 1);
  const glyphDocs = named("Document Glyph public APIs");
  assert.equal(
    glyphDocs.step.if,
    "${{ matrix.lane == 'clippy' && contains(fromJSON(inputs.rust-plan).packages, 'vize_glyph') }}",
  );
  assert.match(
    glyphDocs.step.run ?? "",
    /rust-test-cases\.ts capture[\s\S]*cargo rustdoc --locked -p vize_glyph --lib -- -D missing-docs/,
  );
  job.steps.splice(glyphDocs.index, 1);
  const patinaDocs = named("Document Patina public APIs");
  assert.equal(
    patinaDocs.step.if,
    "${{ matrix.lane == 'clippy' && contains(fromJSON(inputs.rust-plan).packages, 'vize_patina') }}",
  );
  assert.match(
    patinaDocs.step.run ?? "",
    /rust-test-cases\.ts capture[\s\S]*cargo rustdoc --locked -p vize_patina --lib -- -D warnings/,
  );
  job.steps.splice(patinaDocs.index, 1);
}
