import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve, relative } from "node:path";

const directory = resolve(process.argv[2]);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
for (const [name, count] of Object.entries({
  "route-owners": 3,
  "route-refusal": 2,
  "ordinary-fallback": 1,
  "config-refusal": 1,
  "batch-fixture": 1,
  "generic-hosts": 3,
})) {
  assert.equal(readFileSync(join(directory, `${name}.status`), "utf8"), "0\n", name);
  const stdout = readFileSync(join(directory, `${name}.stdout`), "utf8");
  assert(stdout.includes(`test result: ok. ${count} passed; 0 failed; 0 ignored;`), name);
}
const packetRoot = resolve("target/vize-tests/content-mapper-quoted-generic-parameters");
const files = [];
function walk(root) {
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) walk(path);
    else if (entry.isFile() && entry.name.endsWith(".json")) files.push(path);
  }
}
walk(packetRoot);
const expected = {
  "vize.quoted-generic-parameters-host-observation": 32,
  "vize.quoted-generic-syntax-host-observation": 48,
  "vize.generic-body-error-host-observation": 64,
};
const counts = Object.fromEntries(Object.keys(expected).map((key) => [key, 0]));
const packets = [];
for (const path of files) {
  const bytes = readFileSync(path);
  const packet = JSON.parse(bytes.toString("utf8"));
  if (!Object.hasOwn(expected, packet.schema)) continue;
  assert.equal(packet.version, 1, path);
  assert(Number.isInteger(packet.status), path);
  assert.deepEqual(packet.stderr, [], `whole stderr must be empty: ${path}`);
  const stem = path.slice(0, -".json".length);
  assert.deepEqual(readFileSync(`${stem}.stdout.raw`), Buffer.from(packet.stdout), path);
  assert.deepEqual(readFileSync(`${stem}.stderr.raw`), Buffer.from(packet.stderr), path);
  assert(Array.isArray(packet.inputs) && packet.inputs.length > 0, path);
  counts[packet.schema]++;
  packets.push({ path: relative(packetRoot, path), sha256: hash(bytes), host: packet.host, status: packet.status });
}
assert.deepEqual(counts, expected);
assert.equal(packets.length, 144);
writeFileSync(join(directory, "generic-route-verification.json"), JSON.stringify({
  source: readFileSync(join(directory, "source.sha"), "utf8").trim(),
  counts, packetCount: packets.length, wholeStderrEmpty: true, packets,
}, null, 2) + "\n");
