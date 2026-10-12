import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { authenticateNoMutatingComponentJoin } from "./support/no-mutating-component-join.mjs";
import { joinGit, joinObjectID, joinSHA } from "./support/no-mutating-component-join-physical.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const sharedSnapshot = () => {
  const name = joinGit(root, ["rev-parse", "--git-path", "shallow"]).toString().trim();
  const shallow = path.resolve(root, name);
  return {
    head: joinGit(root, ["rev-parse", "HEAD"]).toString(),
    config: joinGit(root, ["config", "--local", "--list"]).toString(),
    shallow: fs.existsSync(shallow) ? fs.readFileSync(shallow) : null,
  };
};
const metadataSnapshot = (owned) =>
  ["HEAD", "shallow"].map((name) => {
    const filename = path.join(owned, ".git", name);
    return fs.existsSync(filename) ? fs.readFileSync(filename) : null;
  });

void test("actual owned frame configuration suppresses Git maintenance during exact local source fetches", () => {
  const sharedBefore = sharedSnapshot();
  const joined = authenticateNoMutatingComponentJoin(root);
  assert.equal(joined.completeCurrentClosedBodies, 323);
  assert.equal(joined.completeAncillarySourceControlBodies, 1);
  const owned = joined.frames[0].root;
  assert.equal(
    joinGit(owned, ["config", "--bool", "--get", "maintenance.auto"]).toString(),
    "false\n",
  );
  assert.equal(joinGit(owned, ["config", "--get", "gc.auto"]).toString(), "0\n");
  const privateBefore = metadataSnapshot(owned);
  assert(Buffer.isBuffer(privateBefore[0]));
  const originalRemote = joinGit(owned, ["remote", "get-url", "origin"]);
  const area = fs.mkdtempSync(path.join(os.tmpdir(), "vize-owned-maintenance-source-control-"));
  const origin = path.join(area, "origin");
  const trace = path.join(area, "git-trace.jsonl");
  const runs = [];
  const sources = [];
  let addedRemote = false;
  let controlAssertionsPassed = false;
  try {
    fs.mkdirSync(origin);
    joinGit(origin, ["init", "--quiet"]);
    for (let index = 0; index < 15; index++) {
      const body = Buffer.from("whole private source " + index + "\n");
      const blob = joinGit(origin, ["hash-object", "-w", "--stdin"], body).toString().trim();
      assert.equal(blob, joinObjectID("blob", body));
      const tree = joinGit(origin, ["mktree"], "100644 blob " + blob + "\tsource.txt\n")
        .toString()
        .trim();
      const argv = [
        "-c",
        "user.name=Source Control",
        "-c",
        "user.email=source@example.invalid",
        "commit-tree",
        tree,
      ];
      if (sources.length) argv.push("-p", sources.at(-1).source);
      const source = joinGit(origin, argv, "owned source " + index + "\n")
        .toString()
        .trim();
      const commit = joinGit(origin, ["cat-file", "commit", source]);
      assert.equal(source, joinObjectID("commit", commit));
      sources.push({ source, blob, tree, body, commit });
    }
    fs.writeFileSync(path.join(origin, ".git/HEAD"), sources.at(-1).source + "\n");
    joinGit(owned, ["remote", "add", "maintenance-control", "file://" + origin]);
    addedRemote = true;
    for (const source of sources.toReversed()) {
      const argv = [
        "--no-replace-objects",
        "fetch",
        "--no-tags",
        "--depth=1",
        "maintenance-control",
        source.source,
      ];
      const result = spawnSync("git", argv, {
        cwd: owned,
        encoding: "utf8",
        timeout: 30_000,
        maxBuffer: 4 * 1024 * 1024,
        env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_TRACE2_EVENT: trace },
      });
      runs.push({
        source: source.source,
        argv,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout,
        stderr: result.stderr,
      });
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(joinGit(owned, ["cat-file", "commit", source.source]), source.commit);
      assert.deepEqual(joinGit(owned, ["cat-file", "blob", source.blob]), source.body);
      assert.equal(joinGit(owned, ["rev-parse", "FETCH_HEAD"]).toString().trim(), source.source);
    }
    const bytes = fs.readFileSync(trace);
    const events = bytes
      .toString()
      .trimEnd()
      .split("\n")
      .map((line) => JSON.parse(line));
    const actors = events.filter(
      (event) =>
        event.event === "child_start" &&
        event.argv?.some((word) => ["maintenance", "repack", "gc"].includes(word)),
    );
    assert.equal(actors.length, 0, JSON.stringify(actors));
    assert.equal(
      events.filter((event) => event.event === "start" && event.argv?.includes("fetch")).length,
      15,
    );
    assert.equal(
      joinGit(owned, ["rev-parse", "HEAD"]).toString().trim(),
      joined.frames[0].frame.source,
    );
    controlAssertionsPassed = true;
  } finally {
    const bytes = fs.existsSync(trace) ? fs.readFileSync(trace) : null;
    let events = [];
    let traceParseError = null;
    if (bytes) {
      try {
        events = bytes
          .toString()
          .trimEnd()
          .split("\n")
          .map((line) => JSON.parse(line));
      } catch (error) {
        traceParseError = String(error);
      }
    }
    const directory = path.join(root, "target/differential/no-mutating-component-join");
    fs.mkdirSync(directory, { recursive: true });
    const filename = path.join(directory, "owned-maintenance-control.json");
    const receipt = {
      sourceOnly: true,
      native: false,
      compiled: false,
      performance: false,
      exactFatalReproduced: false,
      controlAssertionsPassed,
      restoration: "pending",
      runs,
      gitVersion: joinGit(owned, ["--version"]).toString().trim(),
      traceStatus: bytes === null ? "missing" : traceParseError ? "parse-error" : "complete",
      traceParseError,
      traceBase64: bytes?.toString("base64") ?? null,
      traceBytes: bytes?.length ?? 0,
      traceSHA256: bytes ? joinSHA(bytes) : null,
      automaticActors: events.filter(
        (event) =>
          event.event === "child_start" &&
          event.argv?.some((word) => ["maintenance", "repack", "gc"].includes(word)),
      ),
      sources: sources.map((source) => ({
        source: source.source,
        tree: source.tree,
        commitBase64: source.commit.toString("base64"),
        commitBytes: source.commit.length,
        commitSHA256: joinSHA(source.commit),
        blob: source.blob,
        sourceBase64: source.body.toString("base64"),
        sourceBytes: source.body.length,
        sourceSHA256: joinSHA(source.body),
      })),
      ownedBefore: privateBefore.map((body) => body?.toString("base64") ?? null),
      ownedBeforeRestore: metadataSnapshot(owned).map((body) => body?.toString("base64") ?? null),
      sharedBefore: {
        head: sharedBefore.head,
        configBytes: Buffer.byteLength(sharedBefore.config),
        configSHA256: joinSHA(Buffer.from(sharedBefore.config)),
        shallowBase64: sharedBefore.shallow?.toString("base64") ?? null,
      },
    };
    const persist = () => fs.writeFileSync(filename, JSON.stringify(receipt, null, 2) + "\n");
    persist();
    if (addedRemote) joinGit(owned, ["remote", "remove", "maintenance-control"]);
    // Fetch extends only this new store's shallow boundary; restore its exact original bytes.
    const shallow = path.join(owned, ".git/shallow");
    if (privateBefore[1] === null) fs.rmSync(shallow, { force: true });
    else fs.writeFileSync(shallow, privateBefore[1]);
    assert.deepEqual(metadataSnapshot(owned), privateBefore);
    assert.deepEqual(joinGit(owned, ["remote", "get-url", "origin"]), originalRemote);
    assert.deepEqual(sharedSnapshot(), sharedBefore);
    fs.rmSync(area, { recursive: true, force: true });
    receipt.restoration = "verified";
    persist();
  }
});
