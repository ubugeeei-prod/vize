import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  readJson,
  root,
  run,
  setup,
  unusableFailure,
  updateJson,
  writeJson,
  writeVueTsc,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

test("typecheck divergence report fails closed on mismatched matrix artifacts", () => {
  const fixture = setup();
  try {
    const payloadPath = path.join(fixture.reportDir, "fixture-typechecker.json");
    updateJson(payloadPath, (payload) => (payload.project = "wrong-project"));
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /artifact identity is invalid/);
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
      false,
    );
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects missing or mismatched preparation evidence", () => {
  for (const [mutate, message] of [
    [
      (fixture: ReturnType<typeof setup>) =>
        fs.rmSync(path.join(fixture.reportDir, "fixture-typecheck-dependencies.json")),
      /Missing typecheck dependency preparation evidence/,
    ],
    [
      (fixture: ReturnType<typeof setup>) =>
        updateJson(
          path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
          (artifact) => (artifact.evidence.commitSha = "c".repeat(40)),
        ),
      /preparation identity is invalid/,
    ],
    [
      (fixture: ReturnType<typeof setup>) =>
        updateJson(
          path.join(fixture.reportDir, "fixture-typecheck-dependencies.json"),
          (artifact) => (artifact.lockfile.sha256 = "0".repeat(64)),
        ),
      /lockfile evidence is invalid/,
    ],
  ] as const) {
    const fixture = setup();
    try {
      mutate(fixture);
      const result = run(fixture);
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
      assert.equal(
        fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
        false,
      );
    } finally {
      cleanup(fixture);
    }
  }
});

test("typecheck divergence report rejects evidence from another commit", () => {
  const fixture = setup();
  try {
    const result = run(fixture, { GITHUB_SHA: "c".repeat(40) });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /commit does not match GITHUB_SHA/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects parsed output that differs from stdout", () => {
  const fixture = setup();
  try {
    const payloadPath = path.join(fixture.reportDir, "fixture-typechecker.json");
    updateJson(payloadPath, (payload) => (payload.parsed.errorCount = 2));
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /stdout does not match parsed output/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects a mismatched matrix file count", () => {
  const fixture = setup();
  try {
    const summaryPath = path.join(fixture.reportDir, "summary.json");
    updateJson(summaryPath, (summary) => (summary.projects[0].runs[0].fileCount = 2));
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /file count is inconsistent/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects mutated raw typechecker coverage", () => {
  const fixture = setup();
  try {
    updateJson(fixture.outputPath, (payload) => {
      payload.typecheckerCoverage.checked.sha256 = "0".repeat(64);
    });
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /typechecker coverage is inconsistent/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects mutated summary coverage", () => {
  const fixture = setup();
  try {
    const summaryPath = path.join(fixture.reportDir, "summary.json");
    updateJson(summaryPath, (summary) => {
      summary.projects[0].runs[0].coverage.requestedFileCount = 0;
    });
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /summary coverage is inconsistent/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects a mismatched matrix status", () => {
  const fixture = setup();
  try {
    const summaryPath = path.join(fixture.reportDir, "summary.json");
    updateJson(summaryPath, (summary) => (summary.projects[0].runs[0].status = "ok"));
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /status is inconsistent/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects an artifact outside the reported directory", () => {
  const fixture = setup();
  try {
    const summaryPath = path.join(fixture.reportDir, "summary.json");
    updateJson(
      summaryPath,
      (summary) =>
        (summary.projects[0].runs[0].outputPath = path.relative(
          root,
          path.join(fixture.reportDir, "nested", "fixture-typechecker.json"),
        )),
    );
    const result = run(fixture);
    assert.equal(result.status, 1);
    assert.match(result.stderr, /output path is invalid/);
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report rejects invalid performance budgets", () => {
  for (const [field, value, message] of [
    ["hangTimeoutMs", 0, /hangTimeoutMs must be a positive safe integer/],
    ["maxFalsePositiveRatio", Number.NaN, /maxFalsePositiveRatio must be a finite number/],
    ["maxFalseNegativeRatio", 1.1, /maxFalseNegativeRatio must be a finite number/],
  ] as const) {
    const fixture = setup();
    try {
      const registry = readJson(fixture.registryPath);
      registry.projects[0].typecheckPerformance[field] = value;
      writeJson(fixture.registryPath, registry);
      const result = run(fixture);
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
    } finally {
      cleanup(fixture);
    }
  }
});

test("typecheck divergence report rejects unsupported baseline exits and output", () => {
  for (const [body, message] of [
    ["process.exit(3);", /unsupported status 3/],
    ["process.stderr.write('prefix error TS1: bad\\n'); process.exit(2);", /unparseable/],
  ] as const) {
    const fixture = setup();
    try {
      writeVueTsc(fixture.vueTsc, body);
      const result = run(fixture);
      assert.equal(result.status, 1);
      assert.match(result.stderr, message);
    } finally {
      cleanup(fixture);
    }
  }
});

test("seeded mutation oracle fails when either checker misses or mismatches the probe", () => {
  for (const [label, options, expected] of [
    ["vize missing", { vizeMutation: "missing" }, { fp: 0, fn: 1, mm: 0 }],
    ["vue-tsc missing", { baselineMutation: "missing" }, { fp: 1, fn: 0, mm: 0 }],
    ["vize mismatch", { vizeMutation: "mismatch" }, { fp: 0, fn: 0, mm: 1 }],
    ["vue-tsc mismatch", { baselineMutation: "mismatch" }, { fp: 0, fn: 0, mm: 1 }],
  ] as const) {
    const fixture = setup(options);
    try {
      const result = run(fixture);
      assert.equal(result.status, 1, label);
      // The instrument-failure classification is behavior, so it stays in the
      // assertion, but matching a substring keeps the test off the exact stream
      // shape (trailing newline, unrelated warnings).
      assert.ok(
        result.stderr.includes(
          unusableFailure(
            "seeded mutation oracle did not produce one shared broken diagnostic and clean repair",
          ),
        ),
        label,
      );
      assert.equal(
        fs.readFileSync(path.join(fixture.fixtureRoot, "src", "App.vue"), "utf8"),
        "<template />\n",
        label,
      );
      const oracle = readJson(
        path.join(fixture.reportDir, "fixture-typecheck-divergence.json"),
      ).mutationOracle;
      assert.equal(oracle.passed, false, label);
      assert.equal(oracle.expectedDiagnosticMatched, false, label);
      assert.equal(oracle.states[1].falsePositiveCount, expected.fp, label);
      assert.equal(oracle.states[1].falseNegativeCount, expected.fn, label);
      assert.equal(oracle.states[1].messageMismatchCount, expected.mm, label);
    } finally {
      cleanup(fixture);
    }
  }
});

test("typecheck divergence report skips shards with no registered performance projects", () => {
  const fixture = setup();
  try {
    fs.writeFileSync(fixture.registryPath, '{"projects":[]}\n');
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /No typecheck performance projects selected/);
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
      false,
    );
  } finally {
    cleanup(fixture);
  }
});

test("typecheck divergence report shards the full fixture registry before filtering typecheck targets", () => {
  const fixture = setup();
  try {
    updateJson(fixture.registryPath, (registry) => {
      registry.projects = [
        { ...registry.projects[0], id: "padding", typecheckPerformance: { enabled: false } },
        registry.projects[0],
      ];
    });

    const empty = run(fixture, {}, ["--shard-index", "0", "--shard-count", "2"]);
    assert.equal(empty.status, 0, empty.stderr);
    assert.match(empty.stdout, /No typecheck performance projects selected/);
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
      false,
    );

    const selected = run(fixture, {}, ["--shard-index", "1", "--shard-count", "2"]);
    assert.equal(selected.status, 0, selected.stderr);
    assert.equal(
      fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
      true,
    );
  } finally {
    cleanup(fixture);
  }
});
