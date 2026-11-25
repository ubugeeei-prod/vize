const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const report = (name) =>
  JSON.parse(
    readFileSync(`${process.env.RUNNER_TEMP}/prettier-${name}-consumer/report.json`, "utf8"),
  );
const current = report("source");
const minimum = report("node22");
assert.equal(current.node, "v24.14.0");
assert.equal(minimum.node, "v22.0.0");
const identity = (value) => ({
  tarball: value.adapterTarballSha256,
  source: {
    head: value.sourcePreparation.head,
    tree: value.sourcePreparation.tree,
    workingDiff: value.sourcePreparation.workingDiff,
    binding: value.sourcePreparation.binding,
    sha256: value.sourcePreparation.sha256,
  },
  nativePackage: value.nativePackage,
  pluginVersion: value.pluginVersion,
  prettier: value.prettier,
  golden: value.golden,
  art: value.art,
  ignored: value.ignored,
});
assert.deepEqual(identity(minimum), identity(current));
