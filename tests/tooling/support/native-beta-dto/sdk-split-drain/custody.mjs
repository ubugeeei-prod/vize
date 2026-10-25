import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const base = path.dirname(fileURLToPath(import.meta.url));
const copy = path.join(base, "corsa_client");
const stock = JSON.parse(fs.readFileSync(path.join(base, "stock-custody.json"), "utf8"));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const changed = ["src/api/client.rs", "src/api/snapshot.rs"];
const added = ["src/api/close_drain_tests.rs"];
const omitted = [".cargo-ok"];
const actual = fs
  .readdirSync(copy, { recursive: true })
  .filter((name) => fs.statSync(path.join(copy, name)).isFile())
  .sort();
assert.deepEqual(
  actual,
  [
    ...stock.files.map((row) => row.path).filter((name) => !omitted.includes(name)),
    ...added,
  ].sort(),
);
assert.equal(stock.vcs, "1c5d10bd71b2d5ca2a0d885abb14fcf8d4381d1f");
for (const row of stock.files) {
  if (changed.includes(row.path) || omitted.includes(row.path)) continue;
  assert.equal(
    hash(fs.readFileSync(path.join(copy, row.path))),
    row.sha256,
    `unmodified stock SDK file: ${row.path}`,
  );
}
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "corsa-drain-original-"));
try {
  for (const file of changed) {
    const target = path.join(temporary, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.copyFileSync(path.join(copy, file), target);
  }
  execFileSync("patch", ["--batch", "--reverse", "-p1", "-i", path.join(base, "sdk.patch")], {
    cwd: temporary,
  });
  for (const file of changed)
    assert.equal(
      hash(fs.readFileSync(path.join(temporary, file))),
      stock.files.find((row) => row.path === file).sha256,
      `reconstructed exact stock SDK source: ${file}`,
    );
} finally {
  fs.rmSync(temporary, { recursive: true, force: true });
}
const lock = fs.readFileSync("Cargo.lock", "utf8");
const stockLock = execFileSync("git", ["show", "567cfd90b7a:Cargo.lock"], { encoding: "utf8" });
assert.equal(
  lock,
  stockLock.replace(
    /(name = "corsa_client"\nversion = "2\.0\.0-beta\.1"\n)source = "registry\+https:\/\/github.com\/rust-lang\/crates.io-index"\nchecksum = "3e4a1ace5347ca6717e952f6b2fbc73ef46e641a8f3ce1143d13c366880381e1"\n/u,
    "$1",
  ),
  "only the exact local path override lock identity changes",
);
const role = {
  role: "LOCAL-PATCHED-SDK-VALIDATION; NOT STOCK OR PUBLISHED SDK",
  nativeSemanticCredit: false,
  source: execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
  stock: {
    version: stock.version,
    vcs: stock.vcs,
    custodySha256: hash(fs.readFileSync(path.join(base, "stock-custody.json"))),
  },
  patchSha256: hash(fs.readFileSync(path.join(base, "sdk.patch"))),
  files: actual.map((file) => ({
    path: file,
    sha256: hash(fs.readFileSync(path.join(copy, file))),
  })),
  vizeCallerFiles: [
    ".cargo/config.toml",
    "Cargo.lock",
    "crates/vize_canon/src/lsp_client/editor_lsp.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/project_configuration.rs",
  ].map((file) => ({ path: file, sha256: hash(fs.readFileSync(file)) })),
};
fs.mkdirSync("target/vize-tests/native-beta-dto-composition", { recursive: true });
fs.writeFileSync(
  "target/vize-tests/native-beta-dto-composition/local-patched-sdk-role.json",
  JSON.stringify(role, null, 2) + "\n",
);
console.log(
  "Local SDK custody, exact two-file product patch, new test and isolated lock delta verified.",
);
