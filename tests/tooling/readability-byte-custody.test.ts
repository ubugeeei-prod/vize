import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  checkCustody,
  digestsOf,
  main as custodyMain,
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

  for (const setting of ["grep.lineNumber", "grep.column", "color.grep"]) {
    execFileSync("git", ["config", setting, setting === "color.grep" ? "always" : "true"], {
      cwd: root,
    });
  }

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

  const recordOnly = new Map(
    [...registry].map(([file, row]) => [file, { ...row, class: "record-only" as const }]),
  );

  // Moving the pinned file unchanged keeps its row's class.
  fs.renameSync(path.join(root, "crates/a/src/pinned.rs"), path.join(root, "crates/a/src/kept.rs"));
  execFileSync("git", ["add", "-A"], { cwd: root });
  const moved = checkCustody(root, recordOnly);
  assert.equal(moved.next.find((row) => row.path === "crates/a/src/kept.rs")?.class, "record-only");
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
  const copy = path.join(root, "crates/a/src/copy.cts");
  fs.copyFileSync(path.join(root, "crates/a/src/kept.rs"), copy);
  execFileSync("git", ["add", "."], { cwd: root });
  assert.equal(checkCustody(root, recordOnly).problems.length, 2);
  fs.unlinkSync(copy);
  fs.renameSync(path.join(root, "crates/a/src/kept.rs"), path.join(root, "crates/a/src/pinned.rs"));
  execFileSync("git", ["add", "-A"], { cwd: root });

  // Editing the enforced file without re-pinning it is a custody violation.
  fs.writeFileSync(path.join(root, "crates/a/src/pinned.rs"), "pub fn pinned() { }\n");
  assert.equal(checkCustody(root, registry).problems.length, 1);
  const registered = fs.readFileSync(path.join(root, registryPath), "utf8");
  assert.equal(custodyMain(["--write"], root), 1);
  assert.equal(fs.readFileSync(path.join(root, registryPath), "utf8"), registered);

  // A record-only row only goes stale.
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

test("freezing requires a tracked regular fixture with every recorded digest", (t) => {
  const source = "crates/a/src/example.rs";
  const bytes = "pub fn original() {}\n";
  const digests = digestsOf(Buffer.from(bytes));
  const root = repository({
    [source]: bytes,
    "tests/tooling/authority.ts": JSON.stringify(digests),
  });
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const original = checkCustody(root, new Map()).next;
  const registry = new Map(original.map((row) => [row.path, row]));
  assert.deepEqual(original[0].digests, [`sha256:${digests.sha256}`, `blob:${digests.blob}`]);
  fs.writeFileSync(path.join(root, source), "pub fn current() {}\n");

  const fixture = path.join(root, "tests/_fixtures/byte-custody", source);
  fs.mkdirSync(path.dirname(fixture), { recursive: true });
  fs.writeFileSync(fixture, bytes);
  assert.deepEqual(checkCustody(root, registry).unresolved, [source]);
  execFileSync("git", ["add", "--", fixture], { cwd: root });
  fs.writeFileSync(fixture, "changed\n");
  assert.deepEqual(checkCustody(root, registry).unresolved, [source]);
  execFileSync("git", ["add", "--", fixture], { cwd: root });
  fs.writeFileSync(fixture, bytes);
  assert.deepEqual(checkCustody(root, registry).unresolved, [source]);
  fs.unlinkSync(fixture);
  const copy = path.join(root, "stand-in.rs");
  fs.writeFileSync(copy, bytes);
  fs.symlinkSync(copy, fixture);
  assert.deepEqual(checkCustody(root, registry).unresolved, [source]);
  execFileSync("git", ["add", "--", fixture], { cwd: root });
  fs.unlinkSync(fixture);
  fs.writeFileSync(fixture, bytes);
  assert.deepEqual(checkCustody(root, registry).unresolved, [source]);
  execFileSync("git", ["add", "--", fixture], { cwd: root });

  const partial = {
    ...original[0],
    digests: [`sha256:${digests.sha256}`, `blob:${"0".repeat(40)}`],
  };
  assert.deepEqual(checkCustody(root, new Map([[source, partial]])).unresolved, [source]);
  const frozen = checkCustody(root, registry);
  assert.deepEqual(frozen.problems, []);
  assert.deepEqual(frozen.next, []);
  fs.mkdirSync(path.join(root, "tools/config"), { recursive: true });
  fs.writeFileSync(path.join(root, registryPath), renderRegistry(original));
  assert.equal(custodyMain(["--write"], root), 0);
  assert.equal(parseRegistry(fs.readFileSync(path.join(root, registryPath), "utf8")).size, 0);
});
