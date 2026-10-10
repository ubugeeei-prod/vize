import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { deflateSync } from "node:zlib";
import { loadPublicAuthority } from "./authority.ts";
import { sourceOverrideKeys } from "./custody.ts";
import { RawGitRepository } from "./raw-git.ts";

// These objects are deliberately unsigned inert fixtures, never release/source delivery authority.
function fixture(run: (fixture: ReturnType<typeof repository>) => void) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "lsp-raw-git-law-")));
  try {
    run(repository(root));
  } finally {
    fs.rmSync(root, { recursive: true });
  }
}
function repository(root: string) {
  const environment = Object.fromEntries(
    Object.entries(process.env).filter(([key]) => !key.startsWith("GIT_")),
  );
  Object.assign(environment, {
    GIT_AUTHOR_NAME: "Inert Fixture",
    GIT_AUTHOR_EMAIL: "inert@example.invalid",
    GIT_COMMITTER_NAME: "Inert Fixture",
    GIT_COMMITTER_EMAIL: "inert@example.invalid",
    GIT_AUTHOR_DATE: "2026-10-08T00:00:00Z",
    GIT_COMMITTER_DATE: "2026-10-08T00:00:00Z",
  });
  const command = (args: string[], input?: string | Buffer) => {
    const result = spawnSync("git", args, { cwd: root, env: environment, input });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    return result;
  };
  const git = (args: string[], input?: string | Buffer) => {
    const result = command(args, input);
    assert.equal(result.status, 0, result.stderr.toString());
    return result.stdout;
  };
  git(["init", "--object-format=sha1"]);
  const object = (type: string, bytes: string | Buffer) =>
    git(["hash-object", "-w", "--literally", "-t", type, "--stdin"], bytes).toString().trim();
  const tree = (entries: Array<{ name: string; mode: string; type: string; sha: string }>) =>
    git(
      ["mktree"],
      entries.map(({ mode, type, sha, name }) => `${mode} ${type} ${sha}\t${name}\n`).join(""),
    )
      .toString()
      .trim();
  const blob = object("blob", "independently frozen source\n");
  const otherBlob = object("blob", "unrelated source\n");
  const sourceTree = tree([{ name: "file.txt", mode: "100644", type: "blob", sha: blob }]);
  const otherTree = tree([{ name: "file.txt", mode: "100644", type: "blob", sha: otherBlob }]);
  const commit = (treeSha: string, parents: string[], message: string) =>
    git(["commit-tree", treeSha, ...parents.flatMap((sha) => ["-p", sha])], `${message}\n`)
      .toString()
      .trim();
  const source = commit(sourceTree, [], "reviewed identity stand-in");
  const unrelated = commit(otherTree, [], "unrelated cut");
  const cut = commit(sourceTree, [source], "genuine cut");
  const release = commit(sourceTree, [cut], "genuine publication");
  const forged = commit(otherTree, [source], "replacement injects ancestry");
  return {
    root,
    command,
    git,
    object,
    tree,
    commit,
    sourceTree,
    otherTree,
    blob,
    source,
    unrelated,
    cut,
    release,
    forged,
  };
}
function ambient(values: Record<string, string>, run: () => void) {
  const original = new Map(Object.keys(values).map((key) => [key, process.env[key]]));
  try {
    Object.assign(process.env, values);
    run();
  } finally {
    for (const [key, value] of original)
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
  }
}
function rejectConsumer(f: ReturnType<typeof repository>) {
  const receipt = {
    schema: "vize-public-registry-install-v1",
    success: true,
    version: "0.0.0",
    source: { C: f.unrelated, H: f.unrelated, tag: "v0.0.0", R: "1" },
    sourceOverrides: Object.fromEntries(sourceOverrideKeys.map((key) => [key, ""])),
  };
  const bytes = JSON.stringify(receipt);
  const file = path.join(f.root, "inert-receipt.json");
  fs.writeFileSync(file, bytes);
  // All provider/install fields are absent: rejection must precede every provider probe.
  ambient(
    Object.fromEntries(
      [...sourceOverrideKeys, "VIZE_PUBLIC_NATIVE_CUSTODY"].map((key) => [key, ""]),
    ),
    () =>
      assert.throws(
        () =>
          loadPublicAuthority(
            file,
            createHash("sha256").update(bytes).digest("hex"),
            f.root,
            f.source,
          ),
        /actual signed source merge must be included/u,
      ),
  );
}

test("raw ancestry accepts genuine SHA-bound paths and rejects unrelated cuts", () =>
  fixture((f) => {
    const raw = new RawGitRepository(f.root);
    assert.equal(raw.includes(f.source, f.cut), true);
    assert.equal(raw.includes(f.source, f.release), true);
    assert.equal(raw.includes(f.source, f.source), true);
    assert.equal(raw.includes(f.source, f.unrelated), false);
    assert.deepEqual(raw.commit(f.release).parents, [f.cut]);
    rejectConsumer(f);
  }));
test("replace refs cannot add public-cut ancestry or substitute the original source tree", () =>
  fixture((f) => {
    f.git(["replace", f.unrelated, f.forged]);
    assert.equal(f.command(["merge-base", "--is-ancestor", f.source, f.unrelated]).status, 0);
    assert.equal(new RawGitRepository(f.root).includes(f.source, f.unrelated), false);
    rejectConsumer(f);
    f.git(["replace", "-d", f.unrelated]);
    f.git(["replace", f.unrelated, f.source]);
    assert.equal(
      f.git(["show", `${f.unrelated}:file.txt`]).toString(),
      "independently frozen source\n",
    );
    assert.equal(
      new RawGitRepository(f.root).file(f.unrelated, "file.txt").toString(),
      "unrelated source\n",
    );
  }));
test("info/grafts cannot add public-cut ancestry even when no-replace revision walking accepts it", () =>
  fixture((f) => {
    fs.writeFileSync(path.join(f.root, ".git/info/grafts"), `${f.unrelated} ${f.source}\n`);
    assert.equal(
      f.command(["--no-replace-objects", "merge-base", "--is-ancestor", f.source, f.unrelated])
        .status,
      0,
    );
    assert.equal(new RawGitRepository(f.root).includes(f.source, f.unrelated), false);
    rejectConsumer(f);
  }));
test("ambient graft and replacement namespaces cannot add raw ancestry", () =>
  fixture((f) => {
    const graft = path.join(f.root, "external-grafts");
    fs.writeFileSync(graft, `${f.unrelated} ${f.source}\n`);
    f.git(["update-ref", `refs/inert-replacement/${f.unrelated}`, f.forged]);
    ambient({ GIT_GRAFT_FILE: graft, GIT_REPLACE_REF_BASE: "refs/inert-replacement" }, () => {
      assert.equal(new RawGitRepository(f.root).includes(f.source, f.unrelated), false);
      rejectConsumer(f);
    });
  }));
test("ambient repository and object-store variables cannot redirect pinned raw source reads", () =>
  fixture((f) => {
    ambient(
      {
        GIT_DIR: path.join(f.root, "absent-git"),
        GIT_OBJECT_DIRECTORY: path.join(f.root, "absent-objects"),
        GIT_COMMON_DIR: path.join(f.root, "absent-common"),
      },
      () => {
        const raw = new RawGitRepository(f.root);
        assert.equal(raw.includes(f.source, f.release), true);
        assert.equal(raw.file(f.source, "file.txt").toString(), "independently frozen source\n");
      },
    );
  }));
test("shallow revision metadata cannot remove the raw parent proof", () =>
  fixture((f) => {
    fs.writeFileSync(path.join(f.root, ".git/shallow"), `${f.cut}\n`);
    assert.equal(f.command(["merge-base", "--is-ancestor", f.source, f.cut]).status, 1);
    assert.equal(new RawGitRepository(f.root).includes(f.source, f.cut), true);
  }));
test("parent-looking messages and signature continuations never create an ancestry edge", () =>
  fixture((f) => {
    const message = f.commit(f.otherTree, [], `message\nparent ${f.source}`);
    const rawSource = f.git(["--no-replace-objects", "cat-file", "commit", f.unrelated]).toString();
    const signature = f.object(
      "commit",
      rawSource.replace("\n\n", `\ngpgsig inert\n parent ${f.source}\n\n`),
    );
    const raw = new RawGitRepository(f.root);
    assert.equal(raw.includes(f.source, message), false);
    assert.equal(raw.includes(f.source, signature), false);
  }));
test("out-of-order parent headers and missing/non-commit identities fail closed", () =>
  fixture((f) => {
    const bytes = f.git(["--no-replace-objects", "cat-file", "commit", f.unrelated]).toString();
    const malformed = f.object("commit", bytes.replace("\n\n", `\nparent ${f.source}\n\n`));
    const raw = new RawGitRepository(f.root);
    assert.throws(() => raw.includes(f.source, malformed), /out-of-order ancestry header/u);
    assert.throws(() => raw.includes(f.source, "0".repeat(40)), /unavailable or malformed/u);
    assert.throws(() => raw.includes(f.blob, f.source), /object type mismatch/u);
    assert.throws(() => raw.includes(f.source, f.blob), /object type mismatch/u);
    assert.throws(() => raw.includes("HEAD", f.source), /full SHA-1/u);
  }));
test("object bytes under a claimed SHA must reproduce that SHA", () =>
  fixture((f) => {
    const bytes = f.git(["--no-replace-objects", "cat-file", "commit", f.unrelated]);
    const changed = Buffer.from(bytes.toString().replace("unrelated cut", "tampered cut"));
    const objectPath = path.join(
      f.root,
      ".git/objects",
      f.unrelated.slice(0, 2),
      f.unrelated.slice(2),
    );
    fs.chmodSync(objectPath, 0o600);
    fs.writeFileSync(
      objectPath,
      deflateSync(Buffer.concat([Buffer.from(`commit ${changed.length}\0`), changed])),
    );
    assert.throws(() => new RawGitRepository(f.root).commit(f.unrelated), /content hash changed/u);
  }));
test("source tree and blob bytes cannot be substituted under their claimed identities", () => {
  for (const type of ["tree", "blob"] as const)
    fixture((f) => {
      const sha = type === "tree" ? f.sourceTree : f.blob;
      const original = f.git(["--no-replace-objects", "cat-file", type, sha]);
      const changed = Buffer.from(original);
      changed[changed.length - 1] ^= 1;
      const objectPath = path.join(f.root, ".git/objects", sha.slice(0, 2), sha.slice(2));
      fs.chmodSync(objectPath, 0o600);
      fs.writeFileSync(
        objectPath,
        deflateSync(Buffer.concat([Buffer.from(`${type} ${changed.length}\0`), changed])),
      );
      assert.throws(
        () => new RawGitRepository(f.root).file(f.source, "file.txt"),
        /content hash changed/u,
      );
    });
});
test("raw source resolution binds nested tree and blob identity and returns independent bytes", () =>
  fixture((f) => {
    const nested = f.tree([{ name: "nested", mode: "040000", type: "tree", sha: f.sourceTree }]);
    const commit = f.commit(nested, [f.source], "nested source");
    const raw = new RawGitRepository(f.root);
    assert.equal(raw.file(commit, "nested/file.txt").toString(), "independently frozen source\n");
    const first = raw.file(commit, "nested/file.txt");
    first.fill(0);
    assert.equal(raw.file(commit, "nested/file.txt").toString(), "independently frozen source\n");
    assert.throws(() => raw.file(commit, "nested/missing.txt"), /source path is absent/u);
    for (const file of [
      "",
      "/file.txt",
      "../file.txt",
      "nested/./file.txt",
      "nested//file.txt",
      "nested\\file.txt",
      "file.txt\0",
    ])
      assert.throws(() => raw.file(commit, file), /literal relative components/u);
  }));
test("a selected source symlink or gitlink never becomes file-byte authority", () =>
  fixture((f) => {
    const linkTree = f.tree([{ name: "file.txt", mode: "120000", type: "blob", sha: f.blob }]);
    const gitlinkTree = f.tree([
      { name: "file.txt", mode: "160000", type: "commit", sha: f.source },
    ]);
    const raw = new RawGitRepository(f.root);
    for (const tree of [linkTree, gitlinkTree]) {
      const commit = f.commit(tree, [], "non-file source");
      assert.throws(() => raw.file(commit, "file.txt"), /owned regular file/u);
      assert.throws(() => raw.file(commit, "file.txt/child"), /owned regular file/u);
    }
  }));
test("truncated raw trees cannot manufacture source blob authority", () =>
  fixture((f) => {
    const malformed = f.object("tree", Buffer.from("100644 file.txt\0"));
    const commit = f.commit(malformed, [], "malformed tree");
    assert.throws(
      () => new RawGitRepository(f.root).file(commit, "file.txt"),
      /tree entry is incomplete/u,
    );
  }));
