import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { sourceProvenance, validateProductSource } from "./sfc-parse-attributes-source.mjs";

const sourceRoot = path.resolve(process.argv[2]);
const role = process.argv[3];
assert.ok(["base", "head"].includes(role));
const directory = path.resolve(process.argv[4]);
await fs.mkdir(directory, { recursive: true });
const source = execFileSync("git", ["-C", sourceRoot, "rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();
assert.equal(source, process.env.VIZE_SFC_ATTRS_SOURCE_SHA);
const observerActor = execFileSync(
  "git",
  ["-C", fileURLToPath(new URL("../../", import.meta.url)), "rev-parse", "HEAD"],
  { encoding: "utf8" },
).trim();
const provenance = sourceProvenance(observerActor);
validateProductSource(source, role, observerActor);
const nativeRoot = path.join(sourceRoot, "npm/native");
const require = createRequire(path.join(nativeRoot, "package.json"));
const binding = require("./index.js");
const { getBindingIdentity } = require("./native-binding-identity.js");
const identity = getBindingIdentity(binding);
assert.equal(identity.kind, "native");
assert.match(identity.token, /^[0-9a-f]{32}$/);
assert.equal(binding.getNativeBuildToken(), identity.token);
const binaries = (await fs.readdir(nativeRoot)).filter((name) => name.endsWith(".node"));
assert.equal(binaries.length, 1);
const filename = binaries[0];
const binary = await fs.realpath(path.join(nativeRoot, filename));
const hash = (value) => createHash("sha256").update(value).digest("hex");
assert.ok(
  Object.keys(require.cache).includes(binary),
  "the selected image must be the actual locally built addon",
);
const receipt = {
  schema: "vize.sfc-parse-attributes-source.v1",
  role,
  source,
  provenance,
  triggerSha: process.env.GITHUB_SHA,
  runId: process.env.GITHUB_RUN_ID,
  runAttempt: process.env.GITHUB_RUN_ATTEMPT,
  node: process.version,
  platform: process.platform,
  arch: process.arch,
  identity,
  binary: {
    filename,
    bytes: (await fs.stat(binary)).size,
    sha256: hash(await fs.readFile(binary)),
  },
};
await fs.writeFile(path.join(directory, "image.json"), JSON.stringify(receipt, null, 2) + "\n");
const cases = JSON.parse(
  await fs.readFile(
    new URL("../fixtures/sfc/parsed-attributes/cases.json", import.meta.url),
    "utf8",
  ),
);
function capture(operation) {
  try {
    return { kind: "return", json: JSON.stringify(operation()) };
  } catch (error) {
    return { kind: "throw", code: error.code, message: error.message, stack: error.stack };
  }
}
const packets = cases.map((fixture) => {
  const filename = `parsed-attributes/${fixture.id}.vue`;
  return {
    id: fixture.id,
    omitted: capture(() => binding.parseSfc(fixture.source)),
    filename: capture(() => binding.parseSfc(fixture.source, { filename })),
    disabled: capture(() => binding.parseSfc(fixture.source, { filename, includeAttrs: false })),
    enabled: capture(() => binding.parseSfc(fixture.source, { filename, includeAttrs: true })),
    compile: capture(() =>
      binding.compileSfc(fixture.source, { filename, sourceMap: true, ssr: true, vapor: false }),
    ),
  };
});
await fs.writeFile(
  path.join(directory, "packets.json"),
  JSON.stringify({ role, source, receipt, packets }, null, 2) + "\n",
);
const benchmark = cases.find(({ id }) => id === "all-blocks");
const samples = [];
for (let repeat = 0; repeat < 40; repeat++) {
  for (const includeAttrs of repeat % 2 ? [true, false] : [false, true]) {
    const start = process.hrtime.bigint();
    let value;
    let failure;
    try {
      value = binding.parseSfc(benchmark.source, { filename: "profile.vue", includeAttrs });
    } catch (error) {
      failure = error;
    }
    const elapsedNs = String(process.hrtime.bigint() - start);
    const result = capture(() => {
      if (failure) throw failure;
      return value;
    });
    samples.push({ includeAttrs, elapsedNs, result });
  }
}
await fs.writeFile(
  path.join(directory, "profile.json"),
  JSON.stringify(
    {
      source,
      node: process.version,
      platform: process.platform,
      arch: process.arch,
      scope: "native parse and bridge only; packet serialization is outside the clock",
      sourceBytes: Buffer.byteLength(benchmark.source),
      samples,
    },
    null,
    2,
  ) + "\n",
);
await fs.copyFile(
  path.join(nativeRoot, "index.d.ts"),
  path.join(directory, "generated-index.d.ts"),
);
if (role === "head") {
  const fixture = fileURLToPath(
    new URL("../fixtures/sfc/parsed-attributes/types.mts", import.meta.url),
  );
  const target = path.join(sourceRoot, "tests/fixtures/sfc/parsed-attributes/types.mts");
  if (fixture !== target) {
    await fs.mkdir(path.dirname(target), { recursive: true });
    await fs.copyFile(fixture, target);
  }
  const compiler = path.join(sourceRoot, "tests/node_modules/typescript/bin/tsc");
  execFileSync(
    process.execPath,
    [
      compiler,
      "--ignoreConfig",
      "--noEmit",
      "--strict",
      "--target",
      "ES2022",
      "--module",
      "NodeNext",
      "--moduleResolution",
      "NodeNext",
      target,
    ],
    { cwd: sourceRoot, stdio: "inherit" },
  );
}
