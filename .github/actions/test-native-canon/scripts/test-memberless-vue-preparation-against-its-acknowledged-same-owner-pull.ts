import { readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { join } from "node:path";
const directory = process.argv[2];
const stdout = readFileSync(join(directory, "test.stdout"));
const stderr = readFileSync(join(directory, "test.stderr"));
const status = Number(readFileSync(join(directory, "test.status"), "utf8").trim());
const name =
  "lsp_client::editor_lsp::configured_project::shared_transport::tests::native::replay::actual_memberless_vue_preparation_reattaches_same_editor_owner";
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
if (executed !== 1)
  throw new Error("exact memberless preparation law did not execute once successfully");
for (const file of [
  "without-editor/prepared-vs-native.json",
  "without-editor/materialized-tsconfig.json",
  "without-editor/materialized-ui-package.json",
  "with-editor/prepared-vs-native.json",
  "with-editor/materialized-tsconfig.json",
  "with-editor/materialized-ui-package.json",
]) {
  JSON.parse(readFileSync(join(directory, file), "utf8"));
}
for (const world of ["ordered-without-editor", "ordered-with-editor"]) {
  for (const file of [
    "prepared-vs-native.json",
    "materialized-tsconfig.json",
    "materialized-ui-package.json",
    "world.json",
    "other-before-app/world.json",
    "other-before-app/materialized-tsconfig.json",
    "other-prepared-vs-native.json",
  ]) {
    JSON.parse(readFileSync(join(directory, world, file), "utf8"));
  }
  let observations = 0;
  for (const file of ["materialized-ui-package.json", "materialized-ui-package.absent.json"]) {
    let value;
    try {
      value = JSON.parse(readFileSync(join(directory, world, "other-before-app", file), "utf8"));
    } catch (error) {
      if (error instanceof Error && "code" in error && error.code === "ENOENT") continue;
      throw error;
    }
    if (
      file === "materialized-ui-package.absent.json" &&
      (value.status !== "absent-after-materialization" ||
        typeof value.path !== "string" ||
        value.path.length === 0)
    ) {
      throw new Error("invalid first-world UI absence observation");
    }
    observations++;
  }
  if (observations !== 1)
    throw new Error("exactly one first-world UI package observation required");
}
