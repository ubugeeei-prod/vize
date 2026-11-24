import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  checkCustody,
  digestsOf,
  parseRegistry,
  registryPath,
  renderRegistry,
} from "../../tools/commands/ci/byte-custody.ts";

function repository(files: Record<string, string>): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-custody-"));
  for (const [file, text] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), text);
  }
  execFileSync("git", ["init", "--quiet"], { cwd: root });
  execFileSync("git", ["add", "."], { cwd: root });
  return root;
}

const sha = (text: string) => digestsOf(Buffer.from(text)).sha256;

test("a pinned source must be registered, and registering it settles the check", (t) => {
  const source = "pub fn pinned() {}\n";
  const root = repository({
    "crates/a/src/pinned.rs": source,
    "crates/a/src/free.rs": "pub fn free() {}\n",
    "tests/tooling/pinned.test.ts": `const pin = "${sha(source)}";\n`,
  });
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));

  const first = checkCustody(root, new Map());
  assert.equal(first.problems.length, 1);
  assert.equal(first.problems[0].startsWith("crates/a/src/pinned.rs: its current bytes"), true);
  assert.deepEqual(first.next, [
    {
      path: "crates/a/src/pinned.rs",
      digests: [`sha256:${sha(source)}`],
      class: "enforced",
      owners: ["tests/tooling/pinned.test.ts"],
    },
  ]);

  fs.mkdirSync(path.join(root, "tools/config"), { recursive: true });
  fs.writeFileSync(path.join(root, registryPath), renderRegistry(first.next));
  execFileSync("git", ["add", "."], { cwd: root });
  const registry = parseRegistry(fs.readFileSync(path.join(root, registryPath), "utf8"));
  assert.deepEqual(checkCustody(root, registry).problems, []);

  // Moving the pinned file unchanged keeps its row's class.
  fs.renameSync(path.join(root, "crates/a/src/pinned.rs"), path.join(root, "crates/a/src/kept.rs"));
  execFileSync("git", ["add", "-A"], { cwd: root });
  const moved = checkCustody(root, registry);
  assert.deepEqual(
    [moved.problems, moved.stale],
    [
      [],
      [
        "crates/a/src/pinned.rs: gone; its row will be dropped",
        "crates/a/src/kept.rs: moved from crates/a/src/pinned.rs",
      ],
    ],
  );
  fs.renameSync(path.join(root, "crates/a/src/kept.rs"), path.join(root, "crates/a/src/pinned.rs"));
  execFileSync("git", ["add", "-A"], { cwd: root });

  // Editing the enforced file without re-pinning it is a custody violation.
  fs.writeFileSync(path.join(root, "crates/a/src/pinned.rs"), "pub fn pinned() { }\n");
  assert.equal(checkCustody(root, registry).problems.length, 1);

  // A record-only row only goes stale.
  const recordOnly = new Map(
    [...registry].map(([file, row]) => [file, { ...row, class: "record-only" as const }]),
  );
  const report = checkCustody(root, recordOnly);
  assert.deepEqual(report.problems, []);
  assert.deepEqual(report.stale, ["crates/a/src/pinned.rs: re-pinned or edited"]);
});

test("the registry rejects malformed rows", () => {
  assert.throws(
    () => parseRegistry("path\tdigests\tclass\towners\na.rs\tsha256:abc\tenforced\t\n"),
    /malformed digest list/,
  );
  assert.throws(
    () => parseRegistry(`a.rs\tsha256:${"0".repeat(64)}\tmaybe\t\n`),
    /class must be enforced or record-only/,
  );
});
