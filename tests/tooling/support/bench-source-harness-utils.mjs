import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { sha256 } from "../../../tools/benchmarks/scripts/level-memory-lib.mjs";

export const write = (file, value) => {
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
export function listing(root) {
  return files(root)
    .map((file) => {
      const b = fs.readFileSync(path.join(root, file));
      return `100644 blob ${createHash("sha1").update(`blob ${b.length}\0`).update(b).digest("hex")}\t${file}\n`;
    })
    .join("");
}
export function sourcePathFor(repo, root, suite) {
  const directory =
    suite.pkg === "davinci_harness"
      ? "tools/benchmarks/crates"
      : fs.existsSync(path.join(repo, "davinci", suite.pkg))
        ? "davinci"
        : "crates";
  return path.join(root, directory, suite.pkg, "benches", `${suite.bench}.rs`);
}

export function fixtureProcessRecorder(image, tool) {
  return function processRecord(
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
  };
}

export function fixtureReportWriter({ arms, out, authority, budgets, measurement, suiteIndex }) {
  return function writeReport(name, repetition, id, changes = {}) {
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
  };
}
