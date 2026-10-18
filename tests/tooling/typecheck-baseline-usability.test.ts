import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { before, test } from "node:test";
import {
  cleanup,
  readJson,
  run,
  script,
  setup,
  sha256,
  sharedBaselineOutput,
  writeVueTsc,
} from "./_helpers/typecheck-divergence-report-fixture.ts";
import { writeVueTscFixture } from "./_helpers/typecheck-divergence-checker-fakes.ts";
import { buildRustScript } from "./support/rust-script-build.ts";

const linuxOnly = { skip: process.platform !== "linux" };
before(() => {
  if (process.platform === "linux") buildRustScript(script, ["--rust-script-build-check"]);
});

function executeFixture(fixture: ReturnType<typeof setup>) {
  const result = run(fixture, {}, [], { retainEvidence: true });
  if (result.stdout != null) process.stdout.write(result.stdout);
  if (result.stderr != null) process.stderr.write(result.stderr);
  return result;
}

function commandLabels(fixture: ReturnType<typeof setup>) {
  const roots = fs.readdirSync(path.join(fixture.reportDir, "typecheck-deadlines"));
  assert.equal(roots.length, 1);
  const first = roots[0];
  assert.ok(first);
  const directory = path.join(fixture.reportDir, "typecheck-deadlines", first);
  return fs
    .readdirSync(directory)
    .filter((name) => /^command-\d+\.json$/u.test(name))
    .sort(
      (a, b) =>
        Number(a.slice("command-".length, -".json".length)) -
        Number(b.slice("command-".length, -".json".length)),
    )
    .map((name) => readJson(path.join(directory, name)).label);
}
function fakeBaseline(
  fixture: ReturnType<typeof setup>,
  output: string,
  probe: { output?: string; exitCode?: number },
) {
  const files = [path.join(fixture.fixtureRoot, "src/App.vue")];
  const declaration = path.join(fixture.fixtureRoot, "src/env.d.ts");
  if (fs.existsSync(declaration)) files.push(declaration);
  writeVueTsc(
    fixture.vueTsc,
    `if (process.argv.includes("--listFilesOnly")) {
      process.stdout.write(${JSON.stringify(`${files.join("\n")}\n`)});
      process.exit(0);
    }
    process.stdout.write(${JSON.stringify(output)});
    process.exit(2);`,
    fixture.invocationPath,
    probe,
  );
}

test(
  "a legitimate semantic diagnostic remains the full comparison and mutation baseline",
  linuxOnly,
  () => {
    const fixture = setup();
    try {
      const result = executeFixture(fixture);
      assert.equal(result.status, 0, result.stderr);
      const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
      const observations = readJson(
        path.join(fixture.reportDir, "fixture-typecheck-observations.json"),
      );
      assert.equal(artifact.version, 7);
      assert.equal(artifact.budget.passed, true);
      assert.equal(artifact.divergence.summary.sharedCount, 1);
      assert.equal(artifact.mutationOracle.passed, true);
      assert.deepEqual(
        artifact.mutationOracle.states.map((state: { name: string }) => state.name),
        ["clean", "broken", "repaired"],
      );
      assert.equal(observations.baseline.stdout, sharedBaselineOutput);
      assert.equal(artifact.baseline.stdoutSha256, sha256(sharedBaselineOutput));
      assert.equal(observations.preSemantic.stdout, "");
      assert.equal(observations.preSemantic.exitCode, 0);
      assert.equal(observations.preSemantic.command, `${observations.baseline.command} --noCheck`);
      assert.doesNotMatch(artifact.baseline.command, /--noCheck/u);
      assert.equal(
        fs.existsSync(path.join(fixture.reportDir, "fixture-baseline-unusable.json")),
        false,
      );
    } finally {
      cleanup(fixture);
    }
  },
);

test(
  "pre-semantic failure retains authentic positions and raw evidence before any mutation",
  linuxOnly,
  () => {
    const fixture = setup();
    const original = fs.readFileSync(path.join(fixture.fixtureRoot, "src/App.vue"));
    const diagnostic = "src/env.d.ts(2,1): error TS1128: Declaration or statement expected.\n";
    try {
      fs.writeFileSync(
        path.join(fixture.fixtureRoot, "src/env.d.ts"),
        "declare namespace External {}\n}\n",
      );
      fakeBaseline(fixture, diagnostic, { output: diagnostic, exitCode: 2 });
      const result = executeFixture(fixture);
      assert.equal(result.status, 1);
      assert.match(result.stderr, /instrument failure.*pre-semantic diagnostics/u);
      const packet = readJson(path.join(fixture.reportDir, "fixture-baseline-unusable.json"));
      assert.equal(packet.schema, "vize.fixtureTypecheckBaselineUnusable");
      assert.equal(packet.version, 1);
      assert.equal(packet.verdict, "unusable");
      assert.equal(packet.cause.kind, "pre-semantic-diagnostics");
      assert.deepEqual(packet.diagnostics, [
        {
          file: "src/env.d.ts",
          line: 2,
          column: 1,
          severity: "error",
          code: 1128,
          message: "Declaration or statement expected.",
          coordinateBase: 1,
          end: null,
          continuation: [],
        },
      ]);
      const observationsPath = path.join(fixture.reportDir, packet.observations.file);
      assert.equal(sha256(fs.readFileSync(observationsPath)), packet.observations.sha256);
      const observations = readJson(observationsPath);
      assert.equal(observations.baseline.stdout, diagnostic);
      assert.equal(observations.baseline.stderr, "");
      assert.equal(observations.preSemantic.stdout, diagnostic);
      assert.equal(observations.preSemantic.stderr, "");
      assert.equal(observations.preSemantic.command, `${observations.baseline.command} --noCheck`);
      assert.deepEqual(commandLabels(fixture), [
        "vue-tsc",
        "vue-tsc baseline",
        "vue-tsc coverage baseline",
        "vue-tsc pre-semantic baseline",
      ]);
      assert.deepEqual(fs.readFileSync(path.join(fixture.fixtureRoot, "src/App.vue")), original);
      assert.equal(
        fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
        false,
      );
    } finally {
      cleanup(fixture);
    }
  },
);

test(
  "unsupported options, malformed output and broken probe statuses never become comparisons",
  linuxOnly,
  () => {
    for (const [probe, kind] of [
      [
        { output: "error TS5023: Unknown compiler option '--noCheck'.\n", exitCode: 2 },
        "pre-semantic-diagnostics",
      ],
      [{ output: "not a TypeScript diagnostic\n", exitCode: 2 }, "malformed-provider-output"],
      [{ output: "", exitCode: 3 }, "provider-process-failure"],
      [{ output: "", exitCode: 2 }, "missing-provider-diagnostics"],
      [
        { output: "src/env.d.ts(2,1): error TS1128: Unexpected token.\n", exitCode: 0 },
        "provider-status-disagreement",
      ],
    ] as const) {
      const fixture = setup();
      try {
        fakeBaseline(fixture, sharedBaselineOutput, probe);
        const result = executeFixture(fixture);
        assert.equal(result.status, 1);
        const packet = readJson(path.join(fixture.reportDir, "fixture-baseline-unusable.json"));
        assert.equal(packet.cause.kind, kind);
        assert.equal(packet.verdict, "unusable");
        assert.equal(
          fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
          false,
        );
        assert.equal(commandLabels(fixture).includes("Vize mutation run"), false);
      } finally {
        cleanup(fixture);
      }
    }
  },
);

test(
  "already unusable configuration keeps its original failed report without another probe",
  linuxOnly,
  () => {
    const fixture = setup({ baselineOutput: "error TS5058: The specified path does not exist.\n" });
    try {
      const result = executeFixture(fixture);
      assert.equal(result.status, 1);
      const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
      assert.equal(artifact.budget.verdict, "unusable");
      assert.equal(artifact.baseline.configuration.errorCount, 1);
      assert.equal(commandLabels(fixture).includes("vue-tsc pre-semantic baseline"), false);
      assert.equal(
        fs.existsSync(path.join(fixture.reportDir, "fixture-baseline-unusable.json")),
        false,
      );
    } finally {
      cleanup(fixture);
    }
  },
);

test(
  "unrecognized diagnostic exits fail as provider evidence, not as source syntax",
  linuxOnly,
  () => {
    for (const [output, kind] of [
      ["", "missing-full-baseline-diagnostics"],
      ["provider failed without a TypeScript diagnostic\n", "malformed-full-baseline-output"],
    ]) {
      const fixture = setup();
      try {
        fakeBaseline(fixture, output!, { output: "", exitCode: 0 });
        const result = executeFixture(fixture);
        assert.equal(result.status, 1);
        const packet = readJson(path.join(fixture.reportDir, "fixture-baseline-unusable.json"));
        assert.equal(packet.cause.kind, kind);
        assert.deepEqual(packet.diagnostics, []);
        assert.deepEqual(commandLabels(fixture), [
          "vue-tsc",
          "vue-tsc baseline",
          "vue-tsc coverage baseline",
        ]);
        assert.equal(
          fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
          false,
        );
      } finally {
        cleanup(fixture);
      }
    }
  },
);

test("pre-semantic checking cannot change either configuration authority", linuxOnly, () => {
  for (const target of ["generated", "authored"] as const) {
    const fixture = setup();
    try {
      const marker = 'if (process.argv.includes("--noCheck")) {';
      const source = fs.readFileSync(fixture.vueTsc, "utf8");
      assert.ok(source.includes(marker));
      const changedPath =
        target === "generated"
          ? 'process.argv[process.argv.indexOf("-p") + 1]'
          : JSON.stringify(path.join(fixture.fixtureRoot, "tsconfig.json"));
      fs.writeFileSync(
        fixture.vueTsc,
        source.replace(marker, `${marker}\nfs.appendFileSync(${changedPath}, "\\n");`),
      );
      const result = executeFixture(fixture);
      assert.equal(result.status, 1, result.stderr);
      const packet = readJson(path.join(fixture.reportDir, "fixture-baseline-unusable.json"));
      assert.equal(packet.cause.kind, "configuration-authority-failure");
      const observations = readJson(path.join(fixture.reportDir, packet.observations.file));
      assert.notDeepEqual(
        observations.preSemanticAuthorityBefore,
        observations.preSemanticAuthorityAfter,
      );
      assert.equal(observations.preSemantic.exitCode, 0);
      assert.equal(commandLabels(fixture).includes("Vize mutation run"), false);
      assert.equal(
        fs.existsSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.json")),
        false,
      );
    } finally {
      cleanup(fixture);
    }
  }
});

test(
  "a later usable attempt archives the earlier typed refusal without reusing it",
  linuxOnly,
  () => {
    const fixture = setup();
    const diagnostic = "src/env.d.ts(2,1): error TS1128: Declaration or statement expected.\n";
    try {
      fakeBaseline(fixture, diagnostic, { output: diagnostic, exitCode: 2 });
      const failed = executeFixture(fixture);
      assert.equal(failed.status, 1);
      const refusalPath = path.join(fixture.reportDir, "fixture-baseline-unusable.json");
      const originalRefusal = fs.readFileSync(refusalPath);
      writeVueTscFixture(
        fixture.vueTsc,
        {
          baselineOutput: sharedBaselineOutput,
          files: ["src/App.vue"],
          fixtureRoot: fixture.fixtureRoot,
          mutation: "match",
        },
        fixture.invocationPath,
      );
      const succeeded = executeFixture(fixture);
      assert.equal(succeeded.status, 0, succeeded.stderr);
      assert.equal(fs.existsSync(refusalPath), false);
      const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
      assert.equal(artifact.budget.passed, true);
      const archiveRoot = path.join(fixture.reportDir, "previous-typecheck-comparisons/fixture");
      const archives = fs.readdirSync(archiveRoot);
      assert.equal(archives.length, 1);
      const first = archives[0];
      assert.ok(first);
      const archived = fs.readFileSync(path.join(archiveRoot, first, "baseline-unusable.json"));
      assert.deepEqual(archived, originalRefusal);
    } finally {
      cleanup(fixture);
    }
  },
);
