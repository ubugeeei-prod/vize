import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  buildPrPerformanceBinary,
  verifyPrPerformanceBuildPair,
  verifyPrPerformanceBuildReceipt,
} from "./pr-performance-build.ts";

void test("fresh normal builds support Cargo's own output path and legitimate identical optimized code", () => {
  const root = mkdtempSync(join(tmpdir(), "pr-performance-custody-"));
  try {
    const source = join(root, "source");
    mkdirSync(join(source, "crates"), { recursive: true });
    const git = (...args: string[]) => {
      const r = spawnSync(
        "git",
        ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", ...args],
        { cwd: source, encoding: "utf8" },
      );
      assert.equal(r.status, 0, r.stderr);
      return r.stdout.trim();
    };
    git("init", "--quiet");
    writeFileSync(
      join(source, "Cargo.toml"),
      '[profile.release]\nstrip="symbols"\n[profile.ci-opt]\ninherits="release"\nlto="thin"\ncodegen-units=16\n',
    );
    writeFileSync(join(source, "Cargo.lock"), "fixture lock\n");
    const compiler = join(root, "cargo.cjs");
    const fake = (fresh: boolean, assertions = false) => `#!${process.execPath}
if(process.argv.includes('--version')){console.log('fixture cargo 1');process.exit(0)}
const fs=require('node:fs'),p=require('node:path');const binary=p.join(process.env.CARGO_TARGET_DIR,'ci-opt','vize');
fs.mkdirSync(p.dirname(binary),{recursive:true});fs.writeFileSync(binary,'same optimized bytes',{mode:0o755});
console.log(JSON.stringify({reason:'compiler-artifact',target:{name:'vize',kind:['bin']},features:[],profile:{opt_level:'3',debuginfo:0,debug_assertions:${assertions},test:false,overflow_checks:false},executable:binary,fresh:${fresh}}));\n`;
    writeFileSync(compiler, fake(false), { mode: 0o755 });
    const commit = (body: string) => {
      writeFileSync(join(source, "crates", "code.rs"), body);
      git("add", ".");
      git("commit", "--quiet", "-m", "fixture source");
    };
    const build = (role: string, samePath = false) => {
      const targetDir = join(root, "target-" + role);
      return buildPrPerformanceBinary({
        sourceRoot: source,
        targetDir,
        binary: samePath ? join(targetDir, "ci-opt", "vize") : join(root, role + "-vize"),
        receipt: join(root, role + ".json"),
        role,
        cargoExecutable: compiler,
      });
    };
    commit("pub fn value() {}\n");
    const base = build("base", true);
    assert.equal(base.binary, base.cargoArtifact.executable);
    assert.equal(readFileSync(base.binary, "utf8"), "same optimized bytes");
    assert.equal(base.cargo.args.includes("--features"), false);
    assert.equal(
      base.cargo.args.some((x: string) => x.includes("line-tables-only")),
      false,
    );
    assert.equal(base.cargo.args.includes('profile.ci-opt.lto="thin"'), true);
    assert.equal(base.cargo.args.includes("profile.ci-opt.codegen-units=16"), true);
    commit("// dead code and formatting may not affect optimized bytes\npub fn value() {}\n");
    const head = build("head");
    const pair = verifyPrPerformanceBuildPair(join(root, "base.json"), join(root, "head.json"));
    assert.equal(pair.qualified, true);
    assert.equal(pair.identicalBinary, true);
    assert.notEqual(base.sourceSha, head.sourceSha);
    assert.notEqual(base.targetDir, head.targetDir);
    verifyPrPerformanceBuildReceipt(join(root, "head.json"), {
      role: "head",
      sourceSha: head.sourceSha,
    });
    assert.throws(() => verifyPrPerformanceBuildReceipt(join(root, "head.json"), { role: "base" }));
    assert.throws(() => build("base", true), /fresh, unshared/u);
    writeFileSync(compiler, fake(true), { mode: 0o755 });
    assert.throws(() => build("stale"), /fresh target/u);
    writeFileSync(compiler, fake(false, true), { mode: 0o755 });
    assert.throws(() => build("debug-assertions"), /disable debug assertions/u);
    writeFileSync(head.binary, "replaced");
    assert.throws(() => verifyPrPerformanceBuildReceipt(join(root, "head.json")), /bytes changed/u);
    writeFileSync(join(source, "crates", "code.rs"), "dirty source\n");
    assert.throws(() => build("dirty"), /clean immutable/u);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
