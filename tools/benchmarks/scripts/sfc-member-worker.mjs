import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { memberCases, controlCases, profileKeys } from "./sfc-member-fixtures.mjs";

const [root, output, identityFile] = process.argv.slice(2);
const identity = JSON.parse(fs.readFileSync(identityFile, "utf8"));
assert.equal(fs.realpathSync(root), identity.root);
assert.equal(process.env.NAPI_RS_NATIVE_LIBRARY_PATH, undefined);
assert.equal(process.env.NAPI_RS_FORCE_WASI, undefined);
const nativeDir = path.join(root, "npm/native");
const require = createRequire(path.join(nativeDir, "index.js"));
const binding = require(path.join(nativeDir, "index.js"));
assert.equal(typeof binding.analyzeSfc, "function");
assert.equal(binding.getNativeBuildToken(), identity.token);
const images = Object.keys(require.cache).filter(
  (file) => file.endsWith(".node") && file.startsWith(nativeDir + path.sep),
);
assert.deepEqual(images, [identity.binary]);
const hash = (file) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
assert.equal(hash(images[0]), identity.binarySha256);
const report = {
  identity,
  loadedToken: binding.getNativeBuildToken(),
  loadedBinary: images[0],
  profile: "ci-opt; opt3/thinLTO/cgu16",
  samples: 30,
  cpuWarmups: 3,
  rows: [],
  controls: [],
};
function validated(sample, metadata, cacheHit) {
  const { profile, ...data } = sample;
  assert.deepEqual(data, metadata);
  assert.deepEqual(Object.keys(profile).sort(), profileKeys);
  assert.equal(profile.cacheHit, cacheHit);
  for (const key of profileKeys.filter((key) => key !== "cacheHit"))
    assert.ok(Number.isSafeInteger(profile[key]) && profile[key] >= 0, key);
  for (const key of ["descriptorParseCalls", "analyzeCalls", "serializeCalls"])
    assert.equal(profile[key], cacheHit ? 0 : 1, key);
  assert.equal(profile.defaultParseCalls, 0);
  if (cacheHit)
    for (const key of profileKeys.filter((key) => key !== "cacheHit"))
      assert.equal(profile[key], 0, key);
  return profile;
}
for (const vector of memberCases()) {
  const options = { filename: vector.id + ".vue", cache: false, profile: false };
  const metadata = binding.analyzeSfc(vector.source, options);
  assert.deepEqual(metadata, vector.expected);
  assert.equal(Object.hasOwn(metadata, "profile"), false);
  for (let i = 0; i < report.cpuWarmups; i++)
    binding.analyzeSfc(vector.source, { ...options, profile: true });
  let start = performance.now();
  const cold = Array.from({ length: report.samples }, () =>
    binding.analyzeSfc(vector.source, { ...options, profile: true }),
  );
  const coldWallMs = performance.now() - start;
  binding.analyzeSfc(vector.source, { ...options, cache: true });
  start = performance.now();
  const warm = Array.from({ length: report.samples }, () =>
    binding.analyzeSfc(vector.source, { ...options, cache: true, profile: true }),
  );
  const warmWallMs = performance.now() - start;
  const coldProfiles = cold.map((sample) => validated(sample, metadata, false));
  const warmProfiles = warm.map((sample) => validated(sample, metadata, true));
  for (const profile of coldProfiles)
    assert.equal(profile.literalMemberParseCalls, coldProfiles[0].literalMemberParseCalls);
  report.rows.push({
    id: vector.id,
    count: vector.count,
    shape: vector.shape,
    source: vector.source,
    sourceSha256: createHash("sha256").update(vector.source).digest("hex"),
    falseProfile: metadata,
    falseProfileBytes: JSON.stringify(metadata),
    coldWallMs,
    warmWallMs,
    coldProfiles,
    warmProfiles,
  });
}
for (const vector of controlCases()) {
  const value = binding.analyzeSfc(vector.source, {
    filename: vector.id + ".vue",
    cache: false,
    profile: false,
  });
  assert.deepEqual(value, vector.expected);
  report.controls.push({
    id: vector.id,
    source: vector.source,
    falseProfile: value,
    falseProfileBytes: JSON.stringify(value),
  });
}
assert.equal(hash(images[0]), identity.binarySha256, "native image changed during measurement");
fs.writeFileSync(output, JSON.stringify(report, null, 2) + "\n");
