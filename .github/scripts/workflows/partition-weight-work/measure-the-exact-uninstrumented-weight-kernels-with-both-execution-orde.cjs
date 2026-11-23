const fs = require("node:fs"),
  cp = require("node:child_process"),
  crypto = require("node:crypto");
const root = process.env.RUNNER_TEMP + "/partition-weight",
  rows = fs
    .readFileSync(root + "/bench-cargo.jsonl", "utf8")
    .trim()
    .split("\n")
    .map((x) => JSON.parse(x));
const a = rows.filter(
  (x) => x.reason === "compiler-artifact" && x.target?.name === "partition_weight" && x.executable,
);
if (
  a.length !== 1 ||
  a[0].fresh ||
  rows.some(
    (x) =>
      x.reason === "compiler-artifact" &&
      x.target?.name === "vize_carton" &&
      x.features?.includes("profiling"),
  )
)
  throw Error("fresh normal carrier required");
const exe = a[0].executable;
fs.writeFileSync(
  root + "/bench-binary.json",
  JSON.stringify({
    source: cp.execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
    artifact: a[0],
    binarySha256: crypto.createHash("sha256").update(fs.readFileSync(exe)).digest("hex"),
    bytes: fs.statSync(exe).size,
    compilerEnvironment: {
      CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS,
      CARGO_INCREMENTAL: process.env.CARGO_INCREMENTAL,
      CARGO_PROFILE_CI_OPT_DEBUG: process.env.CARGO_PROFILE_CI_OPT_DEBUG,
      CARGO_PROFILE_CI_OPT_STRIP: process.env.CARGO_PROFILE_CI_OPT_STRIP,
      RUSTFLAGS: process.env.RUSTFLAGS ?? null,
      CARGO_ENCODED_RUSTFLAGS: process.env.CARGO_ENCODED_RUSTFLAGS ?? null,
      RUSTC_WRAPPER: process.env.RUSTC_WRAPPER ?? null,
      RUSTC_WORKSPACE_WRAPPER: process.env.RUSTC_WORKSPACE_WRAPPER ?? null,
      RUSTUP_TOOLCHAIN: process.env.RUSTUP_TOOLCHAIN ?? null,
    },
    normal: true,
    hardwareCounters: null,
    wholePipelineTiming: false,
  }),
);
fs.appendFileSync(process.env.GITHUB_ENV, "PARTITION_WEIGHT_BENCH=" + exe + "\n");
const affinity = /Cpus_allowed_list:\s*([^\n]+)/u.exec(
  fs.readFileSync("/proc/self/status", "utf8"),
)?.[1];
if (!affinity) throw Error("CPU affinity unavailable");
const first = affinity.trim().split(/[,-]/u)[0];
if (!first || !/^\d+$/u.test(first)) throw Error("bad CPU affinity");
fs.appendFileSync(process.env.GITHUB_ENV, "PARTITION_WEIGHT_CPU=" + first + "\n");
fs.writeFileSync(
  root + "/bench-machine.json",
  JSON.stringify({
    uname: cp.execFileSync("uname", ["-a"], { encoding: "utf8" }),
    lscpu: cp.execFileSync("lscpu", [], { encoding: "utf8" }),
    allowed: affinity,
    pinned: first,
    clock: "Criterion wall measurement",
    setup: "component-map clone excluded",
    outputDrop: "excluded with BatchSize::PerIteration",
  }),
);
