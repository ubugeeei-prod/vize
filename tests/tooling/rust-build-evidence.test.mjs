import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse } from "yaml";
import {
  prepareBuildEvidence,
  finishBuildEvidence,
} from "../../tools/support/compat/github/rust-build-evidence.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const workflow = parse(readFileSync(join(root, ".github/workflows/pr-rust-checks.yml"), "utf8"));
const step = workflow.jobs["pr-rust-shard"].steps.find(
  (entry) => entry.name === "Build and run the affected Rust test cohort",
);
const json = (file) => JSON.parse(readFileSync(file, "utf8"));
const fixture = () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-rust-build-evidence-"));
  const target = join(directory, "target");
  const output = join(directory, "evidence");
  mkdirSync(join(target, "cargo-timings"), { recursive: true });
  return { directory, target, output };
};

await test("fresh HTML and emitted unit fingerprints preserve source and raw packet identity", () => {
  const { directory, target, output } = fixture();
  try {
    const timing = join(target, "cargo-timings");
    writeFileSync(join(timing, "cargo-timing-old.html"), "stale");
    writeFileSync(join(timing, "cargo-timing.html"), "stale alias");
    prepareBuildEvidence(output, target);
    writeFileSync(join(timing, "cargo-timing-new.html"), "fresh");
    writeFileSync(join(timing, "cargo-timing.html"), "fresh");
    const hash = "0123456789abcdef";
    const filename = join(target, "ci/deps", `libexample-${hash}.rlib`);
    const fingerprint = join(target, "ci/.fingerprint", `example-${hash}`, "lib-example.json");
    mkdirSync(dirname(filename), { recursive: true });
    mkdirSync(dirname(fingerprint), { recursive: true });
    writeFileSync(filename, "compiled object must not be copied");
    writeFileSync(fingerprint, '{"deps":[],"profile":7}');
    const packet = {
      reason: "compiler-artifact",
      package_id: "example@1",
      target: { kind: ["lib"] },
      profile: { test: false },
      features: ["native"],
      fresh: true,
      filenames: [filename],
    };
    writeFileSync(join(output, "cargo.jsonl"), `${JSON.stringify(packet)}\n`);
    const context = { source: "exact source", tree: "exact tree", cohort: "compiler" };
    const report = finishBuildEvidence(output, target, context);
    assert.deepEqual(report.context, context);
    assert.deepEqual(report.units[0].packet, packet);
    const retained = report.units[0].fingerprints[0];
    assert.equal(
      readFileSync(join(output, retained.path), "utf8"),
      readFileSync(fingerprint, "utf8"),
    );
    assert(!retained.path.includes(".fingerprint"));
    const after = json(join(output, "html-after.json"));
    assert.deepEqual(
      after.retained.map((file) => file.name),
      ["cargo-timing-new.html", "cargo-timing.html"],
    );
    for (const file of after.retained)
      assert.equal(file.sha256, createHash("sha256").update("fresh").digest("hex"));
    assert(
      json(join(output, "html-before.json")).files.some(
        (file) => file.name === "cargo-timing-old.html",
      ),
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

await test("failed/missing preparation keeps partial raw output and excludes cached or escaped HTML", () => {
  const { directory, target, output } = fixture();
  try {
    const timing = join(target, "cargo-timings");
    writeFileSync(join(timing, "cargo-timing-old.html"), "cached");
    writeFileSync(join(directory, "outside.html"), "outside must not be copied");
    symlinkSync(join(directory, "outside.html"), join(timing, "cargo-timing-escape.html"));
    prepareBuildEvidence(output, target);
    writeFileSync(join(output, "cargo.jsonl"), '{"reason":"compiler-artifact"\n');
    const report = finishBuildEvidence(output, target, {});
    assert.equal(report.parseGaps.length, 1);
    assert.equal(
      readFileSync(join(output, "cargo.jsonl"), "utf8"),
      '{"reason":"compiler-artifact"\n',
    );
    const after = json(join(output, "html-after.json"));
    assert.equal(after.retained.length, 0);
    assert(after.gaps.some((gap) => gap.name === "cargo-timing-escape.html"));
    assert(after.gaps.some((gap) => gap.reason.includes("no fresh HTML")));
    rmSync(join(output, "html-before.json"));
    finishBuildEvidence(output, target, {});
    assert(
      json(join(output, "html-after.json")).gaps.some(
        (gap) => gap.reason === "prepare did not run",
      ),
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

await test("actual PR shell keeps selected arguments, captured stdout and exit0/17 through tee", () => {
  for (const exit of [0, 17]) {
    const { directory } = fixture();
    try {
      const bin = join(directory, "bin");
      mkdirSync(bin);
      const scripts = join(directory, "tools/support/compat/github");
      mkdirSync(scripts, { recursive: true });
      for (const name of [
        "run-affected-rust.ts",
        "rust-build-evidence.ts",
        "compare-unused-bindings-profiles.ts",
      ])
        copyFileSync(join(root, "tools/support/compat/github", name), join(scripts, name));
      const argsFile = join(directory, "args.txt");
      writeFileSync(
        join(bin, "cargo"),
        `#!/bin/sh\nprintf '%s\\n' "$@" > "$ARGS_FILE"\nprintf '%s\\n' '{"reason":"build-finished","success":${exit === 0}}'\nexit ${exit}\n`,
        { mode: 0o755 },
      );
      const plan = {
        schemaVersion: 1,
        scope: "affected",
        packages: ["vize_l0"],
        cargoArgs: ["--package", "vize_l0"],
      };
      const result = spawnSync("bash", ["-e", "-c", step.run], {
        cwd: directory,
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${bin}:${dirname(process.execPath)}:${process.env.PATH}`,
          RUNNER_TEMP: directory,
          RUST_PLAN_JSON: JSON.stringify(plan),
          NEXTEST_PROFILE: "pr",
          NEXTEST_FILTER: "package(=vize_l0)",
          COHORT: "compiler",
          ARGS_FILE: argsFile,
          VIZE_TEST_DISABLE_TSGO: "1",
        },
      });
      assert.equal(result.status, exit, result.stderr);
      const args = readFileSync(argsFile, "utf8").trim().split("\n");
      assert.deepEqual(args, [
        "nextest",
        "run",
        "--package",
        "vize_l0",
        "--cargo-profile",
        "ci",
        "--timings",
        "--cargo-message-format=json-render-diagnostics",
        "--profile",
        "pr",
        "-E",
        "package(=vize_l0)",
        "--no-tests=pass",
      ]);
      const packet = json(join(directory, "rust-build-evidence/cargo.jsonl"));
      assert.equal(packet.success, exit === 0);
      assert.equal(json(join(directory, "target/nextest/pr/timing.json")).exit_code, exit);
      assert.equal(
        workflow.jobs["pr-rust-shard"].steps.find(
          (entry) => entry.name === "Upload PR Rust build evidence",
        ).if,
        "${{ always() && github.event_name == 'pull_request' }}",
      );
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  }
});
