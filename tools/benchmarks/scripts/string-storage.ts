import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { isRecord } from "./script-types.ts";

const sides = ["base", "head"] as const;
const crate = "crates/vize_croquis_cf";
const reports = path.resolve("reports");

function prepareHarness(): void {
  const base = process.env.BASE_SHA;
  const head = process.env.HEAD_SHA;
  assert.ok(base && /^[0-9a-f]{40}$/u.test(base));
  assert.ok(head && /^[0-9a-f]{40}$/u.test(head));
  assert.equal(head, process.env.DISPATCH_SHA);
  for (const [side, sha] of [
    ["base", base],
    ["head", head],
  ] as const) {
    assert.equal(
      execFileSync("git", ["rev-parse", "HEAD"], { cwd: side, encoding: "utf8" }).trim(),
      sha,
    );
  }
  execFileSync("git", ["merge-base", "--is-ancestor", base, head], { cwd: "head" });
  mkdirSync(reports, { recursive: true });
  writeFileSync(path.join(reports, "base-sha.txt"), base + "\n");
  writeFileSync(path.join(reports, "head-sha.txt"), head + "\n");
  for (const bench of ["cross_file.rs", "string_storage.rs"]) {
    const file = path.join(crate, "benches", bench);
    copyFileSync(path.join("head", file), path.join("base", file));
  }

  const manifest = path.join("base", crate, "Cargo.toml");
  const source = readFileSync(manifest, "utf8");
  const dependency = "davinci_harness.workspace = true";
  if (!source.includes(dependency)) {
    writeFileSync(
      manifest,
      source.replaceAll("[dev-dependencies]", `[dev-dependencies]\n${dependency}`),
    );
  }
}

function readReport(file: string): Record<string, unknown> {
  const value: unknown = JSON.parse(readFileSync(file, "utf8"));
  assert.ok(isRecord(value), file);
  return value;
}

function measure(): void {
  for (const side of sides) {
    const target = path.resolve("head", "target", side);
    const criterion = path.join(target, "criterion");
    rmSync(criterion, { recursive: true, force: true });
    execFileSync(
      "cargo",
      [
        "bench",
        "-p",
        "vize_croquis_cf",
        "--bench",
        "cross_file",
        "--target-dir",
        target,
        "--",
        "string_storage",
        "--warm-up-time",
        "0.5",
        "--measurement-time",
        "1",
        "--sample-size",
        "30",
        "--save-baseline",
        side,
      ],
      {
        cwd: side,
        stdio: "inherit",
        env: {
          ...process.env,
          CRITERION_HOME: criterion,
          STRING_STORAGE_REPORT_DIR: path.join(reports, side),
        },
      },
    );
  }
  for (const name of readdirSync(path.join(reports, "base"))) {
    if (/^tree_.*\.json$/u.test(name)) {
      assert.deepEqual(
        readReport(path.join(reports, "base", name)).semantic,
        readReport(path.join(reports, "head", name)).semantic,
        name,
      );
    }
  }
  // The harness measures two trees and add/resolve for three module-path lengths.
  const expectedEstimates = 2 + 2 * 3;
  for (const side of sides) {
    const criterion = path.resolve("head", "target", side, "criterion");
    const estimates = readdirSync(criterion, { recursive: true, encoding: "utf8" }).filter(
      (file) =>
        path.basename(file) === "estimates.json" && path.basename(path.dirname(file)) === side,
    );
    assert.equal(estimates.length, expectedEstimates, `${side} Criterion estimates`);
    for (const file of estimates) {
      const estimate = path.join(criterion, file);
      const { median } = readReport(estimate);
      assert.ok(
        isRecord(median) && typeof median.point_estimate === "number" && median.point_estimate > 0,
        estimate,
      );
      assert.ok(statSync(path.join(path.dirname(estimate), "sample.json")).isFile(), estimate);
    }
  }
}

switch (process.argv[2]) {
  case "prepare":
    prepareHarness();
    break;
  case "measure":
    measure();
    break;
  default:
    throw new Error("Expected prepare or measure");
}
