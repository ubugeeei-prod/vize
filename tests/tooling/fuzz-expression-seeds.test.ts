import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("expression regression corpus retains the complete issue 7808 input after regeneration", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-expression-seeds-"));
  try {
    fs.writeFileSync(path.join(root, "Cargo.toml"), "[workspace]\n");
    fs.writeFileSync(path.join(root, "pnpm-workspace.yaml"), "packages: []\n");
    const relative = "tests/fuzz/regressions/js_ts_expression/issue-7808.input";
    const bytes = fs.readFileSync(path.join(repoRoot, relative));
    assert.equal(bytes.length, 22_933);
    assert.equal(
      createHash("sha256").update(bytes).digest("hex"),
      "8670c91bace6df50473b3ad261b306446fb59b345d8d644a9771aa5aa4a0e5cb",
    );
    fs.mkdirSync(path.dirname(path.join(root, relative)), { recursive: true });
    fs.writeFileSync(path.join(root, relative), bytes);
    const corpus = path.join(root, "tests/fuzz/corpus/js_ts_expression");
    const digest = createHash("sha1").update(bytes).digest("hex").slice(0, 16);
    for (let pass = 0; pass < 2; pass++) {
      fs.mkdirSync(corpus, { recursive: true });
      fs.writeFileSync(path.join(corpus, "stale-seed"), "discard me");
      const result = spawnSync(
        "rust-script",
        [path.join(repoRoot, "tools/commands/ci/fuzz/seed_corpus.rs")],
        { encoding: "utf8", env: { ...process.env, VIZE_REPO_ROOT: root } },
      );
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
      assert.match(result.stdout, /1 JS\/TS expression entries/);
      assert.deepEqual(fs.readdirSync(corpus), [digest]);
      assert.deepEqual(fs.readFileSync(path.join(corpus, digest)), bytes);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("expression regression corpus seeds every retained campaign artifact byte-for-byte", () => {
  // Sorted like `readdirSync`: name, byte length, SHA-256 of the exact artifact.
  const retained = [
    [
      "issue-10401.input",
      23_857,
      "6d668844f745732cdc92bdafa21f72ba5d09d9c14305d4ea9174fb802493dcc8",
    ],
    [
      "issue-10407.input",
      23_588,
      "e2b83e39a35f555269b7599742d5792c07534ea0d4b1bf10584479785b89e3f9",
    ],
    [
      "issue-10845.input",
      23_857,
      "60c49daaab7e06a062b0ffa7dced0eb4ddab9dbee1b52f1657fc4f1c08aa14a5",
    ],
    [
      "issue-10845.min.input",
      2_133,
      "73e51fe565438b9e18d8696f4bc57d368310898539e1f406387b7592a9e1a926",
    ],
    [
      "issue-10845.slow-1.input",
      22_125,
      "de0029da1a229d4e336d73374bedd33e64649c741a28752d56b4dee40076f60a",
    ],
    [
      "issue-10845.slow-2.input",
      23_857,
      "e73212cb497085f26e85b59ef09aabc4bbdb485965f71ebb8899dc666b89e293",
    ],
    [
      "issue-10937.input",
      23_857,
      "a8b4b31b2c8adfe06e1a63e098fa9eb8e72c5e754f0a4a84179fae71b465df68",
    ],
    [
      "issue-10937.slow-1.input",
      23_857,
      "3c0863f451cb1cfcf65508cc5a63ca4a2167c56a262f6056286875ccd5c6eecd",
    ],
    [
      "issue-10937.slow-2.input",
      23_857,
      "b2b4ed3b25c9b0cb5d72b054e25b139a601f83628adc0f57fd59a543bd153cb8",
    ],
    [
      "issue-11053.input",
      23_262,
      "bf1eda03d323adcf55f942e44e7048a5e28c2d833fc100d52db75581359d9080",
    ],
    [
      "issue-11053.slow-1.input",
      23_261,
      "e7af98c99e3f841291f31ed2e56b228a807c71dc41d0ebd41b3720330009b069",
    ],
    [
      "issue-7808.input",
      22_933,
      "8670c91bace6df50473b3ad261b306446fb59b345d8d644a9771aa5aa4a0e5cb",
    ],
    [
      "issue-9931.input",
      23_134,
      "c09b3d26faca0cee9fb4a57389d2a2393d62f54c8c4d9c7a2091b808e342930f",
    ],
    [
      "slow-unit-20ec7d2257bd09bef36390f4003322c72eb32f0a",
      23_436,
      "6914c3b2e778e7d8c084a94a82dcd61a037bbe8c3fb7be5c35e084f937ed2957",
    ],
    [
      "slow-unit-a117c12decb552d05f038566e9a966039d91acb1",
      23_424,
      "94471c1713309f60403603d9f407823e73b7a603072aa0f3c4fda7eed826e38a",
    ],
  ] as const;
  const directory = "tests/fuzz/regressions/js_ts_expression";
  assert.deepEqual(
    fs
      .readdirSync(path.join(repoRoot, directory))
      .filter((name) => !name.startsWith("."))
      .sort(),
    retained.map(([name]) => name),
  );
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-expression-seeds-"));
  try {
    fs.writeFileSync(path.join(root, "Cargo.toml"), "[workspace]\n");
    fs.writeFileSync(path.join(root, "pnpm-workspace.yaml"), "packages: []\n");
    fs.mkdirSync(path.join(root, directory), { recursive: true });
    const seeds = new Map<string, Buffer>();
    for (const [name, length, sha256] of retained) {
      const bytes = fs.readFileSync(path.join(repoRoot, directory, name));
      assert.equal(bytes.length, length, name);
      assert.equal(createHash("sha256").update(bytes).digest("hex"), sha256, name);
      fs.writeFileSync(path.join(root, directory, name), bytes);
      seeds.set(createHash("sha1").update(bytes).digest("hex").slice(0, 16), bytes);
    }
    assert.equal(seeds.size, retained.length, "every artifact must seed its own corpus entry");
    const result = spawnSync(
      "rust-script",
      [path.join(repoRoot, "tools/commands/ci/fuzz/seed_corpus.rs")],
      { encoding: "utf8", env: { ...process.env, VIZE_REPO_ROOT: root } },
    );
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    assert.match(result.stdout, /15 JS\/TS expression entries/);
    const corpus = path.join(root, "tests/fuzz/corpus/js_ts_expression");
    assert.deepEqual(fs.readdirSync(corpus).sort(), [...seeds.keys()].sort());
    for (const [digest, bytes] of seeds) {
      assert.deepEqual(fs.readFileSync(path.join(corpus, digest)), bytes);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
