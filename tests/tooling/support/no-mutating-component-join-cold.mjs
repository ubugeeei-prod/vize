import assert from "node:assert/strict";
import fs from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { prepareLiteralSource } from "../../performance/support/warm-type-backed-git-reader-nodeep.ts";
import {
  joinGit,
  joinObjectID,
  joinPath,
  joinRawBodies,
  joinSHA,
} from "./no-mutating-component-join-physical.mjs";

function objectPacket(owned, mode, input) {
  assert(mode === "--batch" || mode === "--batch-check");
  const result = spawnSync("git", ["--no-replace-objects", "cat-file", mode], {
    cwd: owned,
    input,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0);
  assert.equal(result.stderr.length, 0, "corrupt or unreadable objects never hydrate");
  return result.stdout;
}

function completeTreeAvailable(owned, rootTree, selectedPaths, expectedFiles, absentPaths) {
  const required = new Map(expectedFiles.map((row) => [row.path, row]));
  assert.equal(required.size, expectedFiles.length);
  const absent = new Set(absentPaths);
  const observed = new Set();
  let pending = [{ id: rootTree, prefix: "" }];
  const visited = new Set();
  while (pending.length) {
    const locations = pending.filter((row) => !visited.has(row.id + "\0" + row.prefix));
    const ids = [...new Set(locations.map((row) => row.id))];
    if (!ids.length) break;
    const packet = objectPacket(owned, "--batch", ids.join("\n") + "\n");
    const children = new Map();
    let cursor = 0;
    let missing = false;
    pending = [];
    for (const id of ids) {
      assert.match(id, /^[a-f0-9]{40}$/u);
      const end = packet.indexOf(10, cursor);
      assert(end >= cursor);
      const header = packet.subarray(cursor, end).toString();
      if (header === id + " missing") {
        missing = true;
        cursor = end + 1;
        continue;
      }
      const match = header.match(new RegExp("^" + id + " tree ([0-9]+)$", "u"));
      assert(match, "only the actual expected tree object");
      const bytes = Number(match[1]);
      assert(Number.isSafeInteger(bytes) && bytes >= 0);
      const body = packet.subarray(end + 1, end + 1 + bytes);
      assert.equal(body.length, bytes);
      assert.equal(joinObjectID("tree", body), id, "whole actual tree content");
      assert.equal(packet[end + 1 + bytes], 10);
      cursor = end + 2 + bytes;
      const directories = [];
      let offset = 0;
      while (offset < body.length) {
        const stop = body.indexOf(0, offset);
        assert(stop >= offset && stop + 21 <= body.length);
        const entry = body.subarray(offset, stop).toString();
        assert.deepEqual(Buffer.from(entry), body.subarray(offset, stop));
        assert.match(entry, /^(40000|100644|100755|120000|160000) [^/\0]+$/u);
        const split = entry.indexOf(" ");
        directories.push({
          mode: entry.slice(0, split),
          name: entry.slice(split + 1),
          id: body.subarray(stop + 1, stop + 21).toString("hex"),
        });
        offset = stop + 21;
      }
      assert.equal(offset, body.length);
      children.set(id, directories);
    }
    assert.equal(cursor, packet.length);
    if (missing) return false;
    for (const location of locations) {
      visited.add(location.id + "\0" + location.prefix);
      for (const child of children.get(location.id)) {
        const name = location.prefix + child.name;
        assert(!absent.has(name), "declared absence must be genuinely absent: " + name);
        if (required.has(name)) {
          const row = required.get(name);
          assert.equal(child.mode, row.mode);
          assert.equal(child.mode, "100644");
          assert.equal(
            child.id,
            row.gitBlob,
            "declared blob must belong to the actual source path",
          );
          observed.add(name);
        }
        if (child.mode !== "40000") continue;
        const prefix = name + "/";
        if (
          selectedPaths === undefined ||
          selectedPaths.some((name) => name.startsWith(prefix) || name === prefix.slice(0, -1))
        )
          pending.push({ id: child.id, prefix });
      }
    }
  }
  assert.equal(observed.size, required.size, "complete actual required source paths");
  return true;
}

/** Hydrate only this newly owned private store; existing commit headers never mask missing blobs. */
export function provisionJoinColdSources(owned, requirements) {
  assert.equal(
    fs.realpathSync(path.join(owned, ".git")),
    path.join(fs.realpathSync(owned), ".git"),
  );
  assert.equal(joinGit(owned, ["rev-parse", "--git-common-dir"]).toString().trim(), ".git");
  for (const declared of requirements) {
    assert.match(declared.source, /^[a-f0-9]{40}$/u);
    const sourceHeader = objectPacket(owned, "--batch-check", declared.source + "\n").toString();
    assert(
      sourceHeader === declared.source + " missing\n" ||
        new RegExp("^" + declared.source + " commit [0-9]+\n$", "u").test(sourceHeader),
      "only the exact source commit",
    );
    prepareLiteralSource(owned, declared.source);
    const commit = joinGit(owned, ["cat-file", "commit", declared.source]);
    assert.equal(joinObjectID("commit", commit), declared.source);
    assert.equal(commit.length, declared.bytes);
    assert.equal(joinSHA(commit), declared.sha256);
    const available = () => {
      for (const row of declared.files) {
        joinPath(row.path);
        assert.match(row.gitBlob, /^[a-f0-9]{40}$/u);
      }
      for (const name of declared.absentPaths) joinPath(name);
      const rootTree = commit.subarray(0, commit.indexOf(10)).toString();
      assert.match(rootTree, /^tree [a-f0-9]{40}$/u);
      const selected = [...declared.files.map((row) => row.path), ...declared.absentPaths];
      if (
        (declared.completeTree || selected.length) &&
        !completeTreeAvailable(
          owned,
          rootTree.slice(5),
          declared.completeTree ? undefined : selected,
          declared.files,
          declared.absentPaths,
        )
      )
        return false;

      const objects = [...new Map(declared.files.map((row) => [row.gitBlob, row])).values()];
      if (objects.length) {
        const packet = objectPacket(
          owned,
          "--batch-check",
          objects.map((row) => row.gitBlob).join("\n") + "\n",
        ).toString();
        const lines = packet.split("\n");
        assert.equal(lines.pop(), "");
        assert.equal(lines.length, objects.length);
        let missing = false;
        for (const [index, row] of objects.entries()) {
          if (lines[index] === row.gitBlob + " missing") missing = true;
          else assert.equal(lines[index], row.gitBlob + " blob " + row.bytes);
        }
        if (missing) return false;
      }
      if (declared.files.length) joinRawBodies(owned, declared.source, declared.files);
      if (declared.absentPaths.length)
        assert.equal(
          joinGit(owned, ["ls-tree", "-z", declared.source, "--", ...declared.absentPaths]).length,
          0,
        );
      return true;
    };
    if (!available()) {
      joinGit(owned, ["fetch", "--refetch", "--no-tags", "--depth=1", "origin", declared.source]);
      assert(available(), "exact private source objects remain incomplete");
    }
  }
}
