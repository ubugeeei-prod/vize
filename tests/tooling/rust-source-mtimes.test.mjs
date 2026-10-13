import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import {
  manifestName,
  recordSourceMtimes,
  restoreSourceMtimes,
} from "../../.github/actions/rust-source-mtimes/source-mtimes.mjs";

const git = (cwd, ...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const mtime = (path) => lstatSync(path, { bigint: true }).mtimeNs / 1_000_000_000n;
// Successive jobs are minutes apart; whole-second records need at least one.
const nextSecond = () => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 1100);

function repository(files) {
  const root = mkdtempSync(join(tmpdir(), "vize-source-mtimes-"));
  const origin = join(root, "origin");
  mkdirSync(origin);
  const write = (path, text) => {
    mkdirSync(dirname(join(origin, path)), { recursive: true });
    writeFileSync(join(origin, path), text);
  };
  const commit = (message) => {
    git(origin, "add", "-A");
    git(origin, "-c", "user.name=t", "-c", "user.email=t@example.test", "commit", "-qm", message);
  };
  git(origin, "init", "-q");
  for (const [path, text] of Object.entries(files)) write(path, text);
  commit("base");
  const disk = join(root, "disk");
  const work = join(root, "work");
  // A fresh checkout per job, with one persistent target directory.
  const checkout = () => {
    rmSync(work, { recursive: true, force: true });
    execFileSync("git", ["clone", "-q", origin, work]);
    if (existsSync(disk)) renameSync(disk, join(work, "target"));
    return work;
  };
  const park = ({ keep }) => {
    const target = join(work, "target");
    if (keep && existsSync(target)) renameSync(target, disk);
  };
  return { root, write, commit, checkout, park, disk };
}

await test("unchanged content regains its recorded mtime; changed and new files move forward", () => {
  const repo = repository({ "a.rs": "a\n", "b.rs": "b\n", "c.rs": "c\n" });
  try {
    let work = repo.checkout();
    let result = restoreSourceMtimes({ target: join(work, "target"), record: true });
    assert.equal(result.reason, "no manifest");
    assert.equal(result.changed, 3);
    const first = result.floor;
    recordSourceMtimes({ target: join(work, "target") });
    repo.park({ keep: true });

    nextSecond();
    repo.write("b.rs", "changed\n");
    repo.write("d.rs", "new\n");
    repo.commit("change");
    work = repo.checkout();
    result = restoreSourceMtimes({ target: join(work, "target"), record: false });
    assert.equal(result.reason, "complete manifest");
    assert.deepEqual([result.restored, result.changed], [2, 2]);
    assert.equal(mtime(join(work, "a.rs")), BigInt(first));
    assert.equal(mtime(join(work, "c.rs")), BigInt(first));
    assert.ok(mtime(join(work, "b.rs")) > BigInt(first));
    assert.equal(mtime(join(work, "b.rs")), mtime(join(work, "d.rs")));
    // A read-only consumer leaves the committed record untouched.
    const manifest = JSON.parse(readFileSync(join(work, "target", manifestName), "utf8"));
    assert.equal(manifest.state, "complete");
    assert.equal(manifest.files["b.rs"][0], git(work, "rev-parse", "HEAD~1:b.rs"));
  } finally {
    rmSync(repo.root, { recursive: true, force: true });
  }
});

await test("files rewritten during a trusted run never keep the record of their checkout", () => {
  const repo = repository({ "a.rs": "a\n", "b.rs": "b\n" });
  try {
    let work = repo.checkout();
    restoreSourceMtimes({ target: join(work, "target"), record: true });
    recordSourceMtimes({ target: join(work, "target") });
    repo.park({ keep: true });

    nextSecond();
    work = repo.checkout();
    restoreSourceMtimes({ target: join(work, "target"), record: true });
    // Only a run that finished may be trusted later.
    assert.equal(
      restoreSourceMtimes({ target: join(work, "target"), record: false }).reason,
      "unfinished manifest",
    );
    writeFileSync(join(work, "a.rs"), "generated during the run\n");
    const result = recordSourceMtimes({ target: join(work, "target") });
    assert.equal(result.rewritten, 1);
    repo.park({ keep: true });

    nextSecond();
    work = repo.checkout();
    const restored = restoreSourceMtimes({ target: join(work, "target"), record: false });
    // The checkout content differs from what the run last built, so it is new.
    assert.deepEqual([restored.restored, restored.changed], [1, 1]);
    assert.ok(mtime(join(work, "a.rs")) > mtime(join(work, "b.rs")));
  } finally {
    rmSync(repo.root, { recursive: true, force: true });
  }
});

await test("edits before the step, foreign roots and shared secondary targets stay conservative", () => {
  const repo = repository({ "a.rs": "a\n", "b.rs": "b\n" });
  try {
    let work = repo.checkout();
    restoreSourceMtimes({ target: join(work, "target"), record: true });
    recordSourceMtimes({ target: join(work, "target") });
    repo.park({ keep: true });

    nextSecond();
    work = repo.checkout();
    writeFileSync(join(work, "b.rs"), "edited before restoration\n");
    const before = mtime(join(work, "b.rs"));
    const restored = restoreSourceMtimes({ target: join(work, "target"), record: false });
    assert.deepEqual([restored.restored, restored.changed], [1, 1]);
    assert.ok(mtime(join(work, "b.rs")) >= before);

    const shared = restoreSourceMtimes({
      target: join(work, "target"),
      secondary: join(work, "other-target"),
      record: true,
    });
    assert.equal(shared.applied, false);
    assert.equal(existsSync(join(work, "target", manifestName)), false);

    const manifest = join(work, "target", manifestName);
    restoreSourceMtimes({ target: join(work, "target"), record: true });
    recordSourceMtimes({ target: join(work, "target") });
    const moved = JSON.parse(readFileSync(manifest, "utf8"));
    writeFileSync(manifest, JSON.stringify({ ...moved, root: "/elsewhere" }));
    assert.equal(
      restoreSourceMtimes({ target: join(work, "target"), record: false }).reason,
      "incompatible manifest",
    );
  } finally {
    rmSync(repo.root, { recursive: true, force: true });
  }
});

await test("release binaries keep rebuilding workspace crates from a fresh checkout", () => {
  const release = parse(
    readFileSync(new URL("../../.github/workflows/release.yml", import.meta.url), "utf8"),
  );
  const mounts = Object.values(release.jobs).flatMap((job) =>
    (job.steps ?? []).filter(({ uses }) => uses === "./.github/actions/setup-rust-sticky-cache"),
  );
  assert.ok(mounts.length > 0);
  for (const mount of mounts) assert.equal(mount.with?.["source-mtimes"], "false", mount.with?.key);
});

// Probe outside this checkout so its pinned toolchain file does not apply.
const cargo = spawnSync("cargo", ["--version"], { cwd: tmpdir(), encoding: "utf8" });

await test(
  "Cargo rebuilds exactly what a persistent checkout would rebuild",
  { skip: cargo.status !== 0 && "cargo is unavailable" },
  () => {
    const repo = repository({
      "Cargo.toml": '[workspace]\nmembers = ["a", "b", "c"]\nresolver = "2"\n',
      "a/Cargo.toml": '[package]\nname = "a"\nversion = "0.1.0"\nedition = "2021"\n',
      "a/src/lib.rs": "pub fn a() -> u32 { 1 }\n",
      "a/data.txt": "one\n",
      "b/Cargo.toml":
        '[package]\nname = "b"\nversion = "0.1.0"\nedition = "2021"\n[dependencies]\na = { path = "../a" }\n',
      "b/src/lib.rs":
        'pub fn b() -> usize { a::a() as usize + include_str!("../../a/data.txt").len() }\n',
      // No rerun-if directive: Cargo scans the whole package.
      "c/Cargo.toml": '[package]\nname = "c"\nversion = "0.1.0"\nedition = "2021"\n',
      "c/build.rs":
        'fn main() { println!("cargo:rustc-env=C_LEN={}", std::fs::read_to_string("input.txt").unwrap().len()); }\n',
      "c/input.txt": "abc\n",
      "c/src/lib.rs": 'pub fn c() -> &\'static str { env!("C_LEN") }\n',
    });
    const job = ({ record, mutate }) => {
      nextSecond();
      const work = repo.checkout();
      const target = join(work, "target");
      const snapshot = join(repo.root, "snapshot");
      if (!record && existsSync(target))
        cpSync(target, snapshot, { recursive: true, preserveTimestamps: true });
      restoreSourceMtimes({ target, record });
      mutate?.(work);
      const output = execFileSync("cargo", ["build", "--message-format=json"], {
        cwd: work,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "ignore"],
      });
      const built = new Set();
      for (const line of output.split("\n").filter((line) => line.startsWith("{"))) {
        const message = JSON.parse(line);
        if (message.reason === "compiler-artifact" && !message.fresh)
          built.add(message.target.name);
      }
      if (record) recordSourceMtimes({ target });
      repo.park({ keep: record });
      // A read-only clone is discarded, like a never-committed sticky disk.
      if (!record && existsSync(snapshot)) renameSync(snapshot, repo.disk);
      return [...built]
        .filter((name) => name !== "build-script-build")
        .sort((a, b) => a.localeCompare(b));
    };
    try {
      assert.deepEqual(job({ record: true }), ["a", "b", "c"]);
      assert.deepEqual(job({ record: true }), []);
      repo.write("a/src/lib.rs", "pub fn a() -> u32 { 2 }\n");
      repo.commit("change a");
      assert.deepEqual(job({ record: true }), ["a", "b"]);
      repo.write("c/input.txt", "abcdef\n");
      repo.commit("change c");
      assert.deepEqual(job({ record: false }), ["c"]);
      assert.deepEqual(job({ record: true }), ["c"]);
      const rewrite = (work) => writeFileSync(join(work, "a/data.txt"), "generated\n");
      assert.deepEqual(job({ record: true, mutate: rewrite }), ["b"]);
      assert.deepEqual(job({ record: false }), ["b"]);
    } finally {
      rmSync(repo.root, { recursive: true, force: true });
    }
  },
);
