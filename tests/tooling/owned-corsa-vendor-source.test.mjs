import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { authenticateProcessReceiptVendorSource } from "../../tools/support/compat/github/owned-corsa-vendor-source.mjs";
import { ownedCorsaSource } from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { ownedCorsaReader } from "../../tools/support/compat/github/owned-corsa-source-reader.mjs";

const originalRoot = process.cwd();
const sha = (body) => createHash("sha256").update(body).digest("hex");
const env = { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" };
const git = (root, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd: root, input, env });
const before = fs.readFileSync(
  new URL("../_fixtures/ci/owned-corsa-vendor-checker-before.mjs.txt", import.meta.url),
);
assert.equal(sha(before), "8517ca30f9fde7c78aa00aaafe17673d337ac9f7d3118520c8452b16f6f65c49");
const checker = fs.readFileSync(
  new URL("../_fixtures/corsa-process-receipt/verify-vendor.mjs", import.meta.url),
);
const checkerPath = "tests/_fixtures/corsa-process-receipt/verify-vendor.mjs";
const helperPath = "tools/support/compat/github/owned-corsa-vendor-source.mjs";
function fixture(run) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vendor-source-"));
  try {
    const head = git(originalRoot, ["rev-parse", "HEAD"]).toString().trim();
    const reader = ownedCorsaReader(originalRoot, ownedCorsaSource.baseline, head);
    for (const row of ownedCorsaSource.rows) {
      const target = path.join(root, row.file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, reader("after", row.file).body, { mode: 0o644, flag: "wx" });
    }
    git(root, ["init", "--quiet"]);
    git(root, ["config", "user.name", "Vendor source control"]);
    git(root, ["config", "user.email", "source@example.invalid"]);
    const common = git(originalRoot, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(root, ".git/objects/info/alternates"),
      path.resolve(originalRoot, common, "objects") + "\n",
    );
    git(root, ["update-ref", "refs/heads/current", head]);
    git(root, ["symbolic-ref", "HEAD", "refs/heads/current"]);
    fs.mkdirSync(path.dirname(path.join(root, checkerPath)), { recursive: true });
    fs.mkdirSync(path.dirname(path.join(root, helperPath)), { recursive: true });
    fs.symlinkSync(
      new URL("../../tools/support/compat/github/owned-corsa-vendor-source.mjs", import.meta.url),
      path.join(root, helperPath),
    );
    run(root, head);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}
function sourceCommit(root, head, file, body) {
  git(root, ["read-tree", head]);
  const blob = git(root, ["hash-object", "-w", "--stdin"], body).toString().trim();
  git(root, ["update-index", "--add", "--cacheinfo", `100644,${blob},${file}`]);
  const tree = git(root, ["write-tree"]).toString().trim();
  const commit = git(root, ["commit-tree", tree, "-p", head], "private source control\n")
    .toString()
    .trim();
  git(root, ["update-ref", "refs/heads/current", commit]);
  return commit;
}

void test("the whole old checker genuinely refuses the owned graph; its finite successor preserves both complete vendor reports", () => {
  let inverse = before.toString();
  for (const [old, next] of [
    ['import { createHash } from "node:crypto";\n', ""],
    [
      'import assert from "node:assert/strict";\n',
      'import assert from "node:assert/strict";\nimport { authenticateProcessReceiptVendorSource } from "../../../tools/support/compat/github/owned-corsa-vendor-source.mjs";\n',
    ],
    [
      'const root = fileURLToPath(new URL("../../../", import.meta.url));\n',
      'const root = fileURLToPath(new URL("../../../", import.meta.url));\nconst vendorSource = authenticateProcessReceiptVendorSource(root);\n',
    ],
    [
      'const sha = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");\n',
      "const sha = vendorSource.checksum;\n",
    ],
    [
      "console.log(JSON.stringify(report, null, 2));\n",
      "console.log(JSON.stringify({ ...report, ownedSource: vendorSource.receipt }, null, 2));\n",
    ],
  ]) {
    assert.equal(inverse.split(old).length, 2);
    inverse = inverse.replace(old, next);
  }
  assert.deepEqual(Buffer.from(inverse), checker, "whole old checker inverse");
  fixture((root) => {
    const execute = (body) => {
      fs.writeFileSync(path.join(root, checkerPath), body, { mode: 0o644 });
      return spawnSync(process.execPath, [checkerPath], {
        cwd: root,
        env,
        encoding: "utf8",
        maxBuffer: 4 * 1024 * 1024,
      });
    };
    const manifests = [
      "corsa_core",
      "corsa_jsonrpc",
      "corsa_client",
      "corsa_lsp",
      "corsa_orchestrator",
      "corsa",
    ].map((name) => "vendor/" + name + "/Cargo.toml");
    const currentManifests = manifests.map((file) => [
      file,
      fs.readFileSync(path.join(root, file)),
    ]);
    for (const file of manifests)
      fs.writeFileSync(
        path.join(root, file),
        git(originalRoot, ["cat-file", "blob", "b2caebeab6fb5bc817b4e31a65d5d560449b691b:" + file]),
      );
    const red = execute(before);
    assert.equal(red.error, undefined);
    assert.equal(red.signal, null);
    assert.equal(red.status, 1);
    assert(red.stderr.includes("AssertionError [ERR_ASSERTION]: Cargo.toml"));
    assert(red.stderr.includes("d90342f5ab6e02f2bc254387d2c62c2f4e931909bb0d6e3ba4ec6bfbdf31f4d7"));
    assert(red.stderr.includes("b92c9f94f932a73e6d34e93c8cea493b744322b5c4892d6bf0b79654fd0c7e13"));
    for (const [file, body] of currentManifests) fs.writeFileSync(path.join(root, file), body);
    const currentRed = execute(before);
    assert.equal(currentRed.error, undefined);
    assert.equal(currentRed.signal, null);
    assert.equal(currentRed.status, 1);
    assert(currentRed.stderr.includes("AssertionError [ERR_ASSERTION]: Cargo.toml"));
    assert(
      currentRed.stderr.includes(
        sha(fs.readFileSync(path.join(root, "vendor/corsa_core/Cargo.toml"))),
      ),
    );
    assert(
      currentRed.stderr.includes(
        "b92c9f94f932a73e6d34e93c8cea493b744322b5c4892d6bf0b79654fd0c7e13",
      ),
    );
    const green = execute(checker);
    assert.equal(green.error, undefined);
    assert.equal(green.signal, null);
    assert.equal(green.status, 0, green.stdout + green.stderr);
    assert.equal(green.stderr, "");
    const report = JSON.parse(green.stdout);
    const custody = JSON.parse(
      fs.readFileSync(path.join(root, "vendor/corsa-published-custody.json")),
    );
    for (const [crate, modified, added] of [
      ["corsa_core", ["src/lib.rs", "src/process.rs"], ["LICENSE", "src/process_receipt.rs"]],
      ["corsa_lsp", ["src/lib.rs", "src/lsp/client.rs"], ["LICENSE"]],
    ])
      assert.deepEqual(report[crate], {
        archiveSha256: custody[crate].archiveSha256,
        originalFiles: Object.keys(custody[crate].files).length,
        unchanged: Object.keys(custody[crate].files).length - modified.length,
        modified,
        added,
        revision: "34bb731bdcaf5d4c154596b0c32912cbffd60b18",
      });
    assert.equal(report.ownedSource.completeProductionBodies, 237);
    assert.equal(report.ownedSource.completeVendorBodies, 214);
    assert.equal(report.ownedSource.runtimeCredit, false);
    console.log(
      JSON.stringify({
        scope: "source-only whole checker counterfactual",
        retainedBefore: { status: red.status, stdout: red.stdout, stderr: red.stderr },
        before: { status: currentRed.status, stdout: currentRed.stdout, stderr: currentRed.stderr },
        after: { status: green.status, stdout: green.stdout, stderr: green.stderr },
      }),
    );
  });
});

void test("real partial, physical body/mode/symlink and unknown vendor sources refuse before projection", () => {
  fixture((root) => {
    const file = path.join(root, "vendor/corsa_core/Cargo.toml");
    const body = fs.readFileSync(file);
    for (const mutation of [
      () => fs.writeFileSync(file, Buffer.concat([body, Buffer.from("\nforeign source\n")])),
      () => fs.chmodSync(file, 0o755),
      () => fs.unlinkSync(file),
      () => {
        fs.unlinkSync(file);
        fs.symlinkSync(path.join(originalRoot, "vendor/corsa_core/Cargo.toml"), file);
      },
    ]) {
      mutation();
      assert.throws(() => authenticateProcessReceiptVendorSource(root));
      if (fs.existsSync(file) || fs.lstatSync(path.dirname(file)).isDirectory()) {
        try {
          fs.unlinkSync(file);
        } catch (error) {
          assert.equal(error.code, "ENOENT");
        }
      }
      fs.writeFileSync(file, body, { mode: 0o644, flag: "wx" });
    }
    fs.writeFileSync(path.join(root, "vendor/corsa_core/unknown.rs"), "foreign owned source\n");
    assert.throws(
      () => authenticateProcessReceiptVendorSource(root),
      /complete six-package physical source census/u,
    );
  });
});

void test("real mixed historical manifests, unknown raw tree entries and replacement refs refuse whole source authority", () => {
  fixture((root, head) => {
    const file = "vendor/corsa_core/Cargo.toml";
    const actual = fs.readFileSync(path.join(root, file));
    const capsule = ownedCorsaSource.historicalCargoBodies.find(
      (row) => row.side === "after" && row.file === file,
    ).historical;
    const old = Buffer.from(capsule.rawBase64, "base64");
    fs.writeFileSync(path.join(root, file), old);
    sourceCommit(root, head, file, old);
    assert.throws(
      () => authenticateProcessReceiptVendorSource(root),
      /complete genuine237 source bodies/u,
    );
    git(root, ["update-ref", "refs/heads/current", head]);
    fs.writeFileSync(path.join(root, file), actual);
    const foreign = sourceCommit(
      root,
      head,
      "vendor/corsa_core/unknown.rs",
      Buffer.from("foreign raw source\n"),
    );
    assert.throws(
      () => authenticateProcessReceiptVendorSource(root),
      /complete six-package raw source census/u,
    );
    git(root, ["update-ref", "refs/heads/current", head]);
    git(root, ["update-ref", "refs/replace/" + head, foreign]);
    assert.throws(() => authenticateProcessReceiptVendorSource(root), /replacement refs refuse/u);
    git(root, ["update-ref", "-d", "refs/replace/" + head]);
    const valid = authenticateProcessReceiptVendorSource(root);
    assert.throws(
      () => valid.checksum(path.join(root, "unknown.rs")),
      /outside authenticated source scope/u,
    );
  });
});
