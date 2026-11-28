import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  differentialJobLanes,
  differentialLaneSeconds,
  differentialMatrix,
  differentialSteps,
  rustDifferentialLanes,
} from "../../tools/support/compat/github/rust-differential-lanes.ts";
import { readToolingProgram } from "./support/program-path.ts";

const allPackages = [
  ...new Set(rustDifferentialLanes.flatMap((lane) => lane.steps.map((step) => step.package))),
  "vize_maestro",
];
const budget = Math.max(...Object.values(differentialLaneSeconds));
const jobSeconds = (lane: string) =>
  differentialJobLanes(lane).reduce((sum, name) => sum + differentialLaneSeconds[name]!, 0);

test("every differential lane has a cost, and packed jobs never outlast the longest lane", () => {
  assert.deepEqual(
    Object.keys(differentialLaneSeconds).sort(),
    rustDifferentialLanes.map(({ name }) => name).sort(),
  );
  // Packed names become job, packet and artifact names.
  for (const { name } of rustDifferentialLanes) assert.match(name, /^[a-z][a-z-]*$/u);
  const { include } = differentialMatrix(allPackages);
  assert.equal(include.length, 6);
  for (const { lane } of include) assert.ok(jobSeconds(lane) <= budget, lane);
});

test("packing selects each affected lane exactly once and keeps navigation in its own job", () => {
  const selections = [
    allPackages,
    ["vize_atelier_ssr"],
    ["vize_maestro"],
    ["vize_l1", "vize_l1_to_l2", "vize", "vize_vitrine"],
    ["vize_atelier_sfc", "vize_atelier_vapor", "vize_patina", "vize_maestro"],
  ];
  for (const packages of selections) {
    const { include } = differentialMatrix(packages);
    const expected = rustDifferentialLanes
      .filter((lane) =>
        lane.navigation
          ? packages.includes("vize_maestro")
          : lane.steps.some((step) => packages.includes(step.package)),
      )
      .map(({ name }) => name);
    const packed = include.flatMap(({ lane }) => differentialJobLanes(lane));
    assert.deepEqual([...packed].sort(), [...expected].sort(), packages.join(" "));
    for (const { lane, elementPlus } of include) {
      const lanes = differentialJobLanes(lane);
      if (lanes.includes("navigation")) assert.deepEqual(lanes, ["navigation"]);
      const reads = rustDifferentialLanes.some(
        (candidate) => lanes.includes(candidate.name) && candidate.elementPlus === true,
      );
      assert.equal(elementPlus, reads, lane);
    }
  }
  assert.deepEqual(differentialMatrix(["vize_unrelated"]), {
    include: [{ lane: "none", elementPlus: false }],
  });
  for (const name of ["program", "sfc-project", "navigation"])
    assert.equal(rustDifferentialLanes.find((lane) => lane.name === name)?.elementPlus, true);
});

test("a packed job runs every lane's commands in order and reports each failing lane", () => {
  const cwd = realpathSync(mkdtempSync(join(tmpdir(), "vize-differential-batch-")));
  const entry = fileURLToPath(
    new URL("../../tools/support/compat/github/rust-differential-lanes.ts", import.meta.url),
  );
  try {
    const bin = join(cwd, "bin");
    mkdirSync(bin);
    writeFileSync(
      join(bin, "cargo"),
      readToolingProgram("commands/ci-iteration-plans/cargo.cjs.in", { NODE: process.execPath }),
      { mode: 0o755 },
    );
    const plan = join(cwd, "plan.json");
    const log = join(cwd, "commands.jsonl");
    writeFileSync(plan, JSON.stringify({ packages: allPackages }));
    const env: NodeJS.ProcessEnv = { ...process.env, PATH: `${bin}:${process.env.PATH}` };
    env.COMMAND_LOG = log;
    delete env.VIZE_TEST_CASES;
    delete env.TEST_CASES_JOB;
    const calls = (lane: string, extra: NodeJS.ProcessEnv = {}) => {
      writeFileSync(log, "");
      const result = spawnSync(process.execPath, [entry, lane, plan], {
        cwd,
        env: { ...env, ...extra },
        encoding: "utf8",
      });
      const lines = readFileSync(log, "utf8").trim().split("\n").filter(Boolean);
      return { status: result.status, args: lines.map((line) => JSON.parse(line).args) };
    };
    for (const { lane } of differentialMatrix(allPackages).include) {
      if (lane === "navigation") continue;
      const single = differentialJobLanes(lane).flatMap((name) => calls(name).args);
      assert.deepEqual(calls(lane), { status: 0, args: single }, lane);
    }
    // The compiler lane's first command fails; the cli lane after it still runs.
    const failed = calls("compiler+cli", { FAIL_TARGET: "vize_atelier_jsx" });
    assert.equal(failed.status, 42);
    assert.deepEqual(failed.args, [
      differentialSteps("compiler", allPackages)[0]!.args,
      ...differentialSteps("cli", allPackages).map((step) => step.args),
    ]);
    assert.notEqual(calls("compiler+navigation").status, 0);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
