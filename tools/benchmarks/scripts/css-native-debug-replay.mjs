import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { Readable, Transform } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";
import { streamedDigest, validateDebugIdentity } from "./css-native-debug-build.mjs";
import { createRequire } from "node:module";

export const debugArchive = {
  artifact: 12615717983,
  run: 40237508866,
  harnessSource: "6279583255817ceb7e98df20b75b2b80a8d992bb",
  bytes: 621786148,
  sha256: "35ba808527fb513ddeb6c8cee96a7470fd4a46597196cb5ee68eb549b83402b7",
};
export function validateDebugArchive(meta) {
  assert.equal(meta.id, debugArchive.artifact);
  assert.equal(meta.workflow_run.id, debugArchive.run);
  assert.equal(meta.workflow_run.head_sha, debugArchive.harnessSource);
  assert.equal(meta.size_in_bytes, debugArchive.bytes);
  assert.equal(meta.digest, `sha256:${debugArchive.sha256}`);
  assert.equal(meta.expired, false);
}
const script = fileURLToPath(import.meta.url);
const read = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
async function run(destination, workspace) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.version, "v24.14.0");
  destination = path.resolve(destination);
  assert.equal(fs.existsSync(destination), false);
  fs.mkdirSync(destination, { recursive: true });
  const record = {
    schema: "vize.css.qualified-debug-producer-replay",
    version: 1,
    status: "attempt",
    authority: debugArchive,
    note: "The original sampler failed its first partial ACK read. Its separately completed debug producers and whole oracles are reused; no product rebuild, original partial-sample success or new absent-target claim.",
  };
  const retain = () => write(path.join(destination, "receipt.json"), record);
  retain();
  try {
    const headers = {
      Authorization: `Bearer ${process.env.GH_TOKEN}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
    };
    assert.ok(process.env.GH_TOKEN);
    const endpoint = `https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/${debugArchive.artifact}`;
    const metadata = await fetch(endpoint, { headers, signal: AbortSignal.timeout(30000) });
    assert.equal(metadata.status, 200);
    const meta = await metadata.json();
    validateDebugArchive(meta);
    write(path.join(destination, "source-artifact-metadata.json"), meta);
    const redirect = await fetch(endpoint + "/zip", {
      headers,
      redirect: "manual",
      signal: AbortSignal.timeout(30000),
    });
    assert.equal(redirect.status, 302);
    const location = redirect.headers.get("location");
    assert.ok(location?.startsWith("https://"));
    const response = await fetch(location, { signal: AbortSignal.timeout(300000) });
    assert.equal(response.status, 200);
    const temporary = fs.mkdtempSync(path.join(process.env.RUNNER_TEMP, "css-debug-replay-")),
      archive = path.join(temporary, "producer.zip"),
      origin = path.join(temporary, "original");
    const hash = createHash("sha256");
    let count = 0;
    await pipeline(
      Readable.fromWeb(response.body),
      new Transform({
        transform(chunk, _, done) {
          count += chunk.length;
          hash.update(chunk);
          done(null, chunk);
        },
      }),
      fs.createWriteStream(archive, { flags: "wx" }),
    );
    assert.equal(count, debugArchive.bytes);
    assert.equal(hash.digest("hex"), debugArchive.sha256);
    record.archiveVerified = true;
    retain();
    execFileSync("unzip", ["-q", archive, "-d", origin]);
    const originalConfigFile = path.join(origin, "debug-producers/inputs.json"),
      original = read(originalConfigFile),
      producerReceiptFile = path.join(origin, "debug-producers/receipt.json"),
      producerReceipt = read(producerReceiptFile);
    assert.equal(producerReceipt.status, "complete");
    assert.equal(original.harnessSource, debugArchive.harnessSource);
    const reference = read(path.join(origin, "reference/receipt.json"));
    assert.equal(reference.status, "prepared-original-inputs");
    assert.equal(reference.node.sha256, streamedDigest(process.execPath));
    const images = {};
    for (const side of ["base", "head"]) {
      const producer = original.images[side].producer,
        sourceRoot = fs.realpathSync(path.join(workspace, side));
      assert.equal(
        execFileSync("git", ["rev-parse", "HEAD"], { cwd: sourceRoot, encoding: "utf8" }).trim(),
        producer.source,
      );
      for (const receipt of producer.compilerReceipts)
        assert.ok(
          receipt.manifestDir.startsWith(sourceRoot + path.sep),
          "actual original DWARF source tree must exist at its original path",
        );
      const transportBinary = path.join(
          origin,
          `debug-producers/${side}-image`,
          path.basename(producer.binary),
        ),
        native = createRequire(import.meta.url)(transportBinary);
      validateDebugIdentity(
        side,
        producer,
        streamedDigest(transportBinary),
        native.getNativeBuildToken(),
      );
      images[side] = { producer, transportBinary };
      const qualified = read(path.join(origin, `debug-producers/${side}-whole-native.json`));
      assert.equal(qualified.status, "complete");
      assert.equal(qualified.calls.length, 84);
      write(path.join(destination, `${side}-qualified-whole-native.json`), qualified);
    }
    for (const name of ["observation.json", "v-bind-observation.json"]) {
      const packet = read(path.join(origin, "debug-producers/head-native-oracles", name));
      assert.deepEqual(packet.failures, []);
      assert.equal(packet.calls.length, name === "observation.json" ? 84 : 40);
      write(path.join(destination, `head-${name}`), packet);
    }
    const recorder = fs.readFileSync(
      path.join(origin, "debug-producers/head-42-recorder.log"),
      "utf8",
    );
    assert.match(recorder, /test result: ok\. 1 passed/u);
    fs.writeFileSync(path.join(destination, "head-42-recorder.log"), recorder);
    const harnessSource = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
    assert.equal(harnessSource, process.env.GITHUB_SHA);
    const config = {
      kind: "debug-source",
      harnessSource,
      rows: original.rows,
      images,
      replayAuthority: {
        ...debugArchive,
        originalConfigSha256: streamedDigest(originalConfigFile),
        originalProducerReceiptSha256: streamedDigest(producerReceiptFile),
        noNewProductBuild: true,
      },
    };
    write(path.join(destination, "inputs.json"), config);
    record.configSha256 = streamedDigest(path.join(destination, "inputs.json"));
    record.nodeSha256 = streamedDigest(process.execPath);
    record.status = "complete";
    retain();
  } catch (error) {
    record.status = "failed";
    record.failure = { name: error.name, message: error.message };
    retain();
    throw error;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === script)
  await run(...process.argv.slice(2));
