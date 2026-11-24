import { readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { join } from "node:path";
const directory = process.argv[2];
const stdout = readFileSync(join(directory, "test.stdout"));
const stderr = readFileSync(join(directory, "test.stderr"));
const status = Number(readFileSync(join(directory, "test.status"), "utf8").trim());
const name =
  "lsp_client::editor_lsp::snapshot_source::tests::native::actual_native_compiler_kind_reads_match_whole_ts_and_generated_vue_pull_packets";
const executed =
  stdout.toString("utf8").includes(`test ${name} ... ok`) &&
  /^test result: ok\. 1 passed; 0 failed; 0 ignored; /m.test(stdout.toString("utf8"))
    ? 1
    : 0;
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
writeFileSync(
  join(directory, "test-run.json"),
  JSON.stringify(
    {
      source: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
      githubSha: process.env.GITHUB_SHA,
      run: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
      status,
      executed,
      name,
      stdoutSha256: digest(stdout),
      stderrSha256: digest(stderr),
    },
    null,
    2,
  ) + "\n",
);
if (status !== 0) process.exit(status);
if (executed !== 1) throw new Error("exact compiler-kind law did not execute once successfully");
for (const file of ["provider.json", "source.ts.json", "View.vue.ts.json"]) {
  JSON.parse(readFileSync(join(directory, file), "utf8"));
}
