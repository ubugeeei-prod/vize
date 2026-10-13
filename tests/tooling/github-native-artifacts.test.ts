import assert from "node:assert/strict";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";

import { runToolCommand } from "./_helpers/tool-commands.ts";

const collect = "tools/support/github/collect-native-artifacts.ts";
const site = "tools/support/github/create-site-structure.ts";

function workspace(t: TestContext): string {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(tmpdir(), "github-native-artifacts-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return root;
}

function write(root: string, file: string, content: string): void {
  fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
  fs.writeFileSync(path.join(root, file), content);
}

void test("collected artifacts are flattened and listed in shortlex order", (t) => {
  const root = workspace(t);
  write(root, "src/nested/aa.node", "aa");
  write(root, "src/b.node", "b");
  write(root, "src/deep/er/a.node", "a");
  write(root, "src/zzzzzzzz.node", "long");
  write(root, "src/node_modules/skip.node", "skip");
  write(root, "src/pkg/node_modules/skip.node", "skip");
  write(root, "src/dir.node/inner/c.node", "c");
  write(root, "src/readme.md", "not native");
  if (process.platform !== "win32") {
    fs.symlinkSync("b.node", path.join(root, "src/link.node"));
    fs.symlinkSync("missing.node", path.join(root, "src/broken.node"));
  }

  const result = runToolCommand(collect, ["./src/", "out/deep/er", "fresco-native"], { cwd: root });

  assert.equal(result.status, 0, result.stderr);
  const copied = [
    "a",
    "b",
    "c",
    "aa",
    ...(process.platform === "win32" ? [] : ["link"]),
    "zzzzzzzz",
  ];
  assert.equal(
    result.stdout,
    [
      "Collected fresco-native artifacts:",
      ...copied.map((name) => `  ${path.join("out", "deep", "er", `${name}.node`)}`),
      "",
    ].join("\n"),
  );
  assert.deepEqual(
    fs.readdirSync(path.join(root, "out/deep/er")).sort(),
    copied.map((name) => `${name}.node`).sort(),
  );
  assert.equal(fs.readFileSync(path.join(root, "out/deep/er/aa.node"), "utf8"), "aa");
  if (process.platform !== "win32") {
    assert.equal(fs.readFileSync(path.join(root, "out/deep/er/link.node"), "utf8"), "b");
    // New artifacts are created like an ordinary 0o644 file under the same umask.
    fs.closeSync(fs.openSync(path.join(root, "reference"), "w", 0o644));
    assert.equal(
      fs.statSync(path.join(root, "out/deep/er/a.node")).mode & 0o777,
      fs.statSync(path.join(root, "reference")).mode & 0o777,
    );
  }
});

void test("an existing output directory keeps unrelated files and existing modes", (t) => {
  const root = workspace(t);
  write(root, "src/vize.node", "fresh");
  write(root, "out/keep.txt", "keep");
  write(root, "out/vize.node", "stale and longer");
  fs.chmodSync(path.join(root, "out/vize.node"), 0o600);

  const result = runToolCommand(collect, [path.join(root, "src"), path.join(root, "out"), "x"], {
    cwd: root,
  });

  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, `Collected x artifacts:\n  ${path.join(root, "out", "vize.node")}\n`);
  assert.equal(fs.readFileSync(path.join(root, "out/vize.node"), "utf8"), "fresh");
  assert.equal(fs.readFileSync(path.join(root, "out/keep.txt"), "utf8"), "keep");
  if (process.platform !== "win32")
    assert.equal(fs.statSync(path.join(root, "out/vize.node")).mode & 0o777, 0o600);
});

void test("missing artifacts and invalid outputs fail without partial success", (t) => {
  const root = workspace(t);
  write(root, "only-modules/node_modules/x.node", "x");
  write(root, "file", "not a directory");
  for (const source of ["only-modules", "absent", "file"]) {
    const result = runToolCommand(collect, [source, "out", "vize-native"], { cwd: root });
    assert.deepEqual(
      [result.status, result.stdout],
      [1, "No vize-native .node files found after build.\n"],
    );
  }
  assert.equal(fs.existsSync(path.join(root, "out")), false);

  write(root, "src/vize.node", "x");
  const blocked = runToolCommand(collect, ["src", "file", "vize-native"], { cwd: root });
  assert.equal(blocked.status, 1);
  assert.equal(blocked.stdout, "");
  assert.match(blocked.stderr, /EEXIST/u);

  const usage = runToolCommand(collect, ["src", "out"], { cwd: root });
  assert.equal(usage.status, 1);
  assert.match(
    usage.stdout,
    /^Usage: .*collect-native-artifacts\.ts <source-dir> <output-dir> <label>\n$/u,
  );
});

void test("the Pages tree refuses to merge into existing destinations", (t) => {
  for (const [name, prepare] of [
    ["existing site", (root: string) => fs.mkdirSync(path.join(root, "site"))],
    ["docs owns play", (root: string) => write(root, "artifacts/docs/play/index.html", "x")],
  ] as const) {
    const root = workspace(t);
    write(root, "artifacts/docs/index.html", "docs");
    write(root, "artifacts/playground/index.html", "play");
    write(root, "artifacts/musea-examples/index.html", "musea");
    write(root, "playground/public/CNAME", "example.com");
    prepare(root);
    const result = runToolCommand(site, [], { cwd: root });
    assert.equal(result.status, 1, name);
    assert.match(result.stderr, /EEXIST/u, name);
    assert.equal(fs.existsSync(path.join(root, "site/CNAME")), false, name);
  }
});

void test("the Pages tree copies nested artifacts and requires every input", (t) => {
  const root = workspace(t);
  const missing = runToolCommand(site, [], { cwd: root });
  assert.equal(missing.status, 1);
  assert.match(missing.stderr, /ENOENT/u);
  // The site root is created before the first missing input is read.
  assert.deepEqual(fs.readdirSync(path.join(root, "site")), []);

  fs.rmSync(path.join(root, "site"), { recursive: true });
  write(root, "artifacts/docs/guide/a/b.html", "b");
  write(root, "artifacts/playground/assets/app.js", "app");
  write(root, "artifacts/musea-examples/index.html", "musea");
  const withoutCname = runToolCommand(site, ["ignored"], { cwd: root });
  assert.equal(withoutCname.status, 1);
  assert.equal(fs.readFileSync(path.join(root, "site/guide/a/b.html"), "utf8"), "b");
  assert.equal(fs.readFileSync(path.join(root, "site/play/assets/app.js"), "utf8"), "app");
  assert.equal(fs.existsSync(path.join(root, "site/CNAME")), false);
});
