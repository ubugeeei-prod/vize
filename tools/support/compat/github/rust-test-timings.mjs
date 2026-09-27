import { execFileSync } from "node:child_process";
import {
  appendFileSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";

const directory = path.join(process.env.RUNNER_TEMP, "rust-test-timings");
mkdirSync(directory, { recursive: true });
const errors = [];
const runPhase = process.env.RUN_PHASE || "execution-and-doctests";
const phases = [
  ["build", process.env.BUILD_OUTCOME],
  ["run", process.env.RUN_OUTCOME],
].map(([phase, outcome]) => {
  const file = path.join(directory, `${phase}.json`);
  const result = { phase, outcome: outcome || "missing" };
  if (!existsSync(file)) {
    result.measurement = "missing";
    if (outcome === "success" || outcome === "failure") {
      errors.push(`Missing ${phase} phase record.`);
    }
    return result;
  }
  try {
    const record = JSON.parse(readFileSync(file, "utf8"));
    if (
      record.phase !== (phase === "build" ? "build" : runPhase) ||
      !Number.isInteger(record.elapsed_seconds) ||
      record.elapsed_seconds < 0 ||
      !Number.isInteger(record.exit_code) ||
      record.exit_code < 0 ||
      record.exit_code > 255 ||
      (outcome === "success" && record.exit_code !== 0) ||
      (outcome === "failure" && record.exit_code === 0)
    ) {
      throw new Error("Invalid phase, elapsed time or exit status.");
    }
    return { ...record, outcome: result.outcome };
  } catch (error) {
    errors.push(`Invalid ${phase} record: ${error.message}`);
    return { ...result, measurement: "invalid" };
  }
});

const identity = {
  event: process.env.GITHUB_EVENT_NAME,
  run_id: process.env.GITHUB_RUN_ID,
  attempt: process.env.GITHUB_RUN_ATTEMPT,
  event_head_sha: process.env.SOURCE_SHA,
  cache_namespace: process.env.CACHE_NAMESPACE,
  profile: process.env.CARGO_TEST_PROFILE || "test",
  cargo_build_jobs: 12,
  rust_test_threads: 4,
  build: process.env.BUILD_COMMAND || "cargo test --workspace --no-run --timings",
  run: process.env.RUN_COMMAND || "cargo test --workspace",
};
writeFileSync(path.join(directory, "identity.json"), `${JSON.stringify(identity, null, 2)}\n`);
for (const [file, command, args] of [
  ["checked-out-sha.txt", "git", ["rev-parse", "HEAD"]],
  ["rustc.txt", "rustc", ["-Vv"]],
  ["cargo.txt", "cargo", ["-V"]],
]) {
  writeFileSync(path.join(directory, file), execFileSync(command, args));
}
const summary = `${JSON.stringify(phases, null, 2)}\n`;
writeFileSync(path.join(directory, "summary.json"), summary);
appendFileSync(
  process.env.GITHUB_STEP_SUMMARY,
  `### Rust test phase measurements\n\n${runPhase === "doctests" ? "Workspace execution is measured independently in four shard JUnit/timing artifacts. This phase contains full doctests." : "Execution includes doctests and residual compilation."}\n\n\`\`\`json\n${summary}\`\`\`\n`,
);

const html = "target/cargo-timings/cargo-timing.html";
if (existsSync(path.join(directory, "build-started")) && existsSync(html)) {
  copyFileSync(html, path.join(directory, "cargo-timing.html"));
} else if (process.env.BUILD_OUTCOME === "success") {
  errors.push("Missing fresh Cargo timing HTML.");
}
if (errors.length > 0) {
  console.error(errors.join("\n"));
  process.exitCode = 1;
}
