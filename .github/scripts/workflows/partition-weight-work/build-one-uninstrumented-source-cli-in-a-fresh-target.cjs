const fs = require("node:fs"),
  crypto = require("node:crypto");
const rows = fs
  .readFileSync(process.env.RUNNER_TEMP + "/partition-weight/cargo.jsonl", "utf8")
  .trim()
  .split("\n")
  .map((x) => JSON.parse(x));
const a = rows.filter(
  (x) => x.reason === "compiler-artifact" && x.target?.name === "vize" && x.executable,
);
if (a.length !== 1 || a[0].fresh) throw Error("fresh exact CLI required");
const p = a[0].executable;
fs.writeFileSync(
  process.env.RUNNER_TEMP + "/partition-weight/binary.json",
  JSON.stringify({
    source: fs
      .readFileSync(process.env.RUNNER_TEMP + "/partition-weight/source.sha", "utf8")
      .trim(),
    artifact: a[0],
    bytes: fs.statSync(p).size,
    sha256: crypto.createHash("sha256").update(fs.readFileSync(p)).digest("hex"),
    profile: "ci-opt",
    targetDir: process.env.CARGO_TARGET_DIR,
    targetStartedEmpty: true,
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
    allocation: false,
    timingUsable: false,
  }),
);
