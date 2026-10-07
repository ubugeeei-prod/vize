import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse } from "yaml";
import { createArchiveReceipt } from "../../tools/support/compat/github/rust-test-archive.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const workflow = (path) => parse(readFileSync(join(root, path), "utf8"));
const rust = workflow(".github/workflows/pr-rust-checks.yml");
const differential = workflow(".github/workflows/pr-rust-differential.yml");
const job = differential.jobs.differential;
const recipePath = ".github/actions/test-rust-workspace-differential";
const recipe = workflow(`${recipePath}/action.yml`);

await test("full Rust differential checks and shards independently require the same completed producer", () => {
  const sibling = rust.jobs["merge-rust-differential"];
  assert.equal(sibling.needs, "merge-rust-source");
  assert.equal(sibling.if, "${{ github.event_name == 'merge_group' && inputs.run-rust }}");
  assert.equal(sibling.uses, "./.github/workflows/pr-rust-differential.yml");
  assert.deepEqual(sibling.with, { "run-rust": "${{ inputs.run-rust }}" });
  assert.deepEqual(rust.jobs["pr-rust-shard"].needs, ["pr-rust-build", "merge-rust-source"]);
  assert(rust.jobs["rust-source-report"].needs.includes("merge-rust-differential"));
  assert(!rust.jobs["merge-rust-source"].steps.some((step) => step.uses === `./${recipePath}`));
  assert.equal(job.if, sibling.if);
  assert.deepEqual(job.env, rust.jobs["merge-rust-source"].env);
  assert.equal(job["continue-on-error"], undefined);
  assert.equal(differential.permissions.contents, "read");
  const producer = rust.jobs["merge-rust-source"].steps;
  const sameTools = [
    "actions/checkout@",
    "dtolnay/rust-toolchain@",
    "wild-linker/action@",
    "voidzero-dev/setup-vp@",
    "./.github/actions/setup-rust-sticky-cache",
    "./.github/actions/setup-rust-script",
    "taiki-e/install-action@",
  ];
  for (const prefix of sameTools) {
    const original = producer.find((step) => step.uses?.startsWith(prefix));
    const actual = job.steps.find((step) => step.uses?.startsWith(prefix));
    assert.equal(actual.uses, original.uses);
    assert.deepEqual(actual.with, original.with);
  }
  const upload = producer.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  const download = job.steps.find((step) => step.uses?.startsWith("actions/download-artifact@"));
  assert.equal(download.with.name, upload.with.name);
  assert.equal(download.with["digest-mismatch"], "error");
  const verify = job.steps.findIndex((step) => step.name === "Verify Rust archive identity");
  const restore = job.steps.findIndex(
    (step) => step.name === "Restore the exact completed workspace archive",
  );
  const tail = job.steps.findIndex((step) => step.uses === `./${recipePath}`);
  const coverage = job.steps.findIndex((step) => step.name === "Check fixture coverage");
  assert(verify >= 0 && verify < restore && restore < tail && tail < coverage);
  assert.equal(job.steps[tail].with["workspace-already-tested"], "true");
  assert.equal(
    job.steps[coverage].run,
    "rust-script tools/commands/ci/github/write-coverage-summary.rs",
  );
});

function expandedRecipe() {
  return recipe.runs.steps.flatMap((step) => {
    if (step.if === "${{ inputs.workspace-already-tested != 'true' }}") return [];
    if (step.if === "${{ github.event_name != 'merge_group' }}") return [];
    if (!step.uses) return [step];
    assert.equal(step.uses, "./.github/actions/test-native-navigation");
    return workflow(`${step.uses}/action.yml`).runs.steps.map((child) => ({
      ...child,
      env: { ...step.env, ...child.env },
    }));
  });
}

async function fixture() {
  const cwd = realpathSync(mkdtempSync(join(tmpdir(), "vize-rust-differential-")));
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  git("init", "-q");
  git("config", "user.name", "CI Test");
  git("config", "user.email", "ci@example.invalid");
  writeFileSync(join(cwd, "source"), "source\n");
  git("add", "source");
  git("commit", "-qm", "fixture");
  const helper = "tools/support/compat/github/rust-test-archive.mjs";
  mkdirSync(dirname(join(cwd, helper)), { recursive: true });
  copyFileSync(join(root, helper), join(cwd, helper));
  const archiveRoot = join(cwd, ".artifacts/rust-test-archive");
  mkdirSync(archiveRoot, { recursive: true });
  const archive = join(archiveRoot, "tests.tar.zst");
  writeFileSync(archive, "exact producer archive bytes");
  const receiptPath = join(archiveRoot, "receipt.json");
  const context = {
    cwd,
    nextestVersion: "cargo-nextest 0.9.146",
    rustcVersion: "rustc 1.98.0 (fixture)",
    env: job.env,
  };
  const receipt = await createArchiveReceipt(archive, context);
  writeFileSync(receiptPath, JSON.stringify(receipt));
  const bin = join(cwd, "bin");
  const log = join(cwd, "commands.jsonl");
  mkdirSync(bin);
  writeFileSync(log, "");
  writeFileSync(join(bin, "rustc"), "#!/bin/sh\nprintf 'rustc 1.98.0 (fixture)\\n'\n", {
    mode: 0o755,
  });
  const simulated = `#!/bin/sh
exec "$NODE" -e 'const fs=require("node:fs");const args=process.argv.slice(1);if(args.join(" ")==="nextest --version"){console.log("cargo-nextest 0.9.146");process.exit(0);}fs.appendFileSync(process.env.LOG,JSON.stringify({tool:process.env.TOOL,args,tsgo:process.env.VIZE_TEST_REQUIRE_TSGO,disable:process.env.VIZE_TEST_DISABLE_TSGO,iterations:process.env.VIZE_NUXT_CONFIG_ITERATIONS,production:process.env.VIZE_VUE_RUNTIME_PRODUCTION})+"\\n");if(process.env.VIZE_TEST_REQUIRE_TSGO!=="1"||Object.hasOwn(process.env,"VIZE_TEST_DISABLE_TSGO")||process.env.VIZE_NUXT_CONFIG_ITERATIONS!=="100")process.exit(92);if(args.includes(process.env.FAIL))process.exit(42);' -- "$@"
`;
  writeFileSync(
    join(bin, "cargo"),
    simulated.replace('exec "$NODE"', 'export TOOL=cargo\nexec "$NODE"'),
    {
      mode: 0o755,
    },
  );
  writeFileSync(
    join(bin, "rust-script"),
    simulated.replace('exec "$NODE"', 'export TOOL=rust-script\nexec "$NODE"'),
    { mode: 0o755 },
  );
  const verify = job.steps.find((step) => step.name === "Verify Rust archive identity");
  const restore = job.steps.find(
    (step) => step.name === "Restore the exact completed workspace archive",
  );
  const coverage = job.steps.find((step) => step.name === "Check fixture coverage");
  const script = [verify, restore, ...expandedRecipe(), coverage]
    .map((step) => {
      const env = Object.entries(step.env ?? {})
        .map(([key, value]) => `export ${key}='${value.replaceAll("'", "'\\''")}'`)
        .join("\n");
      return `(\n${env}\n${step.run}\n)`;
    })
    .join("\n");
  const execute = (failure = "no-such-command", extraEnv = {}) => {
    const env = {
      ...process.env,
      ...job.env,
      PATH: `${bin}:${dirname(process.execPath)}:${process.env.PATH ?? ""}`,
      NODE: process.execPath,
      LOG: log,
      GITHUB_WORKSPACE: cwd,
      RUNNER_TEMP: cwd,
      FAIL: failure,
    };
    delete env.VIZE_TEST_DISABLE_TSGO;
    return spawnSync("/bin/bash", ["--noprofile", "--norc", "-eo", "pipefail", "-c", script], {
      cwd,
      encoding: "utf8",
      env: { ...env, ...extraEnv },
    });
  };
  return {
    cwd,
    archive,
    receipt,
    receiptPath,
    execute,
    calls: () => readFileSync(log, "utf8").trim().split("\n").filter(Boolean).map(JSON.parse),
    cleanup: () => rmSync(cwd, { recursive: true, force: true }),
  };
}

await test("the differential sibling refuses foreign or corrupt archives before running any feature gate", async () => {
  for (const mutation of ["sha", "tree", "workspaceRoot", "archiveSha256", "bytes", "runtime"]) {
    const f = await fixture();
    try {
      if (mutation === "bytes") writeFileSync(f.archive, "corrupt bytes");
      else if (mutation !== "runtime")
        writeFileSync(f.receiptPath, JSON.stringify({ ...f.receipt, [mutation]: "foreign" }));
      const result = f.execute(
        undefined,
        mutation === "runtime" ? { VIZE_TEST_DISABLE_TSGO: "1" } : {},
      );
      assert.notEqual(result.status, 0, mutation);
      assert.deepEqual(f.calls(), [], mutation);
      assert.match(result.stderr, /mismatch|pinned CI toolchain and TSGO runtime envelope/);
    } finally {
      f.cleanup();
    }
  }
});

await test("the restored sibling executes the original feature recipe and propagates every failure", async () => {
  for (const failure of [
    "no-such-command",
    "source_folding::tests::",
    "davinci_dom_corpus",
    "tools/commands/ci/github/write-coverage-summary.rs",
  ]) {
    const f = await fixture();
    try {
      const result = f.execute(failure);
      assert.equal(result.status, failure === "no-such-command" ? 0 : 42, result.stderr);
      const calls = f.calls();
      assert.deepEqual(calls[0].args, [
        "nextest",
        "list",
        "--archive-file",
        ".artifacts/rust-test-archive/tests.tar.zst",
        "--extract-to",
        f.cwd,
        "--extract-overwrite",
        "--workspace-remap",
        f.cwd,
        "--profile",
        "full",
        "--message-format",
        "json",
      ]);
      assert(
        calls.every(
          (call) => call.tsgo === "1" && call.disable === undefined && call.iterations === "100",
        ),
      );
      if (failure === "no-such-command") {
        assert.equal(calls.at(-1).tool, "rust-script");
        assert.equal(calls.at(-2).args.at(-1), "vue_ssr_render");
        assert.equal(calls.at(-2).production, "1");
        assert(!calls.some((call) => call.args.join(" ") === "test --workspace"));
      } else assert(calls.at(-1).args.includes(failure));
    } finally {
      f.cleanup();
    }
  }
});
