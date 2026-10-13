import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

export function ownedCorsaReader(root, baseline, head) {
  root = fs.realpathSync(root);
  const git = (args) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      cwd: root,
      maxBuffer: 64 * 1024 * 1024,
    });
  for (const revision of [baseline, head]) assert.match(revision, /^[a-f0-9]{40}$/u);
  assert.equal(git(["rev-parse", "HEAD"]).toString().trim(), head);
  assert.equal(
    git(["for-each-ref", "--format=%(refname)", "refs/replace"]).length,
    0,
    "replacement refs refuse source publication",
  );
  const values = new Map();
  return (side, file) => {
    assert(["before", "after"].includes(side));
    assert(
      file &&
        !path.isAbsolute(file) &&
        file.split("/").every((part) => part && part !== "." && part !== ".."),
    );
    const key = side + ":" + file;
    if (!values.has(key)) {
      const entry = git(["ls-tree", "-z", side === "before" ? baseline : head, "--", file]);
      let value = null;
      if (entry.length) {
        const match = /^100644 blob ([a-f0-9]{40})\t(.+)\0$/u.exec(entry.toString());
        assert(match && match[2] === file, "one complete regular raw source: " + file);
        value = { mode: "100644", blob: match[1], body: git(["cat-file", "blob", match[1]]) };
        if (side === "after") {
          const target = path.join(root, file);
          let component = root;
          for (const part of file.split("/")) {
            component = path.join(component, part);
            assert(
              !fs.lstatSync(component).isSymbolicLink(),
              "physical source ancestor: " + component,
            );
          }
          const stat = fs.lstatSync(target);
          assert(
            stat.isFile() && (stat.mode & 0o777) === 0o644,
            "physical regular source: " + file,
          );
          assert.deepEqual(
            fs.readFileSync(target),
            value.body,
            "physical body equals selected raw Git: " + file,
          );
        }
      }
      values.set(key, value);
    }
    const value = values.get(key);
    return value ? { ...value, body: Buffer.from(value.body) } : null;
  };
}
