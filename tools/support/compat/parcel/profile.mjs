import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs/promises";
import path from "node:path";

const [destination, profiles] = process.argv.slice(2).map((entry) => path.resolve(entry));
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const receipt = JSON.parse(
  await fs.readFile(path.join(destination, "receipt.json"), "utf8").catch(() => "null"),
);
const output = path.join(destination, "cpu-profiles");
await fs.mkdir(output);
const results = [];
for (const filename of (await fs.readdir(profiles))
  .filter((name) => name.endsWith(".cpuprofile"))
  .sort()) {
  const bytes = await fs.readFile(path.join(profiles, filename));
  await fs.writeFile(path.join(output, filename), bytes, { flag: "wx" });
  const profile = JSON.parse(bytes);
  assert.ok(Array.isArray(profile.nodes) && Array.isArray(profile.samples));
  assert.equal(profile.samples.length, profile.timeDeltas.length);
  const nodes = new Map(profile.nodes.map((node) => [node.id, node]));
  const rows = new Map();
  for (let index = 0; index < profile.samples.length; index++) {
    const node = nodes.get(profile.samples[index]);
    assert.ok(node);
    assert.ok(Number.isFinite(profile.timeDeltas[index]) && profile.timeDeltas[index] >= 0);
    if (!node.callFrame.url.includes("/@vizejs/parcel-transformer/")) continue;
    const row = rows.get(node.id) ?? {
      function: node.callFrame.functionName,
      file: node.callFrame.url,
      line: node.callFrame.lineNumber + 1,
      column: node.callFrame.columnNumber + 1,
      samples: 0,
      sampledTimeMicros: 0,
    };
    row.samples++;
    row.sampledTimeMicros += profile.timeDeltas[index];
    rows.set(node.id, row);
  }
  results.push({
    filename,
    sha256: sha(bytes),
    bytes: bytes.length,
    durationMicros: profile.endTime - profile.startTime,
    samples: profile.samples.length,
    adapter: [...rows.values()].sort(
      (left, right) => right.sampledTimeMicros - left.sampledTimeMicros,
    ),
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
      note: "Sampled JavaScript self time per process/thread; not wall time, RSS, Rust source attribution, or a before/after speedup.",
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
      files: entries.sort((left, right) => left.path.localeCompare(right.path)),
    },
    null,
    2,
  ) + "\n",
);
