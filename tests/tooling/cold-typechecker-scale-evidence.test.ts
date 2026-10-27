import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync, readFileSync, mkdirSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  captureColdScale,
  prepareColdScale,
  verifyColdDiagnostics,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale.ts";

type ScaleWorkflow = {
  on: { pull_request: { paths: string[] } };
  jobs: { profile: { steps: { name?: string; run?: string }[] } };
};

function scaleWorkflow(): ScaleWorkflow {
  return parse(
    readFileSync(
      new URL("../../.github/workflows/cold-typechecker-scale.yml", import.meta.url),
      "utf8",
    ),
  ) as ScaleWorkflow;
}

function workflowScript(name: string): string {
  const step = scaleWorkflow().jobs.profile.steps.find((candidate) => candidate.name === name);
  assert.ok(step && typeof step.run === "string", "the real workflow step must be exercised");
  return step.run;
}

// Independently equal public vize0.596 / native TS7.1 / vue-tsc3.3.13 packets.
const diagnostics = [
  "error:4:7 [TS2322] Type 'string' is not assignable to type 'number'.",
  "error:6:34 [TS2339] Property 'definitelyMissingMethod' does not exist on type 'HTMLButtonElement'.",
  "error:8:61 [TS2322] Type 'number' is not assignable to type 'string'.",
];
const packet = () => ({
  fileCount: 2500,
  errorCount: 3,
  warningCount: 0,
  files: [{ file: "src/Control.vue", diagnostics: [...diagnostics] }],
});

test("scale evidence preserves complete authored coordinates, codes and messages", () => {
  const cli = verifyColdDiagnostics(JSON.stringify(packet()), "cli", 2500, true);
  const native = diagnostics
    .map(
      (x) =>
        "src/Control.vue(" + x.replace(/^error:(\d+):(\d+) \[TS(\d+)\] /, "$1,$2): error TS$3: "),
    )
    .join("\n");
  assert.deepEqual(verifyColdDiagnostics(native, "mapper", 2500, true), cli);
  assert.deepEqual(verifyColdDiagnostics(native, "vue-tsc", 2500, true), cli);
});

test("skipped files, missing findings, wrong source locations and lost ref types fail", () => {
  for (const mutate of [
    (x: ReturnType<typeof packet>) => {
      x.fileCount--;
    },
    (x: ReturnType<typeof packet>) => {
      x.files[0].diagnostics.pop();
      x.errorCount--;
    },
    (x: ReturnType<typeof packet>) => {
      x.files[0].diagnostics[1] = x.files[0].diagnostics[1].replace("6:34", "6:33");
    },
    (x: ReturnType<typeof packet>) => {
      x.files[0].diagnostics[1] = x.files[0].diagnostics[1].replace("HTMLButtonElement", "unknown");
    },
    (x: ReturnType<typeof packet>) => {
      x.files[0].file = "src/Other.vue";
    },
  ]) {
    const x = packet();
    mutate(x);
    assert.throws(() => verifyColdDiagnostics(JSON.stringify(x), "cli", 2500, true), {
      code: "ERR_ASSERTION",
    });
  }
});

test("fixture preparation refuses to replace an existing evidence directory", () => {
  const root = mkdtempSync(join(tmpdir(), "cold-scale-protected-"));
  try {
    const witness = join(root, "evidence.txt");
    writeFileSync(witness, "retained");
    assert.throws(() => prepareColdScale(root, root, 2500, false), { code: "ERR_ASSERTION" });
    assert.equal(readFileSync(witness, "utf8"), "retained");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("exported capture creates its own empty temporary directory before invoking the checker", () => {
  const root = mkdtempSync(join(tmpdir(), "cold-scale-capture-"));
  try {
    const provider = join(root, "provider");
    const native = `@typescript/typescript-${process.platform === "darwin" ? "darwin" : "linux"}-${process.arch}`;
    for (const name of ["vue", "typescript", "typescript-native", native]) {
      const dir = join(provider, "node_modules", name);
      mkdirSync(dir, { recursive: true });
      writeFileSync(join(dir, "package.json"), JSON.stringify({ name, version: "0.0.0" }));
    }
    const binary = join(root, "checker.cjs");
    writeFileSync(
      binary,
      `#!${process.execPath}\nconst fs = require("node:fs");
fs.writeFileSync(process.env.TMPDIR + "/probe", "fresh temporary directory");
console.log(JSON.stringify({fileCount:2000,errorCount:0,warningCount:0,files:[]}));\n`,
      { mode: 0o755 },
    );
    const out = join(root, "evidence");
    // This stub verifies the capture API's filesystem contract, not product accuracy.
    captureColdScale({ provider, out, files: 2000, tool: "cli", invalid: false, binary });
    assert.equal(readFileSync(join(out, "tmp", "probe"), "utf8"), "fresh temporary directory");
    const report = JSON.parse(readFileSync(join(out, "measurement.json"), "utf8"));
    assert.equal(report.status, 0);
    assert.equal(report.correctness.passed, true);
    assert.equal(report.meetsColdTarget, false, "177 SFCs cannot satisfy the 2500-SFC target");
    assert.equal(report.meetsFixtureLatencyThreshold, false);
    assert.equal(report.cold.sourceConfigIncremental, false);
    assert.equal(report.cold.effectiveBackendIncremental, true);
    assert.equal(
      report.cold.cacheQualified,
      false,
      "source config alone does not qualify the native cache",
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("workspace generator edits trigger the complete SFC corpus measurement", () => {
  assert.ok(
    scaleWorkflow().on.pull_request.paths.includes("tools/benchmarks/scripts/check-workspace.ts"),
  );
});

test("manual Actions comparisons reject absent, mutable or identical baseline identities", () => {
  const root = mkdtempSync(join(tmpdir(), "cold-scale-baseline-"));
  try {
    const script = workflowScript("Validate the full SFC cohort");
    const cwd = new URL("../../", import.meta.url);
    const head = spawnSync("git", ["rev-parse", "HEAD"], { cwd, encoding: "utf8" });
    assert.equal(head.status, 0, head.stderr);
    for (const base of ["", "main", "a".repeat(39), head.stdout.trim(), "a".repeat(40)]) {
      const envFile = join(
        root,
        "environment-" + base.length + (base === head.stdout.trim() ? "-head" : ""),
      );
      writeFileSync(envFile, "");
      const result = spawnSync("bash", ["-euc", script], {
        cwd,
        env: {
          ...process.env,
          BASE_SHA: base,
          BENCH_EVENT: "workflow_dispatch",
          RUNNER_TEMP: root,
          VIZE_BENCH_SFCS: "2500",
          GITHUB_ENV: envFile,
        },
        encoding: "utf8",
      });
      assert.equal(result.status === 0, base === "a".repeat(40), result.stderr);
      assert.equal(
        readFileSync(envFile, "utf8"),
        base === "a".repeat(40) ? "BASE_SHA=" + base + "\nVIZE_BENCH_FILES=4323\n" : "",
      );
    }
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("Actions evidence packaging retains packets without following dependency-link loops", () => {
  const root = mkdtempSync(join(tmpdir(), "cold-scale-archive-"));
  try {
    const evidence = join(root, "cold-scale", "head-1");
    const project = join(evidence, "project");
    mkdirSync(join(project, "node_modules", "@ws"), { recursive: true });
    symlinkSync(project, join(project, "node_modules", "@ws", "loop"), "dir");
    const packet = '{"correctness":{"passed":true},"files":4323,"sfcFiles":2500}\n';
    writeFileSync(join(evidence, "measurement.json"), packet);
    writeFileSync(join(evidence, "stdout.txt"), "complete diagnostic bytes\n");
    const script = workflowScript("Package evidence without traversing workspace links");
    const packed = spawnSync("bash", ["-euc", script], {
      env: { ...process.env, RUNNER_TEMP: root },
      encoding: "utf8",
      timeout: 10000,
    });
    assert.equal(packed.status, 0, packed.stderr);
    const archive = join(root, "cold-scale-evidence.tgz");
    const listing = spawnSync("tar", ["-tzf", archive], { encoding: "utf8" });
    assert.equal(listing.status, 0, listing.stderr);
    assert.deepEqual(listing.stdout.trim().split("\n").sort(), [
      "cold-scale/",
      "cold-scale/head-1/",
      "cold-scale/head-1/measurement.json",
      "cold-scale/head-1/stdout.txt",
    ]);
    const restored = spawnSync("tar", ["-xOf", archive, "cold-scale/head-1/measurement.json"], {
      encoding: "utf8",
    });
    assert.equal(restored.status, 0, restored.stderr);
    assert.equal(restored.stdout, packet);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
