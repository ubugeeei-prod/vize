import assert from "node:assert/strict";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  markdown,
  measure,
  summarize,
  type InvocationEvidence,
  type Sample,
  type Tool,
} from "../../tools/benchmarks/scripts/typecheck-vs-vue-tsc.ts";

import { prepareSyntheticWorkspace } from "../../tools/benchmarks/scripts/typecheck-vs-vue-tsc-projects.ts";

const oracle: Tool = { label: "vue-tsc", kind: "vue-tsc", bin: "/unused", env: {} };
const vize: Tool = { label: "vize", kind: "vize", bin: "/unused", env: {} };
const project = { id: "fixture", cwd: "/unused", tsconfig: "tsconfig.json" };

test("Actions retains partial evidence after failure and records the source actually checked out", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/typecheck-vs-vue-tsc.yml", import.meta.url),
    "utf8",
  );
  const upload = workflow
    .split(/^      - /m)
    .find((step) => step.startsWith("uses: actions/upload-artifact@"));
  assert.match(upload ?? "", /^        if: always\(\)$/m);
  assert.match(
    workflow.split(/^      - /m).find((step) => step.startsWith("name: Measure\n")) ?? "",
    /--meta "head=\$\(git rev-parse HEAD\)"/,
  );
});

function sample(
  diagnostics: string[],
  phase: Sample["phase"] = "warm",
  status = diagnostics.length ? 1 : 0,
  wall = 1,
): Sample {
  return {
    phase,
    cold: phase === "cold",
    wall,
    cpu: wall,
    stages: {},
    diagnostics,
    status,
    diagnosticsClassified: status === 0 ? diagnostics.length === 0 : diagnostics.length > 0,
  };
}

function report(baseline: Sample[], actual: Sample[]) {
  return summarize(
    project,
    [oracle, vize],
    new Map([
      [oracle.label, baseline],
      [vize.label, actual],
    ]),
  );
}

function fakeTool(root: string, stdout: string, stderr: string, status: number): Tool {
  const bin = join(root, "fixture checker.cjs");
  writeFileSync(
    bin,
    `#!/usr/bin/env node\nprocess.stdout.write(${JSON.stringify(stdout)});\nprocess.stderr.write(${JSON.stringify(stderr)});\nprocess.exitCode = ${status};\n`,
  );
  chmodSync(bin, 0o755);
  return { ...vize, bin };
}

test("a measured invocation retains complete streams, process status and coarse keys", () => {
  const root = mkdtempSync(join(os.tmpdir(), "vize-bench evidence-"));
  try {
    const stdout = `${join(root, "src/日本 App.vue")}\r\n  error:2:4 [TS2339] Property '不正' does not exist.\r\nType checked 1 files in 3ms (collect:1ms,gen:1ms,corsa:1ms)\r\n`;
    const stderr = "native note\r\n";
    const invocations: InvocationEvidence[] = [];
    const measured = measure(
      fakeTool(root, stdout, stderr, 1),
      { ...project, cwd: root },
      "cold",
      (invocation) => invocations.push(invocation) - 1,
    );
    assert.equal(invocations.length, 1);
    const retained = invocations[0]!;
    assert.equal(retained.stdout, stdout);
    assert.equal(retained.stderr.slice(0, stderr.length), stderr);
    assert.match(retained.stderr.slice(stderr.length), /__tcbench [\d.]+ [\d.]+ [\d.]+/);
    assert.equal(retained.status, 1);
    assert.equal(retained.signal, null);
    assert.equal(retained.phase, "cold");
    assert.equal(retained.cwd, root);
    assert.deepEqual(retained.args, ["check", "--no-config", "--tsconfig", "tsconfig.json"]);
    assert.deepEqual(measured.diagnostics, ["src/日本 App.vue:2:TS2339"]);
    assert.equal(measured.status, 1);
    assert.equal(measured.diagnosticsClassified, true);
    assert.equal(measured.invocation, 0);
    assert.deepEqual(measured.stages, { collect: 0.001, gen: 0.001, corsa: 0.001 });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("failed commands retain the full output before the error excerpt is thrown", () => {
  const root = mkdtempSync(join(os.tmpdir(), "vize-bench-failure-"));
  try {
    const stdout = `retained-prefix\n${"x".repeat(6_000)}\n`;
    const invocations: InvocationEvidence[] = [];
    assert.throws(
      () =>
        measure(
          fakeTool(root, stdout, "unexpected exit\n", 3),
          { ...project, cwd: root },
          "warmup",
          (invocation) => invocations.push(invocation) - 1,
        ),
      /vize failed on fixture \(3\)/,
    );
    assert.equal(invocations.length, 1);
    assert.equal(invocations[0]!.stdout, stdout);
    assert.equal(invocations[0]!.status, 3);
    assert.equal(invocations[0]!.phase, "warmup");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("diagnostic multiplicity remains visible and cannot qualify a faster row", () => {
  const diagnostic = "src/App.vue:4:TS2339";
  const result = report(
    [sample([diagnostic, diagnostic])],
    [sample([diagnostic], "warm", 1, 0.01)],
  );
  const actual = result.tools[1]!;
  assert.deepEqual(actual.vsVueTsc, { missing: 1, extra: 0 });
  assert.equal(actual.comparison, "mismatch");
  assert.deepEqual(actual.samples[0]!.diagnostics, [diagnostic]);
  const text = markdown([result], {});
  assert.match(text, /coarse missing \/ extra/);
  assert.match(text, /1 \/ 0 \| mismatch/);
  assert.match(text, /does not establish full diagnostic parity/);
});

test("additional findings remain a mismatch even when no finding is missing", () => {
  const result = report([sample([])], [sample(["src/App.vue:3:TS2322"])]);
  assert.deepEqual(result.tools[1]!.vsVueTsc, { missing: 0, extra: 1 });
  assert.equal(result.tools[1]!.comparison, "mismatch");
});

test("cold, warmup and warm diagnostics are retained and drift is marked unstable", () => {
  const baseline = [sample(["src/App.vue:3:TS2339"])];
  const rows = [
    sample(["src/App.vue:3:TS2339"], "cold", 1, 8),
    sample([], "warmup", 0, 99),
    sample(["src/App.vue:3:TS2339"], "warm", 1, 2),
    sample(["src/App.vue:3:TS2339"], "warm", 1, 3),
  ];
  const result = report(baseline, rows).tools[1]!;
  assert.equal(result.stable, false);
  assert.equal(result.comparison, "unstable");
  assert.equal(result.warm.wall, 2.5);
  assert.equal(result.cold!.wall, 8);
  assert.deepEqual(
    result.samples.map((row) => [row.phase, row.status, row.diagnostics]),
    rows.map((row) => [row.phase, row.status, row.diagnostics]),
  );
});

test("oracle drift prevents a coarse match with its first sample", () => {
  const result = report([sample([]), sample(["src/App.vue:3:TS2339"])], [sample([])]);
  assert.deepEqual(result.tools[1]!.vsVueTsc, { missing: 0, extra: 0 });
  assert.equal(result.tools[1]!.comparison, "unstable");
});

test("nonzero commands with no classified finding cannot match a clean oracle", () => {
  const result = report([sample([])], [sample([], "warm", 2)]).tools[1]!;
  assert.deepEqual(result.vsVueTsc, { missing: 0, extra: 0 });
  assert.equal(result.comparison, "unclassified");
});

test("matching findings are explicitly coarse and a missing oracle stays unverified", () => {
  const keys = ["src/App.vue:3:TS2339"];
  assert.equal(report([sample(keys)], [sample(keys)]).tools[1]!.comparison, "coarse-match");
  assert.equal(
    summarize(project, [vize], new Map([[vize.label, [sample(keys)]]])).tools[0]!.comparison,
    "unverified",
  );
});

test("scaled workspaces retain the planted oracle and reject vacuous or fractional shapes", () => {
  const root = mkdtempSync(join(os.tmpdir(), "vize-scaled-workspace-"));
  try {
    for (const count of [0, 1, -1, 1.5, Number.NaN, Number.POSITIVE_INFINITY]) {
      assert.throws(() => prepareSyntheticWorkspace(root, {}, count), /integer of at least two/);
    }
    const vue = join(root, "vue"),
      typescript = join(root, "typescript");
    mkdirSync(vue);
    mkdirSync(typescript);
    writeFileSync(join(vue, "package.json"), '{"name":"vue","version":"0.0.0"}');
    writeFileSync(join(typescript, "package.json"), '{"name":"typescript","version":"0.0.0"}');
    const project = prepareSyntheticWorkspace(root, { vue, typescript }, 3);
    assert.deepEqual(readdirSync(join(project.cwd, "src/components")).sort(), [
      "Component0.vue",
      "Component1.vue",
      "Component2.vue",
    ]);
    assert.match(
      readFileSync(join(project.cwd, "src/Planted.vue"), "utf8"),
      /const count: number = "planted"/,
    );
    assert.match(
      readFileSync(join(project.cwd, "src/Planted.vue"), "utf8"),
      /import Child from "\.\/components\/Component1\.vue"/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
