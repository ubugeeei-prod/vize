// Synthetic source-law receipts for the verifier. No Cargo, benchmark or native
// compiler is executed, and these packets provide no performance qualification.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import {
  memorySuites,
  loadMemoryAuthority,
  captureHarnessSource,
  bindHarnessSource,
  sha256,
  MEMORY_ENVIRONMENT,
} from "../../../tools/benchmarks/scripts/level-memory-lib.mjs";
import { parseTomlLite } from "../../../tools/support/compat/davinci/toml-lite.mjs";
const repo = fileURLToPath(new URL("../../..", import.meta.url));
const write = (file, value) => {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
};
function files(root, directory = root) {
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      if (entry.name === "target") return [];
      const file = path.join(directory, entry.name);
      return entry.isDirectory() ? files(root, file) : [path.relative(root, file)];
    })
    .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
}
function listing(root) {
  return files(root)
    .map((file) => {
      const b = fs.readFileSync(path.join(root, file));
      return `100644 blob ${createHash("sha1").update(`blob ${b.length}\0`).update(b).digest("hex")}\t${file}\n`;
    })
    .join("");
}
function sourcePath(root, suite) {
  const directory =
    suite.pkg === "davinci_harness"
      ? "tools/benchmarks/crates"
      : fs.existsSync(path.join(repo, "davinci", suite.pkg))
        ? "davinci"
        : "crates";
  return path.join(root, directory, suite.pkg, "benches", `${suite.bench}.rs`);
}
export function sourceJudgeFixture(t) {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "source-harness-law-")));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const out = path.join(directory, "evidence");
  fs.mkdirSync(out);
  const tool = path.join(directory, "synthetic-tool");
  fs.writeFileSync(tool, "synthetic process image; never executed\n");
  const image = {
    requested: tool,
    path: tool,
    resolved: tool,
    sha256: sha256(fs.readFileSync(tool)),
  };
  function processRecord(
    dir,
    cwd,
    stdout = "",
    args = [],
    environment = {},
    executable = tool,
    stderr = "",
    resolved = tool,
  ) {
    fs.mkdirSync(dir, { recursive: true });
    const a = Buffer.from(stdout),
      b = Buffer.from(stderr);
    const processImage = {
      ...image,
      requested: executable,
      resolved,
      sha256: sha256(fs.readFileSync(resolved)),
    };
    fs.writeFileSync(path.join(dir, "stdout.bin"), a);
    fs.writeFileSync(path.join(dir, "stderr.bin"), b);
    write(path.join(dir, "process.json"), {
      executable,
      args,
      cwd,
      recipeEnvironment: environment,
      imageBefore: processImage,
      imageAfter: processImage,
      started: "synthetic",
      finished: "synthetic",
      status: 0,
      signal: null,
      pid: 1,
      error: null,
      stdout: { bytes: a.length, sha256: sha256(a) },
      stderr: { bytes: b.length, sha256: sha256(b) },
    });
  }
  const suites = memorySuites(repo);
  const arms = {};
  const packageSection = fs
    .readFileSync(path.join(repo, "Cargo.toml"), "utf8")
    .split("[workspace.package]\n")[1]
    .split("\n[")[0];
  const originalVersion = /^version = "([^"]+)"$/m.exec(packageSection)[1];
  for (const [name, version] of [
    ["base", "0.441.0"],
    ["head", "0.442.0"],
  ]) {
    const root = path.join(directory, name);
    fs.mkdirSync(root);
    for (const prefix of ["tools/benchmarks/crates/davinci_harness", "tests/davinci_test_support"])
      fs.cpSync(path.join(repo, prefix), path.join(root, prefix), { recursive: true });
    for (const suite of suites) {
      const dest = sourcePath(root, suite);
      fs.mkdirSync(path.dirname(dest), { recursive: true });
      fs.copyFileSync(sourcePath(repo, suite), dest);
    }
    for (const file of [
      "Cargo.toml",
      "Cargo.lock",
      "rust-toolchain.toml",
      "tools/benchmarks/scripts/instruction-counts-suites.mjs",
      "docs/davinci/plan/budgets.toml",
      "docs/davinci/plan/instruction-budgets.toml",
      "docs/davinci/plan/formatter-instruction-registry.toml",
      "docs/davinci/plan/formatter-instruction-budgets.toml",
    ]) {
      const dest = path.join(root, file);
      fs.mkdirSync(path.dirname(dest), { recursive: true });
      const body = fs.readFileSync(path.join(repo, file));
      fs.writeFileSync(
        dest,
        ["Cargo.toml", "Cargo.lock"].includes(file)
          ? body
              .toString()
              .replaceAll(`"${originalVersion}"`, `"${version}"`)
              .replaceAll(`"=${originalVersion}"`, `"=${version}"`)
          : body,
      );
    }
    const build = path.join(out, name, "build"),
      provenance = path.join(out, name, "provenance");
    const manifest = path.join(root, "tools/benchmarks/crates/davinci_harness/Cargo.toml");
    const library = path.join(root, "tools/benchmarks/crates/davinci_harness/src/lib.rs");
    const packageID = `path+file://${path.dirname(manifest)}#${version}`;
    const metadata = {
      workspace_root: root,
      packages: [{ name: "davinci_harness", id: packageID, version, manifest_path: manifest }],
    };
    const artifacts = [
      {
        reason: "compiler-artifact",
        package_id: packageID,
        manifest_path: manifest,
        target: { kind: ["lib"], src_path: library },
      },
    ];
    arms[name] = { root, version, build, provenance, manifest, library, metadata, artifacts };
  }
  const measurement = {
    source: { base: "1".repeat(40), head: "2".repeat(40) },
    source_harness: {},
    measurements: {},
    observations_complete: true,
    qualified: false,
    gates: [],
  };
  function refresh(name) {
    const arm = arms[name];
    const { root, version, build, provenance, metadata, artifacts } = arm;
    const tree = listing(root);
    processRecord(
      path.join(provenance, "head"),
      root,
      `${measurement.source[name]}\n`,
      ["rev-parse", "HEAD"],
      {},
      "git",
    );
    processRecord(
      path.join(provenance, "status-before"),
      root,
      "",
      ["status", "--porcelain", "--untracked-files=all"],
      {},
      "git",
    );
    processRecord(
      path.join(provenance, "whole-tree"),
      root,
      tree,
      ["ls-tree", "-r", "--full-tree", "HEAD"],
      {},
      "git",
    );
    const pin = captureHarnessSource(root, tree);
    const witness = bindHarnessSource(root, metadata, artifacts, pin);
    write(path.join(build, "harness-source-before.json"), pin);
    write(path.join(build, "harness-source.json"), witness);
    measurement.source_harness[name] = witness;
    processRecord(
      path.join(build, "resolve-cargo"),
      root,
      `${tool}\n`,
      ["which", "--toolchain", "1.99.0", "cargo"],
      {},
      "rustup",
    );
    processRecord(
      path.join(build, "resolve-rustc"),
      root,
      `${tool}\n`,
      ["which", "--toolchain", "1.99.0", "rustc"],
      {},
      "rustup",
    );
    processRecord(path.join(build, "rustc"), root, "rustc 1.99.0 synthetic source-law receipt\n", [
      "-Vv",
    ]);
    const environment = {
      RUSTC: tool,
      RUSTUP_TOOLCHAIN: "1.99.0",
      RUSTFLAGS: "-C target-cpu=x86-64",
      CARGO_INCREMENTAL: "0",
      RUSTC_WRAPPER: "",
      RUSTC_WORKSPACE_WRAPPER: "",
    };
    processRecord(
      path.join(build, "cargo-metadata"),
      root,
      JSON.stringify(metadata),
      ["metadata", "--locked", "--no-deps", "--format-version", "1"],
      environment,
    );
    const target = path.join(arms.head.root, "target", `level-memory-${name}`);
    const binaries = suites.map((suite, index) => {
      const source = sourcePath(root, suite);
      const executable = path.join(target, `synthetic-image-${index}`);
      fs.mkdirSync(target, { recursive: true });
      fs.writeFileSync(executable, `synthetic benchmark image ${index}\n`);
      return {
        ...suite,
        source,
        executable,
        fresh: false,
        source_sha256: sha256(fs.readFileSync(source)),
        sha256: sha256(fs.readFileSync(executable)),
        harness_source: witness,
      };
    });
    const benchArtifacts = binaries.map((row) => ({
      reason: "compiler-artifact",
      package_id: `path+file://${path.dirname(path.dirname(row.source))}#${version}`,
      manifest_path: path.join(path.dirname(path.dirname(row.source)), "Cargo.toml"),
      target: { kind: ["bench"], name: row.bench, src_path: row.source },
      executable: row.executable,
    }));
    const args = [
      "bench",
      "--locked",
      "--no-run",
      "--profile",
      "ci-opt",
      "--features",
      "davinci_harness/instruction-counts",
      "--message-format=json",
      "--target-dir",
      target,
    ];
    for (const pkg of new Set(suites.map((x) => x.pkg))) args.push("-p", pkg);
    for (const bench of new Set(suites.map((x) => x.bench))) args.push("--bench", bench);
    processRecord(
      path.join(build, "cargo"),
      root,
      [...artifacts, ...benchArtifacts].map((x) => JSON.stringify(x)).join("\n") + "\n",
      args,
      environment,
    );
    write(path.join(build, "binaries.json"), binaries);
    arm.binaries = binaries;
    processRecord(
      path.join(out, name, "source-after"),
      root,
      "",
      ["diff", "--name-only", "HEAD"],
      {},
      "git",
    );
  }
  refresh("base");
  refresh("head");
  const authority = loadMemoryAuthority(repo);
  const budgets = parseTomlLite(
    fs.readFileSync(path.join(repo, "docs/davinci/plan/budgets.toml"), "utf8"),
  );
  const levelIds = [...authority.planes.level.registry];
  const suiteIndex = new Map(levelIds.map((id, index) => [id, index % 12]));
  for (const id of authority.planes.formatter.registry) suiteIndex.set(id, 12);
  function save() {
    for (const name of ["base", "head"])
      for (let n = 1; n <= 3; n++)
        for (let index = 0; index < 13; index++) {
          const arm = arms[name],
            directory = path.join(out, name, `run-${n}`, `suite-${index}`);
          const rows = Object.fromEntries(
            Object.entries(measurement.measurements[name][n - 1]).filter(
              ([id]) => suiteIndex.get(id) === index,
            ),
          );
          write(path.join(directory, "rows.json"), rows);
          const probe = path.join(
            path.dirname(out),
            "level-memory-probes",
            `probe-${String(index).padStart(2, "0")}`,
          );
          const stderr =
            "mimalloc: option 'reserve_os_memory': 131072 KiB\n" +
            "mimalloc: option 'purge_delay': -1\n" +
            Object.values(rows)
              .map((row) => `davinci-harness: wrote ${row.file}\n`)
              .join("");
          processRecord(
            path.join(directory, "process"),
            arm.root,
            "",
            ["--bench", "--quick"],
            MEMORY_ENVIRONMENT,
            probe,
            stderr,
            arm.binaries[index].executable,
          );
        }
    write(path.join(out, "measurement.json"), measurement);
  }
  function writeReport(name, repetition, id, changes = {}) {
    const arm = arms[name],
      run = path.join(out, name, `run-${repetition}`);
    const identity =
      authority.planes[authority.planes.level.registry.has(id) ? "level" : "formatter"].budgets
        .instruction[id];
    const original = measurement.measurements[name]?.[repetition - 1]?.[id]?.report;
    const report = {
      bench_id: id,
      fixture: identity.fixture,
      platform: "linux",
      wall_ns: { p50: 1, p95: 1 },
      allocs: budgets.bench[id]?.allocs ?? 0,
      alloc_bytes_peak: budgets.allocation_peak?.[id]?.linux ?? 0,
      rss_peak_bytes: 0,
      harness_version: arm.version,
      ...original,
      ...changes,
    };
    const raw = Buffer.from(JSON.stringify(report) + "\n");
    const rawFile = path.join(run, `suite-${suiteIndex.get(id)}/reports`, `${id}.json`);
    const plane = authority.planes.level.registry.has(id) ? "level" : "formatter";
    for (const file of [
      rawFile,
      path.join(run, "reports", `${id}.json`),
      path.join(run, `${plane}-reports`, `${id}.json`),
    ]) {
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, raw);
    }
    const row = {
      report,
      identity: {
        bench_id: id,
        fixture: identity.fixture,
        fixture_sha256: identity.fixture_sha256,
        window: identity.window,
      },
      raw_sha256: sha256(raw),
      file: path.join(arm.root, "tools/benchmarks/results/davinci", `${id}.json`),
      raw_file: rawFile,
    };
    measurement.measurements[name][repetition - 1][id] = row;
  }
  for (const name of ["base", "head"]) {
    measurement.measurements[name] = [{}, {}, {}];
    for (let n = 1; n <= 3; n++) for (const id of authority.registry) writeReport(name, n, id);
  }
  save();
  const pair = path.join(out, "source-harness-pairs/head-1.json");
  const carrier = {
    schema_version: 1,
    measurement: path.join(out, "measurement.json"),
    baseline: { arm: "base", repetition: 1 },
    current: { arm: "head", repetition: 1 },
  };
  write(pair, carrier);
  const args = [
    "--budgets",
    path.join(arms.head.root, "docs/davinci/plan/budgets.toml"),
    "--baseline",
    path.join(out, "base/run-1/level-reports"),
    "--results",
    path.join(out, "head/run-1/level-reports"),
  ];
  function rewriteStream(directory, stream, body) {
    const raw = Buffer.from(body),
      file = path.join(directory, "process.json");
    const receipt = JSON.parse(fs.readFileSync(file, "utf8"));
    fs.writeFileSync(path.join(directory, `${stream}.bin`), raw);
    receipt[stream] = { bytes: raw.length, sha256: sha256(raw) };
    write(file, receipt);
  }
  return {
    out,
    arms,
    measurement,
    pair,
    carrier,
    args,
    save,
    refresh,
    writeReport,
    write,
    rewriteStream,
    authority,
    budgets,
  };
}
