import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  captureCorpus,
  collectFiles,
  corpusPlan,
  parseGitlinks,
  sameCorpus,
  sha256,
  validateFiles,
} from "../../tools/support/compat/github/canonical-corpus-identity.mjs";
import { validateObserverLog } from "../../tools/support/compat/github/canonical-corpus-observer.mjs";

const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim();
const historicalFiles = 42998;
const historicalGitlinks = 147;
const original = execFileSync("git", ["ls-tree", "-rz", "HEAD", "--", "tests/_fixtures/_git"], {
  encoding: "utf8",
});

test("tracked parent drift refuses unchanged HEAD while restored source and committed growth pass", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-parent-source-"));
  const root = join(temporary, "root");
  const fixture = join(temporary, "fixture");
  const run = (cwd, ...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  const commit = (cwd) => {
    run(cwd, "add", ".");
    run(cwd, "commit", "--quiet", "-m", "authored source control");
  };
  try {
    for (const cwd of [root, fixture]) {
      mkdirSync(cwd);
      run(cwd, "init", "--quiet", "--object-format=sha1");
      run(cwd, "config", "user.name", "Source custody control");
      run(cwd, "config", "user.email", "fixture@example.invalid");
    }
    writeFileSync(join(fixture, "Original.vue"), "<template>unchanged authored input</template>\n");
    commit(fixture);
    const path = "tests/_fixtures/_git/original";
    run(root, "-c", "protocol.file.allow=always", "submodule", "add", "--quiet", fixture, path);
    writeFileSync(join(root, "Cargo.lock"), "# authored source control\n");
    writeFileSync(join(root, "rust-toolchain.toml"), '[toolchain]\nchannel="stable"\n');
    const source = join(root, "compiler.rs");
    const originalSource = "fn source() { original(); }\n";
    writeFileSync(source, originalSource);
    commit(root);
    const env = {
      GITHUB_SHA: run(root, "rev-parse", "HEAD"),
      GITHUB_REPOSITORY: "ubugeeei-prod/vize",
      GITHUB_REPOSITORY_ID: "99",
      GITHUB_RUN_ID: "12",
      GITHUB_RUN_ATTEMPT: "1",
      GITHUB_EVENT_NAME: "merge_group",
      GITHUB_REF: "refs/heads/gh-readonly-queue/main/pr-99",
      GITHUB_EVENT_PATH: join(temporary, "event.json"),
      RUNNER_OS: "Linux",
      RUNNER_ARCH: "X64",
      RUNNER_ENVIRONMENT: "self-hosted",
    };
    writeFileSync(
      env.GITHUB_EVENT_PATH,
      JSON.stringify({
        repository: { id: 99, full_name: env.GITHUB_REPOSITORY, default_branch: "main" },
        merge_group: {},
      }),
    );
    const before = captureCorpus(root, env);
    for (const staged of [false, true]) {
      writeFileSync(source, "fn source() { changed_uncommitted_source(); }\n");
      if (staged) run(root, "add", "compiler.rs");
      assert.equal(run(root, "rev-parse", "HEAD"), env.GITHUB_SHA);
      assert.throws(
        () => corpusPlan(root, env),
        (error) => error.status === 1,
      );
      assert.throws(
        () => captureCorpus(root, env),
        (error) => error.status === 1,
      );
      run(root, "restore", "--staged", "compiler.rs");
      run(root, "restore", "compiler.rs");
      writeFileSync(join(root, "generated-output.json"), '{"untracked":"allowed"}\n');
      const restored = captureCorpus(root, env);
      sameCorpus(before.identity, restored.identity);
      assert.deepEqual(restored.files, before.files);
      assert.equal(readFileSync(source, "utf8"), originalSource);
    }
    const checkout = join(root, path);
    writeFileSync(join(checkout, "Added.vue"), "<template>committed growth</template>\n");
    commit(checkout);
    run(root, "add", path);
    run(root, "commit", "--quiet", "-m", "complete committed fixture growth");
    const expanded = captureCorpus(root, { ...env, GITHUB_SHA: run(root, "rev-parse", "HEAD") });
    assert.equal(expanded.files.length, before.files.length + 1);
    assert(expanded.files.some(([name]) => name === "original/Added.vue"));
    assert.deepEqual(
      expanded.files.find(([name]) => name === "original/Original.vue"),
      before.files[0],
    );
    assert.throws(() => sameCorpus(before.identity, expanded.identity));
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

test("fixture object cache is committed-gitlink keyed and read-only for PR/queue sources", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-plan-"));
  const sha = git("rev-parse", "HEAD");
  const env = {
    GITHUB_SHA: sha,
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REPOSITORY_ID: "99",
    GITHUB_RUN_ID: "12",
    GITHUB_RUN_ATTEMPT: "1",
    GITHUB_EVENT_PATH: join(temporary, "event.json"),
    RUNNER_OS: "Linux",
    RUNNER_ARCH: "X64",
    RUNNER_ENVIRONMENT: "self-hosted",
  };
  const repository = { id: 99, full_name: env.GITHUB_REPOSITORY, default_branch: "main" };
  try {
    const cases = [
      [
        "pull_request",
        "refs/pull/99/merge",
        { repository, pull_request: { head: { sha: "e".repeat(40), repo: { id: 199 } } } },
        false,
      ],
      [
        "merge_group",
        "refs/heads/gh-readonly-queue/main/pr-99",
        { repository, merge_group: {} },
        false,
      ],
      ["workflow_dispatch", "refs/heads/main", { repository }, true],
      ["workflow_dispatch", "refs/heads/topic", { repository }, false],
      ["push", "refs/heads/main", { repository, ref: "refs/heads/main", deleted: true }, false],
    ];
    const keys = new Set();
    for (const [event, ref, payload, trusted] of cases) {
      writeFileSync(env.GITHUB_EVENT_PATH, JSON.stringify(payload));
      const context = { ...env, GITHUB_EVENT_NAME: event, GITHUB_REF: ref };
      const plan = corpusPlan(process.cwd(), context);
      assert.equal(plan.cacheTrusted, trusted);
      assert.equal(plan.providerSha, event === "pull_request" ? "e".repeat(40) : sha);
      assert.equal(plan.providerRepositoryId, event === "pull_request" ? 199 : 99);
      assert.deepEqual(plan.gitlinks, parseGitlinks(original));
      assert.equal(plan.tree, git("rev-parse", "HEAD^{tree}"));
      keys.add(plan.cacheKey);
      assert.throws(
        () => corpusPlan(process.cwd(), { ...context, GITHUB_SHA: "b".repeat(40) }),
        /differs from candidate/,
      );
      assert.throws(() => corpusPlan(process.cwd(), { ...context, GITHUB_RUN_ATTEMPT: "0" }));
      assert.throws(() =>
        corpusPlan(process.cwd(), { ...context, GITHUB_REPOSITORY: "other/repo" }),
      );
    }
    assert.equal(keys.size, 1, "Object identity is independent of candidate/run cache outcomes");
    for (const head of [
      {},
      { sha: "e".repeat(40), repo: { id: 0 } },
      { sha: "0".repeat(40), repo: { id: 199 } },
    ]) {
      writeFileSync(env.GITHUB_EVENT_PATH, JSON.stringify({ repository, pull_request: { head } }));
      assert.throws(() =>
        corpusPlan(process.cwd(), {
          ...env,
          GITHUB_EVENT_NAME: "pull_request",
          GITHUB_REF: "refs/pull/99/merge",
        }),
      );
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

test("committed fixture parsing refuses duplicate, conflict and foreign gitlinks", () => {
  assert.equal(
    parseGitlinks(original)
      .map(({ path, sha }) => `160000 commit ${sha}\t${path}\0`)
      .join(""),
    original,
  );
  for (const value of [
    original + original.split("\0")[0] + "\0",
    original.replace("160000 commit", "100644 blob"),
    original.replace("tests/_fixtures/_git/", "../outside/"),
    original.replace(/commit [0-9a-f]{40}/, `commit ${"0".repeat(40)}`),
  ])
    assert.throws(() => parseGitlinks(value));
});

test("whole Vue bytes are observed and the original corpus exclusions remain", () => {
  const root = mkdtempSync(join(tmpdir(), "canonical-files-"));
  try {
    for (const name of ["nested", "node_modules", "_git-worktrees"]) mkdirSync(join(root, name));
    writeFileSync(join(root, "Original.vue"), "<template> authored </template>\r\n");
    writeFileSync(join(root, "nested/Second.vue"), "<template>{{ exact }}</template>\n");
    writeFileSync(join(root, "node_modules/Excluded.vue"), "dependency");
    writeFileSync(join(root, "_git-worktrees/Excluded.vue"), "local");
    writeFileSync(join(root, "Other.txt"), "outside the Vue sweep");
    const before = collectFiles(root);
    assert.deepEqual(
      before.map((row) => row[0]),
      ["Original.vue", "nested/Second.vue"],
    );
    assert.equal(before[0][1], sha256(readFileSync(join(root, "Original.vue"))));
    writeFileSync(join(root, "Original.vue"), "<template>authored</template>\n");
    assert.notDeepEqual(
      collectFiles(root),
      before,
      "Whitespace and EOL byte changes must be visible",
    );
    symlinkSync(join(root, "missing"), join(root, "Broken.vue"));
    assert.throws(() => collectFiles(root), /ENOENT/, "Unreadable entries must not be skipped");
    rmSync(join(root, "Broken.vue"));
    symlinkSync(root, join(root, "nested", "cycle"));
    assert.throws(() => collectFiles(root), /directory cycle/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("full file manifests refuse shrinkage, duplication, byte digest and path drift", () => {
  const files = Array.from({ length: historicalFiles }, (_, index) => [
    `project-${index}/Original.vue`,
    "a".repeat(64),
    31,
    "e".repeat(40),
  ]);
  const committedFiles = files.map(([path, , , blob]) => [path, blob]);
  const digest = validateFiles(files, committedFiles);
  assert.equal(digest, sha256(JSON.stringify(files)));
  for (const mutate of [
    (value) => value.pop(),
    (value) => {
      value[1] = value[0];
    },
    (value) => {
      value[0][0] = "../outside.vue";
    },
    (value) => {
      value[0][1] = "missing";
    },
    (value) => {
      value[0][2] = -1;
    },
  ]) {
    const forged = structuredClone(files);
    mutate(forged);
    assert.throws(() => validateFiles(forged, committedFiles));
  }
  const identity = {
    schema: "vize.canonical-corpus-identity",
    version: 1,
    files: historicalFiles,
    filesSha256: digest,
    committedSha256: "e".repeat(64),
    sha: "b".repeat(40),
    tree: "c".repeat(40),
    runId: 12,
    attempt: 1,
    repository: "ubugeeei-prod/vize",
    event: "merge_group",
    gitlinks: parseGitlinks(original),
    gitlinksSha256: sha256(original),
    modulesSha256: "d".repeat(64),
  };
  sameCorpus(identity, structuredClone(identity));
  for (const key of [
    "sha",
    "providerSha",
    "tree",
    "attempt",
    "runId",
    "gitlinksSha256",
    "filesSha256",
    "files",
    "committedSha256",
  ])
    assert.throws(() => sameCorpus(identity, { ...identity, [key]: "foreign" }));
});

test("historical raw observers are parser controls and all full counters remain required", () => {
  const root = "tests/tooling/fixtures/canonical-observer-logs";
  const capture = JSON.parse(readFileSync(join(root, "capture.json"), "utf8"));
  assert.equal(
    sha256(readFileSync(join(root, "capture.json"))),
    "a3a3e72c48cf66f7935eaff1817fb26ef5a056dd6f483c3f06cc17f8b8f72cbc",
  );
  assert.equal(capture.runId, 37612234749);
  assert.equal(capture.sha, "538370a5db8f4443538145c8ae891e5e84008328");
  const inventory = {
    files: historicalFiles,
    gitlinks: Array.from({ length: historicalGitlinks }),
  };
  const incomplete = JSON.parse(readFileSync(join(root, "incomplete-capture.json"), "utf8"));
  assert.equal(
    sha256(readFileSync(join(root, "incomplete-capture.json"))),
    "001313a57cf595012fd507caa8e0bf0408a85119deb1f22a15628bc85defae31",
  );
  const incompleteLog = readFileSync(join(root, "incomplete-dom.log"));
  assert.equal(incomplete.runId, 37617991549);
  assert.equal(incomplete.sha, "8a9110229414769be71595d7888c6cbd8f3add88");
  assert.equal(sha256(incompleteLog), incomplete.logSha256);
  assert.throws(() => validateObserverLog("dom", incompleteLog, inventory), /whole corpus/);
  for (const observer of ["dom", "ssr-pug", "reach"]) {
    const bytes = readFileSync(join(root, `${observer}.log`));
    assert.equal(sha256(bytes), capture.logs[observer]);
    const counters = validateObserverLog(observer, bytes, inventory);
    assert(counters.length > 0);
    const text = bytes.toString("utf8");
    for (const forged of [
      text.replaceAll("files=42998", "files=42997"),
      text.replaceAll("closure_evidence=true", "closure_evidence=false"),
      text.replaceAll("submodules=147", "submodules=146"),
      text.replaceAll("0 failed; 0 ignored", "0 failed; 1 ignored"),
      text.replaceAll("divergences=0", "divergences=1"),
      text.replaceAll("test result: ok.", "test result: FAILED."),
    ]) {
      assert.throws(
        () => validateObserverLog(observer, Buffer.from(forged), inventory),
        `${observer}: forged counters`,
      );
    }
  }
});
