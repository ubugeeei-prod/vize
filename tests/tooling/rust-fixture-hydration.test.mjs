import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { test } from "node:test";

const yaml = createRequire(import.meta.url)("yaml");
const fixturePath = "tests/_fixtures/_git/element-plus";
const action = yaml.parse(
  readFileSync(
    new URL("../../.github/actions/test-rust-workspace-differential/action.yml", import.meta.url),
    "utf8",
  ),
);
const hydration = action.runs.steps.find(
  (step) => step.name === "Hydrate pinned Element Plus fixture",
);

void test("shared native consumers hydrate the original pinned fixture before any test", () => {
  const nativeRecipe = readFileSync(
    new URL("../../tools/support/compat/github/native-syntax-semantics.sh", import.meta.url),
    "utf8",
  );
  assert(hydration, "an uninitialized fixture otherwise resolves the parent repository HEAD");
  assert.equal(action.runs.steps[0], hydration);
  assert.equal(hydration.if, undefined, "archive-backed consumers also require the fixture");
  assert.equal(hydration.shell, "bash");
  assert.equal(
    hydration.run.trim(),
    nativeRecipe.split("\n").find((line) => line.startsWith("git submodule update ")),
  );
});

void test("the shared hydration checks out the Gitlink rather than the repository or remote HEAD", () => {
  assert(hydration, "shared Rust tests require the original fixture Gitlink");
  const directory = mkdtempSync(join(tmpdir(), "vize-fixture-hydration-"));
  const env = {
    ...process.env,
    GIT_CONFIG_NOSYSTEM: "1",
    GIT_CONFIG_GLOBAL: "/dev/null",
    GIT_ALLOW_PROTOCOL: "file",
  };
  const git = (cwd, ...args) =>
    execFileSync("git", args, {
      cwd,
      env,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  const initialize = (cwd) => {
    mkdirSync(cwd, { recursive: true });
    git(cwd, "init", "--initial-branch=main");
    git(cwd, "config", "user.name", "Fixture test");
    git(cwd, "config", "user.email", "fixture@example.invalid");
    git(cwd, "config", "commit.gpgsign", "false");
  };
  try {
    const upstream = join(directory, "upstream");
    initialize(upstream);
    writeFileSync(join(upstream, "fixture.vue"), "<template>original fixture</template>\n");
    git(upstream, "add", "fixture.vue");
    git(upstream, "commit", "-qm", "Original fixture");
    const pinned = git(upstream, "rev-parse", "HEAD");
    writeFileSync(join(upstream, "fixture.vue"), "<template>later upstream</template>\n");
    git(upstream, "commit", "-qam", "Later upstream fixture");
    assert.notEqual(git(upstream, "rev-parse", "HEAD"), pinned);

    const consumer = join(directory, "consumer");
    initialize(consumer);
    writeFileSync(
      join(consumer, ".gitmodules"),
      `[submodule "${fixturePath}"]\n\tpath = ${fixturePath}\n\turl = ${pathToFileURL(upstream).href}\n`,
    );
    git(consumer, "add", ".gitmodules");
    git(consumer, "update-index", "--add", "--cacheinfo", `160000,${pinned},${fixturePath}`);
    git(consumer, "commit", "-qm", "Pin the original fixture");
    const fixture = join(consumer, fixturePath);
    mkdirSync(fixture, { recursive: true });
    assert.equal(git(fixture, "rev-parse", "HEAD"), git(consumer, "rev-parse", "HEAD"));
    assert.notEqual(git(fixture, "rev-parse", "HEAD"), pinned);

    execFileSync("bash", ["-euo", "pipefail", "-c", hydration.run], {
      cwd: consumer,
      env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    assert.equal(git(fixture, "rev-parse", "HEAD"), pinned);
    assert.equal(
      readFileSync(join(fixture, "fixture.vue"), "utf8"),
      "<template>original fixture</template>\n",
    );
    assert.match(
      git(consumer, "ls-tree", "HEAD", fixturePath),
      new RegExp(`160000 commit ${pinned}`),
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
