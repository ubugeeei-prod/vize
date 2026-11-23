import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, join, resolve } from "node:path";
import { test, type TestContext } from "node:test";

const driver = resolve(import.meta.dirname, "../../tools/benchmarks/scripts/criterion-ab.ts");

function fixture(t: TestContext) {
  const root = mkdtempSync(join(tmpdir(), "vize-criterion-current-run-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const target = join(root, "target");
  const bin = join(root, "bin");
  const external = join(root, "external-results");
  for (const dir of [bin, external, join(root, "base"), join(root, "head")]) {
    mkdirSync(dir, { recursive: true });
  }
  writeFileSync(join(external, "keep.json"), "external baseline");
  for (const [side, value] of [
    ["base", 100],
    ["head", 300],
  ] as const) {
    const sideTarget = join(target, `${side}-target`);
    mkdirSync(join(sideTarget, "criterion"), { recursive: true });
    mkdirSync(join(sideTarget, "release"), { recursive: true });
    writeFileSync(join(sideTarget, "release", "cached-build"), "compiled artifact");
    writeFileSync(join(sideTarget, "criterion", "removed-benchmark.json"), JSON.stringify(value));
  }
  writeFileSync(
    join(root, "selection.json"),
    JSON.stringify({ selected: ["vize_glyph"], reason: "Formatter-only change." }),
  );
  // Exercise the real driver and filesystem boundaries without compiling Rust.
  // The fake exporter includes every saved row, just as critcmp does.
  writeFileSync(
    join(bin, "cargo"),
    readToolingProgram("commands/criterion-current-run/cargo.cjs"),
    { mode: 0o755 },
  );
  writeFileSync(
    join(bin, "critcmp"),
    readToolingProgram("commands/criterion-current-run/critcmp.cjs"),
    { mode: 0o755 },
  );
  function run(empty = false) {
    return spawnSync(
      process.execPath,
      [
        driver,
        "--base-dir",
        join(root, "base"),
        "--head-dir",
        join(root, "head"),
        "--target-dir",
        target,
        "--selection",
        join(root, "selection.json"),
        "--threshold",
        "10",
      ],
      {
        encoding: "utf8",
        timeout: 10_000,
        env: {
          ...process.env,
          PATH: `${bin}${delimiter}${process.env.PATH ?? ""}`,
          CRITERION_HOME: external,
          GITHUB_STEP_SUMMARY: "",
          VIZE_TEST_EMPTY_CRITERION: empty ? "1" : "",
        },
      },
    );
  }
  return { root, target, external, run };
}

test("Criterion exports only this run while retaining Cargo artifacts and external results", (t) => {
  const { target, external, run } = fixture(t);
  const result = run();
  assert.equal(result.status, 0, result.stderr + result.stdout);
  assert.match(result.stdout, /current-benchmark/);
  assert.doesNotMatch(result.stdout, /removed-benchmark/);
  for (const side of ["base", "head"]) {
    const exported = JSON.parse(readFileSync(join(target, `criterion-ab-${side}.json`), "utf8"));
    assert.deepEqual(Object.keys(exported.benchmarks), ["current-benchmark"]);
    assert.equal(
      readFileSync(join(target, `${side}-target`, "release", "cached-build"), "utf8"),
      "compiled artifact",
    );
  }
  assert.equal(readFileSync(join(external, "keep.json"), "utf8"), "external baseline");
});

test("Criterion rejects an empty measurement instead of reusing a cached passing baseline", (t) => {
  const { run } = fixture(t);
  const result = run(true);
  assert.equal(result.status, 1, result.stderr + result.stdout);
  assert.match(result.stderr, /export for base contains no benchmarks/);
});
