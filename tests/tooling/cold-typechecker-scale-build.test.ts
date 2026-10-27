import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  buildColdScaleBinary,
  verifyColdScaleBuildPair,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-build.ts";

test("immutable role builds reject shared targets, same-binary substitutions and changed executables", () => {
  const root = mkdtempSync(join(tmpdir(), "cold-build-identity-"));
  try {
    const source = join(root, "source");
    mkdirSync(join(source, "crates"), { recursive: true });
    function git(...args: string[]) {
      const result = spawnSync("git", args, { cwd: source, encoding: "utf8" });
      assert.equal(result.status, 0, result.stderr);
    }
    git("init", "--quiet");
    const cargo = join(root, "fake-cargo.cjs");
    // This compiler stub tests build custody, not product code generation.
    writeFileSync(
      cargo,
      `#!${process.execPath}
if(process.argv.includes('--version')){console.log('fixture-cargo 1');process.exit(0)}
const fs=require('node:fs'), path=require('node:path');
const binary=path.join(process.env.CARGO_TARGET_DIR,'ci-opt','vize');
fs.mkdirSync(path.dirname(binary),{recursive:true});
fs.writeFileSync(binary,fs.readFileSync('crates/answer.rs'),{mode:0o755});
console.log(JSON.stringify({reason:'compiler-artifact',target:{name:'vize',kind:['bin']},executable:binary,fresh:false}));\n`,
      { mode: 0o755 },
    );
    function commit(content: string) {
      writeFileSync(join(source, "crates", "answer.rs"), content);
      git("add", "crates/answer.rs");
      git(
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "fixture source",
      );
    }
    function build(role: string) {
      return buildColdScaleBinary({
        sourceRoot: source,
        targetDir: join(root, "targets", role),
        binary: join(root, "binaries", role),
        receipt: join(root, role + ".json"),
        role,
        allocation: false,
        cargoExecutable: cargo,
      });
    }
    commit("pub fn answer() -> u8 { 1 }\n");
    const base = build("normal-base");
    commit("pub fn answer() -> u8 { 2 }\n");
    const head = build("normal-head");
    const basePath = join(root, "normal-base.json"),
      headPath = join(root, "normal-head.json");
    const pair = verifyColdScaleBuildPair(basePath, headPath, source);
    assert.equal(pair.qualified, true);
    assert.deepEqual(pair.productionDiff, ["crates/answer.rs"]);
    assert.notEqual(base.binarySha256, head.binarySha256);
    assert.equal(head.cargoArtifact.fresh, false);
    assert.equal(readFileSync(head.binary, "utf8"), "pub fn answer() -> u8 { 2 }\n");
    const modified = join(root, "tampered.json");
    writeFileSync(modified, JSON.stringify({ ...head, targetDir: base.targetDir }));
    assert.throws(() => verifyColdScaleBuildPair(basePath, modified, source), /must not share/u);
    writeFileSync(
      modified,
      JSON.stringify({
        ...head,
        cargo: { ...head.cargo, args: [...head.cargo.args, "--features", "profiling"] },
      }),
    );
    assert.throws(
      () => verifyColdScaleBuildPair(basePath, modified, source),
      /equal compiler flags/u,
    );
    const saved = readFileSync(head.binary);
    copyFileSync(base.binary, head.binary);
    writeFileSync(
      modified,
      JSON.stringify({ ...head, binarySha256: base.binarySha256, binaryBytes: base.binaryBytes }),
    );
    assert.throws(
      () => verifyColdScaleBuildPair(basePath, modified, source),
      /executable bytes are identical/u,
    );
    assert.throws(
      () => verifyColdScaleBuildPair(basePath, headPath, source),
      /executable was replaced/u,
    );
    writeFileSync(head.binary, saved);
    assert.throws(() => build("normal-head"), /fresh, unshared Cargo target/u);
    assert.equal(readFileSync(head.binary, "utf8"), saved.toString("utf8"));
    writeFileSync(join(source, "crates", "answer.rs"), "dirty source\n");
    assert.throws(() => build("dirty-head"), /clean immutable source/u);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
