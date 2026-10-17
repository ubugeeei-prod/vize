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
import {
  prepareBuildEvidence,
  finishBuildEvidence,
} from "../../tools/support/compat/github/rust-build-evidence.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
// The source-plan job has Node but deliberately installs no dependencies.
// Execute the actual named literal shell block without importing a YAML package.
const workflowSource = readFileSync(join(root, ".github/workflows/pr-rust-checks.yml"), "utf8");
function workflowStep(name) {
  const marker = `      - name: ${name}\n`;
  const start = workflowSource.indexOf(marker);
  assert(start >= 0 && workflowSource.indexOf(marker, start + marker.length) === -1);
  const next = workflowSource.indexOf("\n      - ", start + marker.length);
  const block = workflowSource.slice(start, next < 0 ? undefined : next);
  const run = /^        run: \|\n([\s\S]*)$/m.exec(block)?.[1];
  return {
    if: /^        if: (.+)$/m.exec(block)?.[1],
    run: run
      ?.split("\n")
      .map((line) => {
        assert(line.trim() === "" || line.startsWith("          "));
        return line.slice(10);
      })
      .join("\n"),
  };
}
const step = workflowStep("Build and run the affected Rust test cohort");
assert.equal(typeof step.run, "string");
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

await test("actual PR shell keeps selected arguments, owned stdout capture and exit0/17", () => {
  for (const exit of [0, 17]) {
    const { directory } = fixture();
    try {
      const bin = join(directory, "bin");
      mkdirSync(bin);
      const scripts = join(directory, "tools/support/compat/github");
      mkdirSync(scripts, { recursive: true });
      for (const name of [
        "run-affected-rust.ts",
        "run-rust-test-cohort.ts",
        "rust-test-cohorts.ts",
        "rust-test-default-features.ts",
        "plan-affected-rust.ts",
        "plan-source-checks.ts",
        "rust-differential-lanes.ts",
        "rust-build-evidence.ts",
        "compare-unused-bindings-profiles.ts",
      ])
        copyFileSync(join(root, "tools/support/compat/github", name), join(scripts, name));
      const argsFile = join(directory, "args.txt");
      writeFileSync(
        join(bin, "cargo"),
        `#!/bin/sh\nif [ "$1" = metadata ]; then printf '%s' "$CARGO_METADATA"; exit 0; fi\nprintf '%s\\n' "$@" > "$ARGS_FILE"\nprintf '%s\\n' '{"reason":"build-finished","success":${exit === 0}}'\nexit ${exit}\n`,
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
          CARGO_METADATA: JSON.stringify({
            workspace_root: directory,
            workspace_members: plan.packages,
            packages: plan.packages.map((name) => ({
              id: name,
              name,
              manifest_path: join(directory, name, "Cargo.toml"),
              dependencies: [],
              targets: [{ name, kind: ["lib"], test: true }],
            })),
          }),
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
        "--lib",
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
      const captured = readFileSync(join(directory, "rust-build-evidence/cargo.jsonl"), "utf8");
      assert.equal(captured, result.stdout, "retain the complete unmodified stdout stream");
      const packets = captured.split("\n").filter((line) => line.startsWith("{"));
      assert.equal(packets.length, 1);
      const packet = JSON.parse(packets[0]);
      assert.equal(packet.success, exit === 0);
      assert.equal(json(join(directory, "target/nextest/pr/timing.json")).exit_code, exit);
      assert.equal(
        workflowStep("Upload PR Rust build evidence").if,
        "${{ always() && github.event_name == 'pull_request' }}",
      );
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  }
});
