import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type ObjectType = "commit" | "tree" | "blob";
type Commit = Readonly<{ tree: string; parents: readonly string[] }>;
const objectId = /^[a-f0-9]{40}$/u;

/** Read immutable SHA-1 objects; revision walks and replacement source views are not authority. */
export class RawGitRepository {
  readonly #root: string;
  readonly #objects = new Map<string, { type: ObjectType; bytes: Buffer }>();

  constructor(repositoryRoot: string) {
    assert.ok(path.isAbsolute(repositoryRoot), "raw Git authority requires an absolute repository");
    assert.equal(
      fs.realpathSync(repositoryRoot),
      repositoryRoot,
      "raw Git repository cannot redirect",
    );
    this.#root = repositoryRoot;
  }

  #object(sha: string, type: ObjectType): Buffer {
    assert.match(sha, objectId, "raw Git authority requires a full SHA-1 object identity");
    const cached = this.#objects.get(sha);
    if (cached) {
      assert.equal(cached.type, type, `raw Git object type mismatch: ${sha}`);
      return cached.bytes;
    }
    // Ambient repository, object-store, graft and replacement variables cannot redirect reads.
    const environment = Object.fromEntries(
      Object.entries(process.env).filter(([key]) => !key.startsWith("GIT_")),
    );
    const read = spawnSync("git", ["--no-replace-objects", "cat-file", "--batch"], {
      cwd: this.#root,
      env: { ...environment, GIT_NO_REPLACE_OBJECTS: "1", GIT_NO_LAZY_FETCH: "1" },
      input: `${sha}\n`,
      maxBuffer: 16 * 1024 * 1024,
    });
    assert.equal(read.error, undefined, `raw Git object read failed: ${sha}`);
    assert.equal(read.status, 0, `raw Git object read failed: ${sha}`);
    assert.equal(read.signal, null, `raw Git object read interrupted: ${sha}`);
    const separator = read.stdout.indexOf(10);
    assert.ok(separator > 0, `raw Git object header missing: ${sha}`);
    const header = read.stdout.subarray(0, separator).toString("ascii");
    const match = /^([a-f0-9]{40}) (commit|tree|blob|tag) (0|[1-9][0-9]*)$/u.exec(header);
    assert.ok(match, `raw Git object unavailable or malformed: ${sha}`);
    assert.equal(match[1], sha, "raw Git object identity changed");
    assert.equal(match[2], type, `raw Git object type mismatch: ${sha}`);
    const size = Number(match[3]);
    assert.ok(Number.isSafeInteger(size), "raw Git object size is invalid");
    assert.equal(read.stdout.length, separator + 1 + size + 1, "raw Git object length changed");
    assert.equal(read.stdout.at(-1), 10, "raw Git object terminator missing");
    const bytes = read.stdout.subarray(separator + 1, -1);
    const actual = createHash("sha1")
      .update(`${type} ${bytes.length}\0`)
      .update(bytes)
      .digest("hex");
    assert.equal(actual, sha, `raw Git object content hash changed: ${sha}`);
    this.#objects.set(sha, { type, bytes });
    return bytes;
  }

  commit(sha: string): Commit {
    const bytes = this.#object(sha, "commit");
    const end = bytes.indexOf(Buffer.from("\n\n"));
    assert.ok(end >= 0, "raw Git commit header is incomplete");
    const headers = bytes.subarray(0, end).toString("utf8").split("\n");
    assert.match(headers[0], /^tree [a-f0-9]{40}$/u, "raw Git commit tree is invalid");
    const parents: string[] = [];
    let index = 1;
    while (headers[index]?.startsWith("parent ")) {
      assert.match(headers[index], /^parent [a-f0-9]{40}$/u, "raw Git parent is invalid");
      parents.push(headers[index++].slice(7));
    }
    assert.ok(headers[index]?.startsWith("author "), "raw Git commit author is missing");
    assert.ok(headers[index + 1]?.startsWith("committer "), "raw Git commit committer is missing");
    assert.ok(
      headers.slice(index).every((header) => !/^(?:tree|parent) /u.test(header)),
      "raw Git commit has an out-of-order ancestry header",
    );
    return Object.freeze({ tree: headers[0].slice(5), parents: Object.freeze(parents) });
  }

  /** A genuine raw parent path is required, including when ancestor and destination are equal. */
  includes(ancestor: string, destination: string): boolean {
    this.commit(ancestor);
    const pending = [destination];
    const seen = new Set<string>();
    while (pending.length) {
      const current = pending.pop()!;
      if (seen.has(current)) continue;
      seen.add(current);
      const commit = this.commit(current);
      if (current === ancestor) return true;
      pending.push(...commit.parents.toReversed());
    }
    return false;
  }

  /** Resolve an explicit repository path through authenticated raw trees, never a symlink/gitlink. */
  file(commitSha: string, sourcePath: string): Buffer {
    const parts = sourcePath.split("/");
    assert.ok(
      parts.every((part) => part !== "" && part !== "." && part !== ".." && !/[\\\0]/u.test(part)),
      "raw Git source path must contain literal relative components",
    );
    let object = this.commit(commitSha).tree;
    for (const [index, part] of parts.entries()) {
      const bytes = this.#object(object, "tree");
      const wanted = Buffer.from(part, "utf8");
      let selected: { mode: string; sha: string } | undefined;
      let offset = 0;
      while (offset < bytes.length) {
        const space = bytes.indexOf(32, offset);
        const nul = bytes.indexOf(0, space + 1);
        assert.ok(
          space > offset && nul > space + 1 && nul + 21 <= bytes.length,
          "raw Git tree entry is incomplete",
        );
        const mode = bytes.subarray(offset, space).toString("ascii");
        assert.match(
          mode,
          /^(?:40000|100644|100755|120000|160000)$/u,
          "raw Git tree mode is invalid",
        );
        const name = bytes.subarray(space + 1, nul);
        assert.ok(!name.includes(47), "raw Git tree name contains a path separator");
        if (name.equals(wanted)) {
          assert.equal(selected, undefined, "raw Git source path is ambiguous");
          selected = { mode, sha: bytes.subarray(nul + 1, nul + 21).toString("hex") };
        }
        offset = nul + 21;
      }
      assert.ok(selected, `raw Git source path is absent: ${sourcePath}`);
      const last = index === parts.length - 1;
      assert.ok(
        last ? selected.mode === "100644" || selected.mode === "100755" : selected.mode === "40000",
        "raw Git source path is not an owned regular file",
      );
      object = selected.sha;
      if (last) return Buffer.from(this.#object(object, "blob"));
    }
    throw new Error("raw Git source path is empty");
  }
}
