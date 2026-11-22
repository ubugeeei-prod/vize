import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { verifyNativeAddons } from "../../tools/commands/levels/native-addon-identity.ts";

void test("cache-controlled comparison rejects changed machine code and an unrelated raw identity", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "native-addon-control-"));
  const file = path.join(dir, "report.json");
  try {
    function side(name: string, digit: string) {
      const revision = digit.repeat(40);
      const pluginHostBuildId = revision + ":" + digit.repeat(64);
      const image = Buffer.from("linked code / " + pluginHostBuildId);
      fs.writeFileSync(path.join(dir, name), image);
      return {
        revision,
        pluginHostBuildId,
        members: [
          {
            key: "node-addon",
            bytes: image.length,
            sha256: createHash("sha256").update(image).digest("hex"),
          },
        ],
      };
    }
    const base = side("base", "a"),
      head = side("head", "b");
    const control = {
      produced: head.pluginHostBuildId,
      compiled: base.pluginHostBuildId,
      evidence: { ...head, members: base.members },
      differences: [],
    };
    const report = {
      package: "vize_vitrine",
      product: "node-addon",
      differences: ["node-addon"],
      base,
      head,
      pluginIdentityControl: control,
      executables: { base: "base", head: "head" },
    };
    const verify = () => {
      fs.writeFileSync(file, JSON.stringify(report));
      return verifyNativeAddons(file);
    };
    assert.equal(verify().controlled.sha256, base.members[0].sha256);
    control.evidence.members = head.members;
    assert.throws(verify, /cache-controlled native addon differs/u);
    control.evidence.members = base.members;
    control.compiled = head.pluginHostBuildId;
    assert.throws(verify);
    control.compiled = base.pluginHostBuildId;
    head.pluginHostBuildId = head.revision + ":" + "c".repeat(64);
    assert.throws(verify, /missing native cache identity/u);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

void test(
  "independent stamped builds detect an exported code change with the same cache key",
  {
    skip: process.env.VIZE_CODEGEN_PROBE !== "1" || process.platform === "win32",
  },
  () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "native-addon-build-"));
    const tool = fileURLToPath(
      new URL("../../tools/commands/levels/codegen-equivalence.ts", import.meta.url),
    );
    const run = (bin: string, ...args: string[]) =>
      execFileSync(bin, args, {
        cwd: root,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      }).trim();
    const commit = (subject: string) => {
      run("git", "add", "Cargo.toml", "Cargo.lock", "lib.rs", "build.rs");
      run(
        "git",
        "-c",
        "user.email=fixture@example.invalid",
        "-c",
        "user.name=Fixture",
        "-c",
        "commit.gpgsign=false",
        "commit",
        "-qm",
        subject,
      );
      return run("git", "rev-parse", "HEAD");
    };
    const compare = (base: string, head: string) => {
      const file = path.join(root, "report.json");
      const child = spawnSync(
        process.execPath,
        [
          tool,
          base,
          head,
          "vize_vitrine",
          "node-addon",
          "default",
          file,
          "--control-plugin-identity",
        ],
        { cwd: root, encoding: "utf8" },
      );
      assert.equal(child.status, 1, child.stderr + child.stdout);
      return () => verifyNativeAddons(file);
    };
    try {
      run("git", "init", "-q");
      fs.writeFileSync(
        path.join(root, "Cargo.toml"),
        '[package]\nname="vize_vitrine"\nversion="0.1.0"\nedition="2024"\n[lib]\ncrate-type=["cdylib"]\npath="lib.rs"\n[features]\nnapi=[]\nlegacy=[]\n[profile.ci-opt]\ninherits="release"\n',
      );
      fs.writeFileSync(
        path.join(root, "build.rs"),
        [
          "fn main() {",
          ' let output=std::process::Command::new("git").args(["rev-parse","HEAD"]).output().unwrap();',
          " let sha=std::str::from_utf8(&output.stdout).unwrap().trim();",
          ' println!("cargo:rustc-env=VIZE_PLUGIN_HOST_BUILD_ID={sha}:{}", "a".repeat(64));',
          "}",
        ].join("\n"),
      );
      const src = path.join(root, "lib.rs");
      fs.writeFileSync(
        src,
        '#[unsafe(no_mangle)] pub extern "C" fn cache() -> *const u8 { env!("VIZE_PLUGIN_HOST_BUILD_ID").as_ptr() }\n#[unsafe(no_mangle)] pub extern "C" fn value() -> u32 { 7 }\n',
      );
      run("cargo", "generate-lockfile");
      const base = commit("initial native export");
      fs.appendFileSync(src, "// source-only edit\n");
      const edited = commit("original exported code");
      assert.doesNotThrow(compare(base, edited));
      fs.writeFileSync(src, fs.readFileSync(src, "utf8").replace("{ 7 }", "{ 8 }"));
      const changed = commit("changed result");
      assert.throws(compare(edited, changed));
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);
