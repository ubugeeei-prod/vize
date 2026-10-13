import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  authenticateEmptyVueStarJoinedSource,
  retainedEmptyVueStarBody,
} from "../../tools/support/compat/github/empty-vue-star-javascript-source.mjs";
import {
  emptyVueStarBodyPairs,
  emptyVueStarMetadataPairs,
  emptyVueStarFiles,
  qualifyEmptyVueStarHost,
} from "../performance/support/warm-type-backed-empty-vue-star-host.ts";
import { ensureHostedLiteralSource } from "../performance/support/warm-type-backed-git-reader-literal-source.ts";

const owner = fs.realpathSync(
  process.env.STAR_JS_SOURCE_ROOT ?? new URL("../../", import.meta.url),
);
const capsule = path.resolve(import.meta.dirname, "../../tools/support/compat/github");
const main = "3a680d0a0b59b720af0d23a70a85cf56c5c4f175";
const git = (root, args, input) =>
  execFileSync(
    "git",
    ["--no-replace-objects", "-c", "gc.auto=0", "-c", "maintenance.auto=false", ...args],
    {
      cwd: root,
      input,
      maxBuffer: 64 * 1024 * 1024,
      env: { ...process.env, GIT_ALLOW_PROTOCOL: "", GIT_NO_LAZY_FETCH: "1" },
    },
  );
const load = (file) => JSON.parse(fs.readFileSync(path.join(capsule, file)));
const manifest = load("empty-vue-star-javascript-source.json");
const auxiliary = manifest.parts.flatMap(({ file }) => load(file));
const shared = load("javascript-sfc-star-source.json");
const contexts = shared.parts.map(([file]) => load(file));
contexts.push(load("javascript-sfc-star-alias-source.json").pair);
const originalFiles = emptyVueStarBodyPairs.concat(emptyVueStarMetadataPairs).map(([file]) => file);
const executingFiles = [
  "tools/support/compat/github/empty-vue-star-javascript-source.mjs",
  "tools/support/compat/github/empty-vue-star-custody-runner.mjs",
  "tools/support/compat/github/empty-vue-star-javascript-source.json",
  "tools/support/compat/github/empty-vue-star-javascript-aux-0.json",
  "tools/support/compat/github/empty-vue-star-javascript-aux-1.json",
  "tests/tooling/empty-vue-star-javascript-source.test.mjs",
];
const files = [...originalFiles, ...auxiliary.map(({ path }) => path), ...executingFiles];
const corrected = new Map(
  contexts.map((row) => [row.path, Buffer.from(row.currentAfter.body, "base64")]),
);
const indexPath = process.env.JS_STAR_SOURCE_INDEX;
if (indexPath)
  assert.notEqual(
    process.env.GITHUB_ACTIONS,
    "true",
    "private overlay cannot qualify hosted current source",
  );
const authored = indexPath
  ? new Map(
      JSON.parse(fs.readFileSync(indexPath)).files.map((row) => [
        row.path,
        fs.readFileSync(row.after.filePath),
      ]),
    )
  : new Map(
      auxiliary.slice(9).map(({ path: file }) => [file, fs.readFileSync(path.join(owner, file))]),
    );
for (const file of executingFiles)
  authored.set(file, fs.readFileSync(new URL("../../" + file, import.meta.url)));
authored.set(
  "tests/tooling/warm-type-backed-empty-vue-star-custody.test.ts",
  fs.readFileSync(new URL("warm-type-backed-empty-vue-star-custody.test.ts", import.meta.url)),
);
ensureHostedLiteralSource(owner, main);
ensureHostedLiteralSource(owner, "a93e78147b0990c813e965a7868b32dda341eb77");
const bodies = new Map();
for (const file of files) {
  const entry = git(owner, ["ls-tree", "-z", main, "--", file]).toString();
  const match = /^100644 blob ([a-f0-9]{40})\t(.+)\0$/u.exec(entry);
  const before = match ? git(owner, ["cat-file", "blob", match[1]]) : null;
  const after = corrected.get(file) ?? authored.get(file) ?? before;
  assert(after, "complete selected current fixture: " + file);
  bodies.set(file, { before, after });
}
function fixture(run) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-star-js-current-source-"));
  try {
    git(root, ["init", "--quiet"]);
    git(root, ["config", "user.name", "Star source controls"]);
    git(root, ["config", "user.email", "source@example.invalid"]);
    const common = git(owner, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(root, ".git/objects/info/alternates"),
      path.resolve(owner, common, "objects") + "\n",
    );
    git(root, ["read-tree", main]);
    for (const [file, { after }] of bodies) {
      const target = path.join(root, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, after, { mode: 0o644, flag: "wx" });
    }
    assert(files.every((file) => !/[\r\n\t]/u.test(file)));
    const blobs = git(root, ["hash-object", "-w", "--stdin-paths"], files.join("\n") + "\n")
      .toString()
      .trim()
      .split("\n");
    assert.equal(blobs.length, files.length);
    for (const blob of blobs) assert.match(blob, /^[a-f0-9]{40}$/u);
    git(
      root,
      ["update-index", "--index-info"],
      files.map((file, i) => "100644 " + blobs[i] + "\t" + file + "\n").join(""),
    );
    const tree = git(root, ["write-tree"]).toString().trim();
    const head = git(root, ["commit-tree", tree, "-p", main], "private complete source controls\n")
      .toString()
      .trim();
    git(root, ["update-ref", "refs/heads/current", head]);
    git(root, ["symbolic-ref", "HEAD", "refs/heads/current"]);
    run(root, head);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}
function changed(root, head, file, body, mode = "100644") {
  git(root, ["read-tree", head]);
  const blob = git(root, ["hash-object", "-w", "--stdin"], body).toString().trim();
  git(root, ["update-index", "--add", "--cacheinfo", mode + "," + blob + "," + file]);
  const tree = git(root, ["write-tree"]).toString().trim();
  const commit = git(root, ["commit-tree", tree, "-p", head], "private refused source\n")
    .toString()
    .trim();
  git(root, ["update-ref", "refs/heads/current", commit]);
  fs.writeFileSync(path.join(root, file), body);
  fs.chmodSync(path.join(root, file), Number.parseInt(mode, 8) & 0o777);
  return commit;
}
void test("actual complete322 current bodies precede the unchanged290 Star source model", () => {
  if (!indexPath) {
    const actual = authenticateEmptyVueStarJoinedSource(owner);
    assert.equal(actual.head, git(owner, ["rev-parse", "HEAD"]).toString().trim());
    assert.equal(actual.completeCurrentBodies, 322);
    assert.equal(actual.runtimeCredit, false);
  }
  console.log(
    JSON.stringify({
      sourceMode: indexPath
        ? "explicit-local-overlay-private-only"
        : "actual-repository-owner-and-private-controls",
      actualRepositoryOwnerQualified: !indexPath,
      nativeCredit: false,
    }),
  );
  fixture((root) => {
    const packet = authenticateEmptyVueStarJoinedSource(root);
    assert.equal(packet.completeCurrentBodies, 322);
    assert.equal(packet.executingAdapterBodies, 6);
    assert.equal(packet.compiledBodies, 281);
    assert.equal(packet.starMetadataBodies, 9);
    assert.equal(packet.nativeAndOriginalCallerBodies, 9);
    assert.equal(packet.javascriptAuxiliaryBodies, 17);
    assert.equal(packet.retainedStarQualifier.completeBodyPairs, 44);
    assert.equal(packet.retainedStarQualifier.supplementalBodyPairs, 85);
    assert.equal(packet.retainedStarQualifier.preservedBodyPairs, 152);
    assert.equal(packet.retainedStarQualifier.historicalCasesPreserved, 32);
    assert.equal(packet.runtimeCredit, false);
    assert.equal(packet.original400Credit, false);
    assert.equal(packet.performanceCredit, false);
    for (const file of [originalFiles[0], originalFiles.at(-1)])
      assert.deepEqual(retainedEmptyVueStarBody(packet, file), bodies.get(file).before);
    assert.deepEqual(
      retainedEmptyVueStarBody(packet, contexts[5].path),
      bodies.get(contexts[5].path).before,
    );
  });
});
void test("each declared current source group refuses missing or mutated physical bodies before any loan", () => {
  fixture((root) => {
    for (const file of [
      originalFiles[0],
      emptyVueStarMetadataPairs[0][0],
      auxiliary[2].path,
      auxiliary[10].path,
    ]) {
      const target = path.join(root, file),
        body = fs.readFileSync(target);
      fs.unlinkSync(target);
      assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
      fs.writeFileSync(target, Buffer.concat([body, Buffer.from("foreign source\n")]), {
        mode: 0o644,
      });
      assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
      fs.writeFileSync(target, body);
    }
  });
});
void test("all seven old/fused hybrids refuse even when physical and raw bytes agree", () => {
  fixture((root, head) => {
    for (const row of contexts) {
      changed(root, head, row.path, bodies.get(row.path).before);
      assert.throws(() => authenticateEmptyVueStarJoinedSource(root), row.path);
      git(root, ["update-ref", "refs/heads/current", head]);
      fs.writeFileSync(path.join(root, row.path), bodies.get(row.path).after);
    }
    assert(authenticateEmptyVueStarJoinedSource(root));
  });
});
void test("changed raw source, physical and raw modes, symlinks and replacement refs refuse", () => {
  fixture((root, head) => {
    const file = contexts[1].path,
      target = path.join(root, file),
      body = bodies.get(file).after;
    changed(root, head, file, Buffer.concat([body, Buffer.from("foreign whole body\n")]));
    assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
    git(root, ["update-ref", "refs/heads/current", head]);
    fs.writeFileSync(target, body);
    fs.chmodSync(target, 0o755);
    assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
    fs.chmodSync(target, 0o644);
    const foreign = changed(root, head, file, body, "100755");
    assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
    git(root, ["update-ref", "refs/heads/current", head]);
    fs.chmodSync(target, 0o644);
    fs.unlinkSync(target);
    fs.symlinkSync(path.join(owner, file), target);
    assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
    fs.unlinkSync(target);
    fs.writeFileSync(target, body, { mode: 0o644 });
    git(root, ["update-ref", "refs/replace/" + head, foreign]);
    assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
    git(root, ["update-ref", "-d", "refs/replace/" + head]);
    assert(authenticateEmptyVueStarJoinedSource(root));
  });
});
void test("forged, mutable, stale and unknown source loans refuse; returned buffers are isolated", () => {
  fixture((root, head) => {
    const packet = authenticateEmptyVueStarJoinedSource(root),
      file = contexts[1].path;
    assert.equal(Reflect.set(packet, "head", "0".repeat(40)), false);
    assert.equal(Reflect.set(packet.retainedStarQualifier, "runtimeCredit", true), false);
    assert.throws(() => retainedEmptyVueStarBody({ ...packet }, file));
    assert.throws(() => retainedEmptyVueStarBody(packet, "crates/vize_canon/src/unreviewed.rs"));
    const body = retainedEmptyVueStarBody(packet, file);
    body.fill(0);
    assert.deepEqual(retainedEmptyVueStarBody(packet, file), bodies.get(file).before);
    changed(root, head, file, bodies.get(file).after);
    assert.throws(() => retainedEmptyVueStarBody(packet, file));
  });
});
void test("the complete original Star qualifier still refuses unknown, partial and inherited-only footprints", () => {
  const forbidden = () => assert.fail("an ineligible footprint cannot request a source loan");
  assert.equal(
    qualifyEmptyVueStarHost([...emptyVueStarFiles, "crates/unreviewed.rs"], new Set(), forbidden),
    null,
  );
  assert.equal(qualifyEmptyVueStarHost(emptyVueStarFiles.slice(1), new Set(), forbidden), null);
  assert.equal(
    qualifyEmptyVueStarHost([...emptyVueStarFiles], new Set(emptyVueStarFiles), forbidden),
    null,
  );
});
void test("ancestor symlinks and same-HEAD mutation or replacement refs refuse before an old source loan", () => {
  fixture((root, head) => {
    const file = contexts[1].path,
      target = path.join(root, file),
      body = bodies.get(file).after;
    const packet = authenticateEmptyVueStarJoinedSource(root);
    fs.writeFileSync(target, Buffer.concat([body, Buffer.from("foreign physical source\n")]));
    assert.throws(() => retainedEmptyVueStarBody(packet, file));
    fs.writeFileSync(target, body);
    git(root, ["update-ref", "refs/replace/" + head, main]);
    assert.throws(() => retainedEmptyVueStarBody(packet, file));
    git(root, ["update-ref", "-d", "refs/replace/" + head]);
    const directory = path.dirname(target),
      held = directory + "-identical";
    fs.renameSync(directory, held);
    fs.symlinkSync(held, directory);
    assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
    assert.throws(() => retainedEmptyVueStarBody(packet, file));
    fs.unlinkSync(directory);
    fs.renameSync(held, directory);
    assert.deepEqual(retainedEmptyVueStarBody(packet, file), bodies.get(file).before);
  });
});
void test("executing adapter and all metadata snapshots must be committed and physically exact", () => {
  fixture((root, head) => {
    for (const file of executingFiles) {
      const target = path.join(root, file),
        body = bodies.get(file).after;
      fs.unlinkSync(target);
      assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
      fs.writeFileSync(target, body, { mode: 0o644 });
      changed(root, head, file, Buffer.concat([body, Buffer.from("foreign snapshot\n")]));
      assert.throws(() => authenticateEmptyVueStarJoinedSource(root));
      git(root, ["update-ref", "refs/heads/current", head]);
      fs.writeFileSync(target, body);
    }
    assert(authenticateEmptyVueStarJoinedSource(root));
  });
});
void test("a real current-only depth1 object store refuses missing pinned history without fetch or mutation", () => {
  fixture((prepared) => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-star-js-cold-source-"));
    try {
      git(root, ["init", "--quiet"]);
      git(root, ["config", "user.name", "Cold source control"]);
      git(root, ["config", "user.email", "source@example.invalid"]);
      for (const file of files) {
        const target = path.join(root, file);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.copyFileSync(path.join(prepared, file), target);
      }
      git(root, ["add", "--", ...files]);
      git(root, ["commit", "--quiet", "-m", "complete current-only source"]);
      const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
      const shallow = path.join(root, ".git/shallow");
      fs.writeFileSync(shallow, head + "\n");
      for (const source of [main, "a93e78147b0990c813e965a7868b32dda341eb77"])
        assert.equal(
          git(
            root,
            ["cat-file", "--batch-check=%(objectname) %(objecttype)"],
            source + "\n",
          ).toString(),
          source + " missing\n",
        );
      const actions = process.env.GITHUB_ACTIONS;
      delete process.env.GITHUB_ACTIONS;
      try {
        assert.throws(
          () => authenticateEmptyVueStarJoinedSource(root),
          /missing literal requires Actions/u,
        );
      } finally {
        if (actions === undefined) delete process.env.GITHUB_ACTIONS;
        else process.env.GITHUB_ACTIONS = actions;
      }
      assert.equal(git(root, ["rev-parse", "HEAD"]).toString().trim(), head);
      assert.equal(fs.readFileSync(shallow, "utf8"), head + "\n");
      assert.equal(fs.existsSync(path.join(root, ".git/FETCH_HEAD")), false);
      assert.equal(fs.existsSync(path.join(root, ".git/objects/info/alternates")), false);
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
});
