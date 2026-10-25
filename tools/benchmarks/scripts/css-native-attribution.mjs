import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { profileVectors } from "./css-source-map-profile.mjs";

export const campaign = {
  run: 40228200747,
  artifact: 12612347385,
  bytes: 70827715,
  digest: "d9e335d06f4905d4247cdfc6a3db70f3346e0a9bbdff92438376a4a8b0527548",
  node: "v24.14.0",
  head: "fcab3f87d3249cccaecc5050d73d2af6ac89d3f8",
  images: {
    base: {
      source: "3a3c8460f966fddbd8e1be4c083d460b24202f90",
      sha256: "747bb8519efd624007c7f55d8f521d0ca9fab1908d4f2119cb696b39405d9ab1",
      token: "b71f3499c0e71fd1b1ee1cada629a2cc",
    },
    head: {
      source: "fcab3f87d3249cccaecc5050d73d2af6ac89d3f8",
      sha256: "6c43c3f1e1809a6620ebadfbfbdb587febdabd0fa38ff8ef8a02414f9858baae",
      token: "c2cae6927b838292d4b0225bd2468d93",
    },
  },
};
const script = fileURLToPath(import.meta.url),
  hash = (bytes) => createHash("sha256").update(bytes).digest("hex"),
  digest = (file) => hash(fs.readFileSync(file));
const write = (file, value) => fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n"),
  read = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
export function validateArtifact(meta) {
  assert.equal(meta.id, campaign.artifact);
  assert.equal(meta.workflow_run.id, campaign.run);
  assert.equal(meta.workflow_run.head_sha, campaign.head);
  assert.equal(meta.digest, `sha256:${campaign.digest}`);
  assert.equal(meta.size_in_bytes, campaign.bytes);
  assert.equal(meta.expired, false);
}
export function validateReplayImage(side, identity, actualHash, actualToken) {
  assert.ok(Object.hasOwn(campaign.images, side));
  const expected = campaign.images[side];
  assert.equal(identity.side, side);
  assert.equal(identity.source, expected.source);
  assert.equal(identity.targetInitiallyEmpty, true);
  assert.equal(identity.binarySha256, expected.sha256);
  assert.equal(actualHash, expected.sha256);
  assert.equal(identity.token, expected.token);
  assert.equal(actualToken, expected.token);
}
export function attributionOrder() {
  return ["base", "head", "head", "base"].flatMap((side, index) =>
    (index % 2 === 0 ? ["authored", "processed"] : ["processed", "authored"]).map((operation) => ({
      index,
      side,
      operation,
    })),
  );
}
function inputsFromOriginal(origin) {
  const receipt = read(path.join(origin, "receipt.json"));
  assert.equal(receipt.node, campaign.node);
  assert.equal(receipt.completed, true);
  assert.equal(receipt.nativeCompleted, true);
  assert.equal(receipt.isolatedOffCompleted, true);
  const rows = {};
  for (const side of ["base", "head"]) {
    const indices = receipt.order.flatMap((value, index) => (value === side ? [index] : []));
    assert.equal(indices.length, 2);
    const raw = indices.map((index) => ({
      native: read(path.join(origin, `${index}-${side}.json`)),
      phase: read(path.join(origin, `${index}-${side}-phases.json`)),
    }));
    for (const value of raw) assert.equal(value.phase.status, "complete");
    rows[side] = profileVectors().map((vector) => {
      const native = raw[0].native.timings.find((row) => row.id === vector.id),
        phase = raw[0].phase.vectors.find((row) => row.id === vector.id);
      assert.equal(native.source, vector.source);
      assert.equal(phase.source, vector.source);
      for (const other of raw) {
        assert.deepEqual(
          other.native.timings.find((row) => row.id === vector.id).outputs,
          native.outputs,
        );
        assert.deepEqual(
          other.phase.vectors.find((row) => row.id === vector.id).outputs,
          phase.outputs,
        );
      }
      assert.deepEqual(phase.outputs.fullCompile, native.outputs.off);
      const row = {
        ...vector,
        options: phase.options,
        processed: phase.outputs.scanner.code,
        variables: phase.logicalWork.variables,
        expected: { authored: native.outputs.off, processed: phase.outputs.processedInputCompile },
      };
      assert.deepEqual(row.options, {
        filename: "profile-雪😀.css",
        minify: false,
        sourceMap: false,
      });
      assert.equal(row.expected.authored.cssVars.length, row.variables);
      assert.equal(row.expected.processed.cssVars.length, 0);
      if (vector.id !== "400-bindings") {
        assert.equal(row.processed, row.source);
        assert.deepEqual(row.expected.authored, row.expected.processed);
      }
      return row;
    });
  }
  assert.deepEqual(
    rows.base,
    rows.head,
    "complete derived inputs/references must agree across original source roles",
  );
  return rows;
}
function worker(configFile, side, operation, destination) {
  const config = read(configFile);
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(process.version, campaign.node);
  assert.ok(["authored", "processed"].includes(operation));
  const image = config.images[side],
    native = createRequire(import.meta.url)(image.transportBinary);
  validateReplayImage(
    side,
    image.producer,
    digest(image.transportBinary),
    native.getNativeBuildToken(),
  );
  const packet = {
    schema: "vize.css.native-operation-attribution",
    version: 1,
    harnessSource: config.harnessSource,
    operation,
    side,
    producer: image.producer,
    transportBinary: image.transportBinary,
    calls: [],
    timings: [],
    status: "attempt",
    environment: {
      node: process.version,
      execArgv: process.execArgv,
      nodeOptions: process.env.NODE_OPTIONS ?? null,
      cpu: os.cpus()[0].model,
    },
  };
  const retain = () => write(destination, packet);
  retain();
  for (const row of config.rows) {
    const source = operation === "authored" ? row.source : row.processed,
      result = native.compileCss(source, row.options);
    packet.calls.push({
      id: row.id,
      source,
      sourceSha256: hash(Buffer.from(source)),
      options: row.options,
      result,
      reference: row.expected[operation],
      logicalVariables: operation === "authored" ? row.variables : 0,
    });
    retain();
    assert.deepEqual(result, row.expected[operation]);
    assert.equal(result.map, undefined);
    const timing = { id: row.id, batch: row.batch, samples: [] };
    packet.timings.push(timing);
    for (let warm = 0; warm < 100; warm++) native.compileCss(source, row.options);
    for (let sample = 0; sample < 20; sample++) {
      const start = process.hrtime.bigint();
      let units = 0;
      for (let call = 0; call < row.batch; call++) {
        const result = native.compileCss(source, row.options);
        units += result.code.length;
      }
      timing.samples.push({
        calls: row.batch,
        elapsedNs: Number(process.hrtime.bigint() - start),
        codeUnits: units,
      });
    }
    retain();
  }
  packet.status = "complete";
  retain();
}
async function run(destination) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(process.version, campaign.node);
  destination = path.resolve(destination);
  assert.equal(fs.existsSync(destination), false);
  fs.mkdirSync(destination, { recursive: true });
  const harnessSource = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  assert.equal(harnessSource, process.env.GITHUB_SHA);
  const record = {
    schema: "vize.css.native-operation-attribution-campaign",
    version: 1,
    harnessSource,
    harnessEvent: process.env.GITHUB_EVENT_NAME,
    harnessRef: process.env.GITHUB_REF,
    campaign,
    order: attributionOrder(),
    status: "attempt",
    note: "Artifact replay attribution only. Original producer identities remain unchanged; no new product build, instruction ceiling or threshold waiver.",
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
    const endpoint = `https://api.github.com/repos/ubugeeei-prod/vize/actions/artifacts/${campaign.artifact}`;
    const metadata = await fetch(endpoint, { headers, signal: AbortSignal.timeout(30000) });
    assert.equal(metadata.status, 200);
    const meta = await metadata.json();
    write(path.join(destination, "original-artifact-metadata.json"), meta);
    validateArtifact(meta);
    const redirect = await fetch(endpoint + "/zip", {
      headers,
      redirect: "manual",
      signal: AbortSignal.timeout(30000),
    });
    assert.equal(redirect.status, 302);
    const location = redirect.headers.get("location");
    assert.ok(location?.startsWith("https://"));
    const download = await fetch(location, { signal: AbortSignal.timeout(120000) });
    assert.equal(download.status, 200);
    const archive = Buffer.from(await download.arrayBuffer());
    assert.equal(archive.length, campaign.bytes);
    assert.equal(hash(archive), campaign.digest);
    const temporary = fs.mkdtempSync(path.join(process.env.RUNNER_TEMP, "css-native-attribution-")),
      zip = path.join(temporary, "original.zip"),
      origin = path.join(temporary, "original");
    fs.writeFileSync(zip, archive);
    execFileSync("unzip", ["-q", zip, "-d", origin]);
    const rows = inputsFromOriginal(origin),
      original = read(path.join(origin, "receipt.json")),
      images = {};
    for (const side of ["base", "head"]) {
      const producer = original.builds.find((image) => image.side === side),
        transportBinary = path.join(origin, `${side}-image`, path.basename(producer.binary));
      validateReplayImage(side, producer, digest(transportBinary), producer.token);
      images[side] = { producer, transportBinary };
    }
    const config = {
      harnessSource,
      rows: rows.base,
      images,
      referenceSources: Object.fromEntries(
        Object.entries(campaign.images).map(([side, image]) => [side, image.source]),
      ),
    };
    record.derivedInputsVerifiedAcrossBothSources = true;
    write(path.join(destination, "inputs.json"), config);
    record.inputsSha256 = digest(path.join(destination, "inputs.json"));
    record.originalReceiptSha256 = digest(path.join(origin, "receipt.json"));
    record.archiveVerified = true;
    retain();
    const packets = [];
    for (const [ordinal, item] of attributionOrder().entries()) {
      assert.equal(digest(path.join(destination, "inputs.json")), record.inputsSha256);
      const file = path.join(destination, `${ordinal}-${item.side}-${item.operation}.json`),
        log = fs.openSync(
          path.join(destination, `${ordinal}-${item.side}-${item.operation}.log`),
          "w",
        );
      try {
        execFileSync(
          "timeout",
          [
            "--signal=TERM",
            "--kill-after=5s",
            "60s",
            process.execPath,
            script,
            "--worker",
            path.join(destination, "inputs.json"),
            item.side,
            item.operation,
            file,
          ],
          { stdio: ["ignore", log, log] },
        );
      } finally {
        fs.closeSync(log);
      }
      const packet = read(file);
      assert.equal(packet.status, "complete");
      assert.equal(packet.calls.length, 3);
      assert.equal(packet.timings.length, 3);
      packets.push(packet);
    }
    record.summary = profileVectors().map((vector) => ({
      id: vector.id,
      results: ["base", "head"].flatMap((side) =>
        ["authored", "processed"].map((operation) => {
          const matches = packets.filter(
            (packet) => packet.side === side && packet.operation === operation,
          );
          assert.equal(matches.length, 2);
          assert.deepEqual(matches[0].calls, matches[1].calls);
          const samples = matches
            .flatMap((packet) =>
              packet.timings
                .find((row) => row.id === vector.id)
                .samples.map((sample) => {
                  assert.equal(sample.calls, vector.batch);
                  assert.ok(sample.elapsedNs > 0 && sample.codeUnits > 0);
                  return sample.elapsedNs / sample.calls;
                }),
            )
            .sort((a, b) => a - b);
          assert.equal(samples.length, 40);
          return {
            side,
            operation,
            samples,
            p50Ns: samples[Math.floor(samples.length / 2)],
            p95Ns: samples[Math.floor(samples.length * 0.95)],
          };
        }),
      ),
    }));
    record.retainedCalls = packets.reduce((count, packet) => count + packet.calls.length, 0);
    assert.equal(record.retainedCalls, 24);
    record.retainedControlCalls = packets
      .flatMap((packet) => packet.calls)
      .filter((call) => call.id !== "400-bindings").length;
    assert.equal(record.retainedControlCalls, 16);
    record.retainedBindingCalls = record.retainedCalls - record.retainedControlCalls;
    assert.equal(record.retainedBindingCalls, 8);
    record.timingBatches = packets
      .flatMap((packet) => packet.timings)
      .reduce((count, row) => count + row.samples.length, 0);
    assert.equal(record.timingBatches, 480);
    record.status = "complete";
    retain();
    console.log(JSON.stringify(record.summary, null, 2));
  } catch (error) {
    record.status = "failed";
    record.failure = { name: error.name, message: error.message };
    retain();
    throw error;
  }
}
if (process.argv[1] && path.resolve(process.argv[1]) === script) {
  if (process.argv[2] === "--worker") worker(...process.argv.slice(3));
  else await run(process.argv[2]);
}
