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
const runtimeFiles = [
  "package.json",
  "tools/support/compat/github/comparison-base.ts",
  "tools/support/compat/github/plan-check-inputs.ts",
  "tools/support/github/opinionated-source-inputs.ts",
];
const plannerPath = "tools/support/compat/github/plan-check-inputs.ts";
const workflow = parse(readFileSync(join(root, ".github/workflows/check.yml"), "utf8"));
const job = workflow.jobs["iteration-plan"];
const checkoutInputs = job.steps.find((step: { uses?: string }) =>
  step.uses?.startsWith("actions/checkout@"),
)?.with;
const command = job.steps
  .find((step: { id?: string }) => step.id === "plan")
  .run.trim()
  .split(/\s+/);

function git(cwd: string, ...args: string[]): string {
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

function assertExpectedOutputs(output: string, expected: Record<string, string>) {
  const pairs = output
    .trimEnd()
    .split("\n")
    .map((line) => {
      const pair = /^([a-z][a-z-]*)=(true|false)$/.exec(line);
      assert.ok(pair, `invalid planner output pair: ${line}`);
      return [pair[1], pair[2]] as const;
    });
  const keys = pairs.map(([key]) => key);
  assert.equal(new Set(keys).size, keys.length, "duplicate planner output key");
  assert.deepEqual(Object.fromEntries(pairs), expected);
}

test("expected output pairs retain strict keys without imposing key order", () => {
  const expected = {
    full: "false",
    rust: "false",
    dependencies: "false",
    engines: "false",
    apps: "true",
  };
  const packet = Object.entries(expected)
    .map(([key, value]) => `${key}=${value}\n`)
    .join("");
  const reordered = packet.trimEnd().split("\n").reverse().join("\n") + "\n";
  assertExpectedOutputs(reordered, expected);
  for (const invalid of [
    packet.replace("apps=true\n", ""),
    packet + "unexpected=true\n",
    packet + "apps=true\n",
    packet.replace("apps=true", "apps=1"),
  ])
    assert.throws(() => assertExpectedOutputs(invalid, expected));
});

function fixture(scratch: string) {
  const origin = join(scratch, "origin");
  mkdirSync(origin);
  git(origin, "init", "-q", "--initial-branch=main");
  git(origin, "config", "user.name", "CI fixture");
  git(origin, "config", "user.email", "ci@example.invalid");
  git(origin, "config", "commit.gpgsign", "false");
  git(origin, "config", "core.hooksPath", "/dev/null");
  git(origin, "config", "uploadpack.allowFilter", "true");
  for (const file of runtimeFiles) put(origin, file, readFileSync(join(root, file), "utf8"));
  put(origin, "docs/theme/components/Deleted.tsx", "export const deleted = true;\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "initial runtime and outside source");
  const eventBase = git(origin, "rev-parse", "HEAD");
  put(origin, "Cargo.lock", "base-only dependency change\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "advance the merge base");
  const mergeBase = git(origin, "rev-parse", "HEAD");
  git(origin, "switch", "-qc", "proposal", eventBase);
  rmSync(join(origin, "docs/theme/components/Deleted.tsx"));
  put(origin, "docs/theme/components/Added.tsx", "export const added = true;\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "add and remove authored sources outside the runtime closure");
  const prHead = git(origin, "rev-parse", "HEAD");
  git(origin, "switch", "-q", "main");
  git(origin, "merge", "--no-ff", "proposal", "-m", "synthetic pull request merge");
  return { origin, eventBase, mergeBase, prHead, head: git(origin, "rev-parse", "HEAD") };
}

function checkout(
  scratch: string,
  name: string,
  source: ReturnType<typeof fixture>,
  actualWorkflow: boolean,
) {
  const cwd = join(scratch, name);
  mkdirSync(cwd);
  git(cwd, "init", "-q");
  git(cwd, "remote", "add", "origin", pathToFileURL(source.origin).href);
  const sparse: string | undefined = actualWorkflow ? checkoutInputs["sparse-checkout"] : undefined;
  const depth = actualWorkflow ? checkoutInputs["fetch-depth"] : 2;
  assert.equal(depth, 2, "retain the verified merge-parent checkout contract");
  // The pinned checkout action selects blob:none for sparse input, then writes
  // the non-cone patterns before checking out the fetched commit.
  git(
    cwd,
    "fetch",
    "--no-tags",
    "--depth=2",
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
  return cwd;
}

function plan(
  cwd: string,
  scratch: string,
  label: string,
  event: string,
  source: ReturnType<typeof fixture>,
) {
  const output = join(scratch, `${label}.output`);
  const summary = join(scratch, `${label}.summary`);
  assert.equal(command[0], "node");
  const result = spawnSync(process.execPath, command.slice(1), {
    cwd,
    encoding: "utf8",
    env: {
      ...process.env,
      GITHUB_EVENT_NAME: event,
      EVENT_BASE_SHA: source.eventBase,
      GITHUB_SHA: source.head,
      PR_HEAD_SHA: source.prHead,
      GITHUB_OUTPUT: output,
      GITHUB_STEP_SUMMARY: summary,
    },
  });
  return { result, output, summary };
}

await test("the actual iteration-plan checkout preserves complete planning from its runtime closure", async (t) => {
  const scratch = mkdtempSync(join(tmpdir(), "vize-iteration-checkout-"));
  t.after(() => rmSync(scratch, { recursive: true, force: true }));
  const source = fixture(scratch);
  const full = checkout(scratch, "full", source, false);
  const selected = checkout(scratch, "workflow", source, true);
  assert.equal(job["timeout-minutes"], 2);
  for (const cwd of [full, selected]) {
    assert.equal(git(cwd, "rev-parse", "--is-shallow-repository"), "true");
    assert.notEqual(
      spawnSync("git", ["cat-file", "-e", `${source.eventBase}^{commit}`], {
        cwd,
        env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
      }).status,
      0,
    );
    assert.deepEqual(
      git(cwd, "cat-file", "-p", "HEAD")
        .split("\n")
        .filter((line) => line.startsWith("parent ")),
      [`parent ${source.mergeBase}`, `parent ${source.prHead}`],
    );
  }
  for (const event of ["pull_request", "merge_group", "push", "schedule", "workflow_dispatch"]) {
    await t.test(`${event} keeps full-checkout outputs and the exact comparison base`, () => {
      const actual = plan(selected, scratch, `selected-${event}`, event, source);
      const reference = plan(full, scratch, `full-${event}`, event, source);
      assert.equal(reference.result.status, 0, reference.result.stderr);
      assert.equal(actual.result.status, 0, actual.result.stderr);
      const output = readFileSync(actual.output, "utf8");
      assert.equal(output, readFileSync(reference.output, "utf8"));
      assert.equal(actual.result.stdout, reference.result.stdout);
      assert.equal(readFileSync(actual.summary, "utf8"), readFileSync(reference.summary, "utf8"));
      const fullQualification = ["schedule", "workflow_dispatch"].includes(event);
      const dependencies = fullQualification || event !== "pull_request";
      assertExpectedOutputs(output, {
        full: String(fullQualification),
        rust: String(fullQualification),
        dependencies: String(dependencies),
        engines: String(dependencies),
        apps: "true",
      });
      if (!fullQualification) {
        const base = event === "pull_request" ? source.mergeBase : source.eventBase;
        const paths = git(selected, "diff", "--name-only", "--no-renames", base, "HEAD").split(
          "\n",
        );
        assert.ok(paths.includes("docs/theme/components/Added.tsx"));
        assert.ok(paths.includes("docs/theme/components/Deleted.tsx"));
        assert.match(
          readFileSync(actual.summary, "utf8"),
          new RegExp(`${paths.length} changed paths against ${base}`),
        );
      }
    });
  }
  await t.test(
    "only runtime files are materialized and later base fetches retain blob filtering",
    () => {
      assert.deepEqual(files(selected), runtimeFiles);
      assert.equal(
        git(selected, "config", "--get", "remote.origin.partialclonefilter"),
        "blob:none",
      );
      assert.equal(git(selected, "config", "--get", "remote.origin.promisor"), "true");
      assert.ok(files(full).includes("Cargo.lock"));
      assert.ok(files(full).includes("docs/theme/components/Added.tsx"));
      for (const revision of [
        "HEAD:docs/theme/components/Added.tsx",
        `${source.eventBase}:docs/theme/components/Deleted.tsx`,
      ]) {
        const blob = git(selected, "rev-parse", revision);
        for (const [cwd, expected] of [
          [selected, false],
          [full, true],
        ] as const) {
          const exists =
            spawnSync("git", ["cat-file", "-e", blob], {
              cwd,
              env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
            }).status === 0;
          assert.equal(
            exists,
            expected,
            `${revision}: selected paths must not fetch product blobs`,
          );
        }
      }
    },
  );
  for (const helper of runtimeFiles.filter(
    (file) => file.endsWith(".ts") && file !== plannerPath,
  )) {
    await t.test(`missing ${helper} fails before publishing any selection`, () => {
      const saved = readFileSync(join(selected, helper), "utf8");
      rmSync(join(selected, helper));
      try {
        const observed = plan(
          selected,
          scratch,
          `missing-${helper.split("/").at(-1)}`,
          "merge_group",
          source,
        );
        assert.notEqual(observed.result.status, 0);
        assert.match(observed.result.stderr, /ERR_MODULE_NOT_FOUND/);
        assert.equal(existsSync(observed.output), false);
        assert.equal(existsSync(observed.summary), false);
      } finally {
        put(selected, helper, saved);
      }
    });
  }
});
