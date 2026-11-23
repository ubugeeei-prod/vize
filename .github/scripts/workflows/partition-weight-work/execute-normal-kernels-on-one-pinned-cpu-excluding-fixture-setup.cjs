const fs = require("node:fs"),
  path = require("node:path");
const root = process.env.RUNNER_TEMP + "/partition-weight",
  results = [];
function walk(dir) {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(p);
    else if (
      p.includes(path.sep + "new" + path.sep) &&
      ["estimates.json", "sample.json", "benchmark.json"].includes(ent.name)
    )
      results.push({ path: path.relative(root, p), value: JSON.parse(fs.readFileSync(p, "utf8")) });
  }
}
for (const order of ["baseline-first", "candidate-first"]) walk(root + "/criterion-" + order);
if (results.filter((x) => x.path.endsWith("/new/estimates.json")).length !== 16)
  throw Error("incomplete normal timing census");
const build = JSON.parse(fs.readFileSync(root + "/bench-binary.json", "utf8")),
  control = JSON.parse(fs.readFileSync(root + "/kernel-control.json", "utf8"));
if (
  control.candidateSource !== build.source ||
  require("node:crypto")
    .createHash("sha256")
    .update(fs.readFileSync(build.artifact.executable))
    .digest("hex") !== build.binarySha256
)
  throw Error("post-run source/image seal");
const report = {
  scope:
    "normal source-pinned binpacking kernel only; warmed in-process; not full partition/check/cold latency",
  hardwareCounters: null,
  allocationCounts: null,
  source: build.source,
  control,
  results,
};
const body = JSON.stringify(report);
if (Buffer.byteLength(body) > 100 * 1024) throw Error("normal summary exceeds 100KiB");
fs.writeFileSync(root + "/normal-summary.json", body);
