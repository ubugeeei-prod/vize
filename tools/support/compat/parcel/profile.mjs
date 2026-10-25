import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";
import { analyseProfile } from "./profile-analysis.mjs";

const [destination, profiles] = process.argv.slice(2).map((entry) => path.resolve(entry));
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const receipt = JSON.parse(
  await fs.readFile(path.join(destination, "receipt.json"), "utf8").catch(() => "null"),
);
const output = path.join(destination, "cpu-profiles");
await fs.mkdir(output);
const results = [];
const inputs = [];
for (const filename of (await fs.readdir(profiles))
  .filter((name) => name.endsWith(".cpuprofile"))
  .sort()) {
  const bytes = await fs.readFile(path.join(profiles, filename));
  await fs.writeFile(path.join(output, filename), bytes, { flag: "wx" });
  inputs.push({ filename, sha256: sha(bytes), bytes: bytes.length });
}
// Capture the entire producer directory before validating any individual file.
// A malformed later profile must not discard the remaining original profiles.
await fs.writeFile(
  path.join(destination, "cpu-producer-manifest.json"),
  JSON.stringify({ source: receipt?.source, files: inputs }, null, 2) + "\n",
);
for (const input of inputs) {
  const { filename } = input;
  const bytes = await fs.readFile(path.join(output, filename));
  assert.equal(sha(bytes), input.sha256);
  const profile = JSON.parse(bytes);
  const analysis = analyseProfile(profile);
  results.push({
    filename,
    sha256: input.sha256,
    bytes: input.bytes,
    recordingSpanMicros: profile.endTime - profile.startTime,
    ...analysis,
  });
}
assert.ok(results.length, "actual Node CPU profiles must be retained");
await fs.writeFile(
  path.join(destination, "source-profile.json"),
  JSON.stringify(
    {
      schema: "vize.parcel.node-profile.v1",
      source: receipt?.source,
      complete: receipt?.complete === true,
      sourceQualification: receipt?.sourceQualification === true,
      note: "JavaScript self samples per process/thread. Raw signed deltas are retained; derived intervals are withheld for non-monotonic profiles. No timestamp normalization, cross-process totals, Rust attribution, RSS or speedup claim.",
      profiles: results,
    },
    null,
    2,
  ) + "\n",
);

const entries = [];
async function visit(directory) {
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name === "evidence-manifest.json") continue;
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) await visit(filename);
    else if (entry.isFile()) {
      const bytes = await fs.readFile(filename);
      entries.push({
        path: path.relative(destination, filename),
        bytes: bytes.length,
        sha256: sha(bytes),
      });
    }
  }
}
await visit(destination);
await fs.writeFile(
  path.join(destination, "evidence-manifest.json"),
  JSON.stringify(
    {
      source: receipt?.source,
      complete: receipt?.complete === true,
      files: entries.sort((left, right) =>
        left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
      ),
    },
    null,
    2,
  ) + "\n",
);
