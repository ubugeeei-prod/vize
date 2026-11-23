import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  artifactPath,
  cleanup,
  commit,
  git,
  run,
  setup,
  successBody,
  writeJson,
  writeManager,
} from "./support/typecheck-dependency-prepare-fixture.ts";

test("dependency prepare runs ordered baseline steps that build on each other", () => {
  const fixture = setup();
  try {
    const prepare = [
      ["pnpm", "exec", "fixture", "stub"],
      ["pnpm", "exec", "fixture", "generate"],
    ];
    const project = {
      ...fixture.project,
      typecheckPerformance: {
        ...fixture.project.typecheckPerformance,
        baseline: { tsconfig: ".generated/tsconfig.json", prepare },
      },
    };
    writeJson(fixture.registryPath, { projects: [project] });
    git(fixture.fixtureRoot, ["add", "registry.json"]);
    commit(fixture.fixtureRoot, "configure ordered baseline steps");
    // `generate` needs the stub `stub` wrote, like element-plus `gen:version`.
    const steps = `const step = process.argv[4];
      if (step === "stub") { fs.mkdirSync(".generated", { recursive: true }); fs.writeFileSync(".generated/stub", "ok"); }
      else if (step === "generate") { if (!fs.existsSync(".generated/stub")) process.exit(7); fs.writeFileSync(".generated/tsconfig.json", "{}\\n"); }
      else process.exit(9);`;
    writeManager(
      fixture.runner,
      fixture.invocationPath,
      "10.0.0",
      readToolingProgram(
        "commands/typecheck-dependency-prepare-steps/package-manager-runs-ordered-baseline-build-each-other.js.in",
        { SUCCESS_BODY: successBody, STEPS: steps },
      ),
      { spec: "pnpm@10.0.0" },
    );
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = JSON.parse(fs.readFileSync(artifactPath(fixture), "utf8"));
    assert.deepEqual(artifact.baselinePrepare.command, prepare);
    assert.equal(artifact.baselinePrepare.exitCode, 0);
    assert.equal(fs.existsSync(path.join(fixture.fixtureRoot, ".generated/tsconfig.json")), true);

    fs.rmSync(path.join(fixture.fixtureRoot, ".generated"), { recursive: true, force: true });
    fs.rmSync(artifactPath(fixture), { force: true });
    writeManager(
      fixture.runner,
      fixture.invocationPath,
      "10.0.0",
      readToolingProgram(
        "commands/typecheck-dependency-prepare-steps/package-manager-dependency-prepare-runs-ordered-baseline-steps-build-each-other.js.in",
        { SUCCESS_BODY: successBody, STEPS: steps },
      ),
      { spec: "pnpm@10.0.0" },
    );
    const failed = run(fixture);
    assert.equal(failed.status, 1);
    assert.match(failed.stderr, /baseline prepare step 2 exited with status 7/);
  } finally {
    cleanup(fixture);
  }
});
