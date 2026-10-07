import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  artifactRoot,
  assertFixtureCheckout,
  sha256,
} from "../../tools/support/compat/github/canonical-corpus-identity.mjs";
import {
  forcedCheckoutArgs,
  recoverFixtureCheckout,
  validateHydration,
} from "../../tools/support/compat/github/canonical-corpus-hydration.mjs";

const git = (cwd, ...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const initialize = (cwd) => {
  mkdirSync(cwd, { recursive: true });
  git(cwd, "init", "--quiet");
  git(cwd, "config", "user.name", "Fixture control");
  git(cwd, "config", "user.email", "fixture@example.invalid");
};
const commit = (cwd) => {
  git(cwd, "add", ".");
  git(cwd, "commit", "--quiet", "-m", "fixture");
};
const selected = (cwd, gitlinks) => {
  mkdirSync(join(cwd, artifactRoot), { recursive: true });
  writeFileSync(
    join(cwd, artifactRoot, "selected-gitlinks.txt"),
    gitlinks.map((row) => row.path).join("\n") + "\n",
  );
};

await test("a same-HEAD incomplete fixture is forced back to the complete authored checkout", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-recheckout-"));
  const source = join(temporary, "source");
  const root = join(temporary, "root");
  try {
    initialize(source);
    const bytes = "<template> authored bytes </template>\r\n";
    writeFileSync(join(source, "Original.vue"), bytes);
    writeFileSync(join(source, "Second.vue"), "<template>{{ second }}</template>\n");
    commit(source);
    initialize(root);
    const path = "tests/_fixtures/_git/original";
    git(root, "-c", "protocol.file.allow=always", "submodule", "add", "--quiet", source, path);
    commit(root);
    const fixture = join(root, path);
    const gitlinks = [{ path, sha: git(source, "rev-parse", "HEAD") }];
    selected(root, gitlinks);
    rmSync(join(fixture, "Original.vue"));
    assert.equal(
      git(fixture, "rev-parse", "HEAD"),
      gitlinks[0].sha,
      "HEAD-only inventory cannot prove this missing file",
    );
    const plan = {
      sha: git(root, "rev-parse", "HEAD"),
      tree: git(root, "rev-parse", "HEAD^{tree}"),
      gitlinks,
      gitlinksSha256: sha256(JSON.stringify(gitlinks)),
    };
    recoverFixtureCheckout(root, plan);
    assert.equal(readFileSync(join(fixture, "Original.vue"), "utf8"), bytes);
    assert.equal(git(fixture, "rev-parse", "HEAD"), gitlinks[0].sha);
    assert.equal(git(fixture, "diff", "--name-only", "HEAD", "--"), "");
    validateHydration(join(root, artifactRoot), plan);
    const receipt = JSON.parse(readFileSync(join(root, artifactRoot, "hydration.json")));
    assert.deepEqual(receipt.args, forcedCheckoutArgs(gitlinks));
    writeFileSync(join(root, artifactRoot, "hydration-stderr.log"), "replaced diagnostics");
    assert.throws(
      () => validateHydration(join(root, artifactRoot), plan),
      /diagnostics were replaced/,
    );
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

await test("forced checkout failures retain raw diagnostics and never become successful corpus evidence", () => {
  const root = mkdtempSync(join(tmpdir(), "canonical-hydration-refusal-"));
  const gitlinks = [{ path: "tests/_fixtures/_git/original", sha: "a".repeat(40) }];
  const plan = {
    sha: "b".repeat(40),
    tree: "c".repeat(40),
    gitlinks,
    gitlinksSha256: sha256(JSON.stringify(gitlinks)),
  };
  try {
    selected(root, gitlinks);
    const stderr = Buffer.from("fatal: fixture acquisition failed\r\n");
    assert.throws(
      () =>
        recoverFixtureCheckout(root, plan, (_command, args) => {
          assert.deepEqual(args, forcedCheckoutArgs(gitlinks));
          return { status: 128, signal: null, stdout: Buffer.alloc(0), stderr };
        }),
      /raw Git diagnostics are retained/,
    );
    const directory = join(root, artifactRoot);
    assert.deepEqual(readFileSync(join(directory, "hydration-stderr.log")), stderr);
    assert.equal(JSON.parse(readFileSync(join(directory, "hydration.json"))).status, 128);
    assert.throws(() => validateHydration(directory, plan), /did not succeed/);
    writeFileSync(join(directory, "selected-gitlinks.txt"), "tests/_fixtures/_git/foreign\n");
    assert.throws(
      () => recoverFixtureCheckout(root, plan, () => assert.fail("No checkout allowed")),
      /selection differs/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("fixture CRLF filter warnings require exact raw blobs, modes and a clean index", () => {
  const root = mkdtempSync(join(tmpdir(), "canonical-fixture-raw-bytes-"));
  try {
    initialize(root);
    git(root, "config", "core.filemode", "true");
    const path = join(root, "authored.json");
    const original = '{"authored":"CRLF fixture"}\r\n';
    writeFileSync(join(root, ".gitattributes"), "*.json -text\n");
    writeFileSync(path, original);
    writeFileSync(join(root, "Original.vue"), "<template>original</template>\n");
    commit(root);
    writeFileSync(join(root, ".gitattributes"), "*.json text eol=lf\n");
    git(root, "add", ".gitattributes");
    git(root, "commit", "--quiet", "-m", "authored upstream attributes");
    git(root, "checkout", "--force", "HEAD");
    assert.equal(readFileSync(path, "utf8"), original);
    assert.equal(git(root, "diff", "--name-only", "--"), "authored.json");
    assertFixtureCheckout(root);
    writeFileSync(path, '{"changed":"unstaged"}\r\n');
    assert.throws(() => assertFixtureCheckout(root), /raw bytes drift/);
    git(root, "add", "authored.json");
    assert.throws(
      () => assertFixtureCheckout(root),
      (error) => error.status === 1,
    );
    git(root, "restore", "--staged", "authored.json");
    writeFileSync(path, original);
    assertFixtureCheckout(root);
    chmodSync(path, 0o755);
    assert.throws(() => assertFixtureCheckout(root), /mode drift/);
    chmodSync(path, 0o644);
    rmSync(path);
    assert.throws(() => assertFixtureCheckout(root), /ENOENT/);
    symlinkSync("Original.vue", path);
    assert.throws(() => assertFixtureCheckout(root), /mode drift/);
    rmSync(path);
    writeFileSync(path, original);
    assertFixtureCheckout(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
