import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parse } from "yaml";

const root = fileURLToPath(new URL("../../", import.meta.url));
const workflow = parse(readFileSync(join(root, ".github/workflows/pr-source-checks.yml"), "utf8"));
const job = workflow.jobs["pr-source-plan"];
const checkoutInputs = job.steps.find((step: { uses?: string }) =>
  step.uses?.startsWith("actions/checkout@"),
)?.with;
const comparisonStep = job.steps.find(
  (step: { name?: string }) => step.name === "Resolve and fetch comparison base",
);
const sourceStep = job.steps.find((step: { name?: string }) => step.name === "Plan source checks");
const expansionStep = job.steps.find(
  (step: { name?: string }) => step.name === "Expand the source planner checkout",
);
const sparseCheckoutPath = ".github/source-plan-sparse-checkout";
const comparisonPath = "tools/support/compat/github/comparison-base.ts";
const sourcePath = "tools/support/compat/github/plan-source-checks.ts";
const runtimeFiles = [
  sparseCheckoutPath,
  "Cargo.lock",
  "package.json",
  comparisonPath,
  sourcePath,
].sort();

function git(cwd: string, ...args: string[]) {
  return execFileSync("git", args, {
    cwd,
    encoding: "utf8",
    stdio: ["pipe", "pipe", "pipe"],
  }).trim();
}

function put(cwd: string, file: string, value: string) {
  mkdirSync(dirname(join(cwd, file)), { recursive: true });
  writeFileSync(join(cwd, file), value);
}

function files(cwd: string, prefix = ""): string[] {
  return readdirSync(join(cwd, prefix), { withFileTypes: true })
    .flatMap((entry) => {
      if (entry.name === ".git") return [];
      const file = prefix ? `${prefix}/${entry.name}` : entry.name;
      return entry.isDirectory() ? files(cwd, file) : [file];
    })
    .sort();
}

function fixture(scratch: string) {
  const origin = join(scratch, "origin");
  mkdirSync(origin);
  git(origin, "init", "-q", "--initial-branch=main");
  for (const [key, value] of [
    ["user.name", "CI fixture"],
    ["user.email", "ci@example.invalid"],
    ["commit.gpgsign", "false"],
    ["core.hooksPath", "/dev/null"],
    ["uploadpack.allowFilter", "true"],
  ])
    git(origin, "config", key, value);
  for (const file of runtimeFiles.filter((file) => file !== "Cargo.lock"))
    put(origin, file, readFileSync(join(root, file), "utf8"));
  put(origin, "playground/Deleted.vue", "<template>deleted</template>\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "initial planner and outside source");
  const eventBase = git(origin, "rev-parse", "HEAD");
  put(origin, "Cargo.lock", "version = 4\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "advance the event base");
  const mergeBase = git(origin, "rev-parse", "HEAD");
  git(origin, "switch", "-qc", "proposal", eventBase);
  rmSync(join(origin, "playground/Deleted.vue"));
  put(origin, "playground/Added.vue", "<template>added</template>\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "change source outside the planner checkout");
  const prHead = git(origin, "rev-parse", "HEAD");
  git(origin, "switch", "-q", "main");
  git(origin, "merge", "--no-ff", "proposal", "-m", "synthetic pull request merge");
  return { origin, eventBase, mergeBase, prHead, head: git(origin, "rev-parse", "HEAD") };
}

function checkout(
  scratch: string,
  name: string,
  source: ReturnType<typeof fixture>,
  selected: boolean,
) {
  const cwd = join(scratch, name);
  mkdirSync(cwd);
  git(cwd, "init", "-q");
  git(cwd, "remote", "add", "origin", pathToFileURL(source.origin).href);
  const sparse: string | undefined = selected ? checkoutInputs["sparse-checkout"] : undefined;
  assert.equal(checkoutInputs["fetch-depth"], undefined);
  git(
    cwd,
    "fetch",
    "--no-tags",
    "--depth=1",
    ...(sparse ? ["--filter=blob:none"] : []),
    "origin",
    source.head,
  );
  if (sparse) {
    assert.equal(checkoutInputs["sparse-checkout-cone-mode"], false);
    git(cwd, "config", "core.sparseCheckout", "true");
    put(cwd, ".git/info/sparse-checkout", `${sparse.trim()}\n`);
  }
  git(cwd, "checkout", "-q", "--detach", "FETCH_HEAD");
  if (sparse) {
    assert.equal(sparse, `/${sparseCheckoutPath}`);
    execFileSync("git", ["sparse-checkout", "set", "--no-cone", "--stdin"], {
      cwd,
      input: readFileSync(join(cwd, sparseCheckoutPath)),
      stdio: ["pipe", "pipe", "pipe"],
    });
  }
  return cwd;
}

function comparison(
  cwd: string,
  output: string,
  event: "pull_request" | "merge_group",
  source: ReturnType<typeof fixture>,
) {
  const result = spawnSync(process.execPath, [comparisonPath], {
    cwd,
    encoding: "utf8",
    env: {
      ...process.env,
      GITHUB_EVENT_NAME: event,
      EVENT_BASE_SHA: source.eventBase,
      GITHUB_SHA: source.head,
      PR_HEAD_SHA: source.prHead,
      GITHUB_OUTPUT: output,
    },
  });
  assert.equal(result.status, 0, result.stderr);
  const base = /^base=([0-9a-f]{40})$/m.exec(readFileSync(output, "utf8"))?.[1];
  assert.ok(base);
  git(cwd, "fetch", "--no-tags", "--depth=1", "origin", base);
  return { base, stdout: result.stdout };
}

function sourcePlan(
  cwd: string,
  output: string,
  event: "pull_request" | "merge_group",
  base: string,
  head: string,
) {
  const result = spawnSync(process.execPath, [sourcePath, base, head, event], {
    cwd,
    encoding: "utf8",
    env: { ...process.env, GITHUB_OUTPUT: output },
  });
  assert.equal(result.status, 0, result.stderr);
  return { output: readFileSync(output, "utf8"), stdout: result.stdout };
}

await test("the source plan keeps complete Git selection outside its sparse worktree", async (t) => {
  const scratch = mkdtempSync(join(tmpdir(), "vize-source-plan-checkout-"));
  t.after(() => rmSync(scratch, { recursive: true, force: true }));
  const source = fixture(scratch);
  const full = checkout(scratch, "full", source, false);
  const selected = checkout(scratch, "selected", source, true);
  assert.equal(job["timeout-minutes"], 5);
  assert.equal(
    expansionStep.run,
    `git sparse-checkout set --no-cone --stdin < ${sparseCheckoutPath}`,
  );
  assert.match(comparisonStep.run, /git fetch --no-tags --depth=1 origin/);
  assert.match(sourceStep.run, /plan-pr-checks\.ts/);
  assert.equal(git(selected, "rev-parse", "--is-shallow-repository"), "true");
  assert.notEqual(
    spawnSync("git", ["cat-file", "-e", `${source.eventBase}^{commit}`], {
      cwd: selected,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
    }).status,
    0,
  );
  assert.deepEqual(files(selected), runtimeFiles);
  assert.ok(files(full).includes("playground/Added.vue"));
  assert.equal(git(selected, "config", "--get", "remote.origin.partialclonefilter"), "blob:none");

  for (const event of ["pull_request", "merge_group"] as const)
    await t.test(`${event} preserves the comparison and source selection`, () => {
      const selectedComparison = comparison(
        selected,
        join(scratch, `selected-${event}-comparison`),
        event,
        source,
      );
      const fullComparison = comparison(
        full,
        join(scratch, `full-${event}-comparison`),
        event,
        source,
      );
      assert.deepEqual(selectedComparison, fullComparison);
      assert.equal(
        selectedComparison.base,
        event === "pull_request" ? source.mergeBase : source.eventBase,
      );
      const actual = sourcePlan(
        selected,
        join(scratch, `selected-${event}-source`),
        event,
        selectedComparison.base,
        source.head,
      );
      const reference = sourcePlan(
        full,
        join(scratch, `full-${event}-source`),
        event,
        fullComparison.base,
        source.head,
      );
      assert.deepEqual(actual, reference);
      const paths = git(selected, "diff", "--name-only", selectedComparison.base, "HEAD");
      assert.match(paths, /playground\/Added\.vue/);
      assert.match(paths, /playground\/Deleted\.vue/);
      assert.ok(!existsSync(join(selected, "playground")));
    });

  for (const helper of [comparisonPath, sourcePath])
    await t.test(`missing ${helper} refuses selection`, () => {
      const path = join(selected, helper);
      const saved = readFileSync(path, "utf8");
      rmSync(path);
      const output = join(scratch, `missing-${helper.replaceAll("/", "-")}`);
      const args =
        helper === comparisonPath
          ? [helper]
          : [helper, source.eventBase, source.head, "merge_group"];
      const result = spawnSync(process.execPath, args, {
        cwd: selected,
        encoding: "utf8",
        env: {
          ...process.env,
          GITHUB_EVENT_NAME: "merge_group",
          EVENT_BASE_SHA: source.eventBase,
          GITHUB_SHA: source.head,
          PR_HEAD_SHA: source.prHead,
          GITHUB_OUTPUT: output,
        },
      });
      assert.notEqual(result.status, 0);
      assert.equal(existsSync(output), false);
      assert.match(result.stderr, /Cannot find module|MODULE_NOT_FOUND/);
      put(selected, helper, saved);
    });
});
