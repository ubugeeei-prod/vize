import assert from "node:assert/strict";
import { test } from "node:test";
import { archiveKeys, differences } from "../../tools/commands/levels/codegen-equivalence.ts";

const rust = (hash: string, index: number) => `sample-${hash}.a-cgu.${index}.rcgu.o`;
const members = (hash: string) =>
  archiveKeys(["lib.rmeta", "__.SYMDEF", rust(hash, 1), "support.o", rust(hash, 0)]).map(
    (member) => ({
      ...member,
      bytes: 12,
      sha256: "first",
    }),
  );
const build = (hash: string) => ({
  revision: hash,
  manifest: "crates/sample/Cargo.toml",
  target: { name: "custom", kind: ["lib"], source: "crates/sample/src/lib.rs" },
  lock: "lock",
  cargo: "cargo",
  rustc: "rustc",
  features: ["native"],
  profile: { opt_level: "3" },
  members: members(hash),
});

test("only metadata and Rust member filenames are excluded from direct byte comparison", () => {
  const a = build("aaaa");
  const b = build("bbbb");
  assert.deepEqual(differences(a, b), []);
  b.members[0].sha256 = "different";
  b.members[1].bytes = 15;
  assert.deepEqual(differences(a, b), ["rust:1", "member:support.o"]);
});

test("reordering, omissions, extra members and configuration changes fail the comparison", () => {
  const a = build("aaaa");
  const b = build("bbbb");
  b.members.reverse();
  b.features = [];
  assert.deepEqual(differences(a, b), ["features", "archive membership or order"]);
  b.members.pop();
  assert.deepEqual(differences(a, b), ["features", "archive membership or order"]);
  assert.deepEqual(differences(b, a), ["features", "archive membership or order"]);
  b.members = [...a.members];
  b.features = [...a.features];
  b.manifest = "other/Cargo.toml";
  b.target.name = "another";
  b.lock = "changed";
  b.profile.opt_level = "0";
  assert.deepEqual(differences(a, b), ["manifest", "target", "lock", "profile"]);
});

test("empty and ambiguous archives cannot pass", () => {
  assert.throws(() => archiveKeys(["lib.rmeta", "support.o"]), /no Rust codegen units/u);
  assert.throws(() => archiveKeys(["unknown.rcgu.o"]), /unrecognized Rust object/u);
  assert.throws(() => archiveKeys([rust("a", 0), rust("b", 0)]), /ambiguous/u);
  assert.throws(() => archiveKeys([rust("a", 0), "x.o", "x.o"]), /ambiguous/u);
});

test(
  "actual compiled products distinguish dormant, emitted and downstream-generic changes",
  {
    skip: process.env.VIZE_CODEGEN_PROBE !== "1",
  },
  async () => {
    const { execFileSync, spawnSync } = await import("node:child_process");
    const { createHash } = await import("node:crypto");
    const fs = await import("node:fs");
    const os = await import("node:os");
    const path = await import("node:path");
    const { fileURLToPath } = await import("node:url");
    const tool = fileURLToPath(
      new URL("../../tools/commands/levels/codegen-equivalence.ts", import.meta.url),
    );
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-codegen-fixture-"));
    const run = (...args: string[]) =>
      execFileSync(args[0], args.slice(1), { cwd: root, encoding: "utf8" }).trim();
    const source = path.join(root, "src/lib.rs");
    let commits = 0;
    const commit = (message: string) => {
      run("git", "add", "Cargo.toml", "Cargo.lock", "src", "helper", "build.rs");
      const date = new Date(Date.UTC(2001, 1, 3, 4, 5, 6 + commits++)).toISOString();
      execFileSync(
        "git",
        [
          "-c",
          "user.name=Fixture",
          "-c",
          "user.email=fixture@example.invalid",
          "-c",
          "commit.gpgsign=false",
          "commit",
          "-qm",
          message,
        ],
        { cwd: root, env: { ...process.env, GIT_AUTHOR_DATE: date, GIT_COMMITTER_DATE: date } },
      );
      return run("git", "rev-parse", "HEAD");
    };
    const compare = (base: string, head: string, product: string, status: number) => {
      const report = path.join(root, "report.json");
      const result = spawnSync(
        process.execPath,
        [tool, base, head, "sample", product, "default", report],
        { cwd: root, encoding: "utf8" },
      );
      assert.equal(result.status, status, result.stderr + result.stdout);
      const receipt = JSON.parse(fs.readFileSync(report, "utf8"));
      assert.equal(receipt.base.revision, base);
      assert.equal(receipt.head.revision, head);
      assert.equal(receipt.sourceDateEpoch, run("git", "show", "-s", "--format=%ct", base));
      if (receipt.differences.includes("executable")) {
        for (const side of ["base", "head"] as const) {
          const executable = path.join(root, receipt.executables[side]);
          const data = fs.readFileSync(executable);
          const time = new Date(Number(receipt.sourceDateEpoch) * 1000).toISOString().slice(11, 19);
          assert.ok(run(executable).includes('"Feb  3 2001" "' + time + '"'));
          assert.equal(data.length, receipt[side].members[0].bytes);
          assert.equal(
            createHash("sha256").update(data).digest("hex"),
            receipt[side].members[0].sha256,
          );
        }
      } else {
        assert.equal(receipt.executables, undefined);
      }
      return receipt.differences as string[];
    };
    try {
      run("git", "init", "-q");
      fs.mkdirSync(path.join(root, "src"));
      fs.mkdirSync(path.join(root, "helper", "src"), { recursive: true });
      fs.writeFileSync(
        path.join(root, "helper", "Cargo.toml"),
        '[package]\nname="helper"\nversion="0.1.0"\nedition="2024"\n[lib]\nname="sample"\n',
      );
      fs.writeFileSync(
        path.join(root, "helper", "src", "lib.rs"),
        "pub fn dependency() -> u32 { 7 }\n",
      );
      fs.writeFileSync(
        path.join(root, "Cargo.toml"),
        '[workspace]\nmembers=["helper"]\n[package]\nname="sample"\nversion="0.1.0"\nedition="2024"\n[lib]\nname="custom"\n[dependencies]\nhelper={path="helper"}\n[features]\nextra=[]\n[profile.ci-opt]\ninherits="release"\ncodegen-units=4\n',
      );
      fs.writeFileSync(
        source,
        '#[unsafe(no_mangle)] pub extern "C" fn emitted(a: u32) -> u32 { a.wrapping_add(1) }\npub fn generic<T: Into<u32>>(a: T) -> u32 { a.into().wrapping_mul(3) }\n',
      );
      fs.writeFileSync(path.join(root, "src/stamp.c"), "__DATE__ __TIME__\n");
      fs.writeFileSync(
        path.join(root, "build.rs"),
        [
          "fn main() {",
          '    let stamp = std::process::Command::new("cc").args(["-E", "-P", "src/stamp.c"]).output().unwrap();',
          "    assert!(stamp.status.success());",
          '    let dir = std::env::var_os("OUT_DIR").unwrap();',
          '    std::fs::write(std::path::Path::new(&dir).join("stamp.txt"), stamp.stdout).unwrap();',
          "}",
        ].join("\n"),
      );
      fs.writeFileSync(
        path.join(root, "src/main.rs"),
        'fn main() { println!("{} {}", include_str!(concat!(env!("OUT_DIR"), "/stamp.txt")).trim(), custom::emitted(std::hint::black_box(7)) + custom::generic(std::hint::black_box(3u8))); }\n',
      );
      run("cargo", "generate-lockfile");
      const base = commit("base");
      fs.appendFileSync(source, '#[cfg(feature="extra")] pub fn optional() -> u32 { 14 }\n');
      const gated = commit("gate new method");
      assert.deepEqual(compare(base, gated, "lib", 0), []);
      assert.deepEqual(compare(base, gated, "bin:sample", 0), []);
      fs.writeFileSync(
        source,
        fs.readFileSync(source, "utf8").replace("wrapping_add(1)", "wrapping_add(2)"),
      );
      const emitted = commit("change exported code");
      assert.ok(compare(gated, emitted, "lib", 1).some((key) => key.startsWith("rust:")));
      fs.writeFileSync(
        source,
        fs.readFileSync(source, "utf8").replace("wrapping_mul(3)", "wrapping_mul(5)"),
      );
      const generic = commit("change instantiated generic");
      assert.deepEqual(compare(emitted, generic, "bin:sample", 1), ["executable"]);
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);
