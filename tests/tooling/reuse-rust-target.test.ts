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
} from "../../.github/actions/reuse-rust-target/source-mtimes.ts";
import type { RestoreResult } from "../../.github/actions/reuse-rust-target/source-mtimes.ts";
import { pruneRunOutputs } from "../../.github/actions/reuse-rust-target/prune-run-outputs.ts";

const git = (cwd: string, ...args: string[]) =>
  execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const mtime = (path: string) => lstatSync(path, { bigint: true }).mtimeNs / 1_000_000_000n;
function applied(result: RestoreResult) {
  if (!result.applied) throw new Error(`expected restored mtimes: ${result.reason}`);
  return result;
}

// Successive CI jobs are minutes apart. The conservative floor may sit up to
// two whole seconds after the previous run's post step, so leave more than that.
const nextSecond = () => Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 2100);

function repository(files: Record<string, string>) {
  const root = mkdtempSync(join(tmpdir(), "vize-reuse-rust-target-"));
  const origin = join(root, "origin");
  mkdirSync(origin);
  const write = (path: string, text: string) => {
    mkdirSync(dirname(join(origin, path)), { recursive: true });
    writeFileSync(join(origin, path), text);
  };
  const commit = (message: string) => {
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
  const park = ({ keep }: { keep: boolean }) => {
    const target = join(work, "target");
    if (keep && existsSync(target)) renameSync(target, disk);
  };
  return { root, write, commit, checkout, park, disk };
}

await test("unchanged content regains its recorded mtime; changed and new files move forward", () => {
  const repo = repository({ "a.rs": "a\n", "b.rs": "b\n", "c.rs": "c\n" });
  try {
    let work = repo.checkout();
    let result = applied(restoreSourceMtimes({ target: join(work, "target"), record: true }));
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
    result = applied(restoreSourceMtimes({ target: join(work, "target"), record: false }));
    assert.equal(result.reason, "complete manifest");
    assert.deepEqual([result.restored, result.changed], [2, 2]);
    assert.equal(mtime(join(work, "a.rs")), BigInt(first));
    assert.equal(mtime(join(work, "c.rs")), BigInt(first));
    assert.ok(mtime(join(work, "b.rs")) > BigInt(first));
    assert.equal(mtime(join(work, "b.rs")), mtime(join(work, "d.rs")));
    // A read-only consumer leaves the committed record untouched.
    const manifest = JSON.parse(readFileSync(join(work, "target", manifestName), "utf8")) as {
      state: string;
      files: Record<string, [string, number]>;
    };
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
    const restored = applied(restoreSourceMtimes({ target: join(work, "target"), record: false }));
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
    const restored = applied(restoreSourceMtimes({ target: join(work, "target"), record: false }));
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
    const moved = JSON.parse(readFileSync(manifest, "utf8")) as Record<string, unknown>;
    writeFileSync(manifest, JSON.stringify({ ...moved, root: "/elsewhere" }));
    assert.equal(
      restoreSourceMtimes({ target: join(work, "target"), record: false }).reason,
      "incompatible manifest",
    );
  } finally {
    rmSync(repo.root, { recursive: true, force: true });
  }
});

await test("release binaries reuse verified Cargo caches without accepting cached run outputs", () => {
  const release = parse(
    readFileSync(new URL("../../.github/workflows/release.yml", import.meta.url), "utf8"),
  ) as {
    jobs: Record<string, { steps?: Array<{ uses?: string; with?: Record<string, string> }> }>;
  };
  const mounts = Object.values(release.jobs).flatMap((job) =>
    (job.steps ?? []).filter(({ uses }) => uses === "./.github/actions/setup-rust-sticky-cache"),
  );
  assert.ok(mounts.length > 0);
  for (const mount of mounts) assert.equal(mount.with?.["reuse-target"], "true", mount.with?.key);
});

await test("a reused target keeps only Cargo's build cache", () => {
  const target = mkdtempSync(join(tmpdir(), "vize-prune-target-"));
  const touch = (path: string) => {
    mkdirSync(dirname(join(target, path)), { recursive: true });
    writeFileSync(join(target, path), "x");
  };
  const kept = [
    ".rustc_info.json",
    "CACHEDIR.TAG",
    manifestName,
    "ci/.fingerprint/vize-1/lib-vize",
    "ci/deps/libvize-1.rlib",
    "ci/build/vize-2/out/generated.rs",
    "ci/incremental/vize-3/s",
    "ci/examples/cli_build_result_oracle",
    "wasm32-unknown-unknown/release/.fingerprint/vitrine-4/lib",
    "wasm32-unknown-unknown/release/deps/vize_vitrine.wasm",
    "llvm-cov-target/debug/.fingerprint/carton-5/lib",
    "contract-guests/wasm32-wasip2/release/.fingerprint/guest-6/lib",
  ];
  const removed = [
    "ci/vize",
    "ci/vize.d",
    "ci/vize.differential-build.json",
    "ci/receipts/run.json",
    "wasm32-unknown-unknown/release/vize_vitrine.wasm",
    "wasm32-unknown-unknown/notes.txt",
    "differential/formatter/observations.json",
    "oxlint-original-project-transport/run-1/package.json",
    "template-style-fixes-7905.json",
    "nextest/pr/junit.xml",
    "tmp/scratch",
    "doc/vize/index.html",
    "llvm-cov/source-summary.json",
  ];
  try {
    for (const path of [...kept, ...removed]) touch(path);
    pruneRunOutputs(target);
    for (const path of kept) assert.ok(existsSync(join(target, path)), `${path} must stay`);
    for (const path of removed) assert.ok(!existsSync(join(target, path)), `${path} must go`);
    for (const directory of ["differential", "nextest", "tmp", "doc", "ci/receipts"])
      assert.ok(!existsSync(join(target, directory)), `${directory} must go`);
  } finally {
    rmSync(target, { recursive: true, force: true });
  }
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
    const job = ({ record, mutate }: { record: boolean; mutate?: (work: string) => void }) => {
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
      const built = new Set<string>();
      for (const line of output.split("\n").filter((line) => line.startsWith("{"))) {
        const message = JSON.parse(line) as {
          reason?: string;
          fresh?: boolean;
          target?: { name: string };
        };
        if (message.reason === "compiler-artifact" && !message.fresh && message.target)
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

await test(
  "Cargo relinks pruned binaries without recompiling fresh units",
  { skip: cargo.status !== 0 && "cargo is unavailable" },
  () => {
    const root = mkdtempSync(join(tmpdir(), "vize-prune-cargo-"));
    try {
      writeFileSync(
        join(root, "Cargo.toml"),
        '[package]\nname = "app"\nversion = "0.1.0"\nedition = "2021"\n',
      );
      mkdirSync(join(root, "src"));
      writeFileSync(join(root, "src/main.rs"), "fn main() {}\n");
      const build = () =>
        execFileSync("cargo", ["build", "--message-format=json"], {
          cwd: root,
          encoding: "utf8",
          stdio: ["ignore", "pipe", "ignore"],
        })
          .split("\n")
          .filter((line) => line.startsWith("{"))
          .map((line) => JSON.parse(line) as { reason?: string; fresh?: boolean })
          .filter(({ reason }) => reason === "compiler-artifact");
      build();
      writeFileSync(join(root, "target/debug/app.differential-build.json"), "{}");
      pruneRunOutputs(join(root, "target"));
      assert.ok(!existsSync(join(root, "target/debug/app")));
      assert.ok(!existsSync(join(root, "target/debug/app.differential-build.json")));
      assert.ok(build().every(({ fresh }) => fresh === true));
      assert.ok(existsSync(join(root, "target/debug/app")), "Cargo relinks the binary");
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  },
);
