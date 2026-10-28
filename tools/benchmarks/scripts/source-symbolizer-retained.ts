/** Offline source-line admission using only immutable existing ELF/capture bytes. */
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import { retainedNormalIdentity } from "./source-symbolizer-smoke.ts";
import { frozenSourceLocations } from "./source-symbolizer-protocol.ts";
import {
  boundedCommand,
  createSentinelAdapter,
  digest,
  probeDecoders,
  renderSourceCpu,
} from "./source-symbolizer-render.ts";

const sourceSha = "d16307e5eebffa43277a4fd9e5a471b5312256cc";
const normalSha256 = "bc6d3fa4d1fa4488fe996a330248c52d51451ca6b9e94a88656b73ca1398996b";
const carrierSha256 = "930688f953ef0f6c8341d70dd34be749ee2971823ae63b1655182268ba3d3bfb";
export function addressFrames(output: string, addresses: string[]) {
  const frames: { address: string; body: string[] }[] = [];
  let current: { address: string; body: string[] } | undefined;
  for (const line of output.replaceAll("\r\n", "\n").trimEnd().split("\n")) {
    if (line === ",") continue;
    if (/^0x[0-9a-f]+$/iu.test(line)) {
      current = { address: BigInt(line).toString(16), body: [] };
      frames.push(current);
    } else {
      assert.ok(current, "source body must follow its address header");
      current.body.push(line);
    }
  }
  assert.deepEqual(
    frames.map((frame) => frame.address),
    addresses.map((address) => BigInt("0x" + address).toString(16)),
  );
  assert.ok(
    frames.every((frame) => frame.body.length >= 2 && frame.body.length % 2 === 0),
    "every address needs complete function/source pairs",
  );
  return frames;
}
export function replayRetainedSource(options: {
  perf: string;
  root: string;
  carrierRoot: string;
  out: string;
  decoder: string;
}) {
  assert.ok(!existsSync(options.out), "fresh offline evidence required");
  mkdirSync(options.out, { recursive: true });
  const carrier = join(options.carrierRoot, "carrier/carrier");
  const carrierReceipt = JSON.parse(
    readFileSync(join(options.carrierRoot, "carrier/result.json"), "utf8"),
  );
  assert.equal(carrierReceipt.qualified, true);
  assert.equal(carrierReceipt.binarySha256, carrierSha256);
  assert.equal(digest(carrier), carrierSha256);
  const adapter = createSentinelAdapter(join(options.out, "adapter"), options.decoder);
  const tiny = probeDecoders(carrier, join(options.out, "tiny-query"), [adapter]);
  writeFileSync(join(options.out, "tiny-probes.json"), JSON.stringify(tiny, null, 2) + "\n");
  assert.ok(tiny[0]?.qualified, "retained tiny original Rust query must pass first");
  const { original, build, binary } = retainedNormalIdentity(options.root);
  assert.equal(build.sourceSha, sourceSha);
  assert.equal(original.binarySha256, normalSha256);
  const normal = probeDecoders(binary, join(options.out, "normal-query"), [adapter], {
    symbol: "generate_vue_virtual_ts",
    source: "/crates/vize_canon/src/batch/virtual_project/vue_codegen.rs:46",
    timeoutMs: 10_000,
  });
  writeFileSync(join(options.out, "normal-probes.json"), JSON.stringify(normal, null, 2) + "\n");
  const probe = normal[0];
  assert.ok(probe?.qualified && probe.knownAddresses, "sealed normal ELF protocol failed");
  const addresses = probe.knownAddresses;
  const control = boundedCommand(
    join(options.out, "gnu-independent"),
    "/usr/bin/addr2line",
    ["-e", binary, "-a", "-i", "-f"],
    10_000,
    addresses.join("\n") + "\n",
  );
  assert.equal(control.status, 0, "independent one-shot GNU decoder failed");
  const reference = addressFrames(control.stdout, addresses);
  const adapted = addressFrames(
    readFileSync(join(options.out, "normal-query/0/known-source/stdout.txt"), "utf8"),
    addresses,
  );
  for (const [index, frame] of adapted.entries()) {
    const expected = reference[index];
    assert.ok(expected);
    assert.ok(frame.body.some((line) => line.includes("generate_vue_virtual_ts")));
    const expectedSource =
      build.sourceRoot + "/crates/vize_canon/src/batch/virtual_project/vue_codegen.rs:46";
    assert.ok(
      expected.body.includes(expectedSource),
      "reference must identify the exact compiled function line",
    );
    assert.ok(
      frame.body.includes(expectedSource),
      "adapted frame must identify the same frozen function line",
    );
  }
  const before = digest(binary),
    data = join(options.root, "record-head/direct-process/perf.data");
  const dataBefore = digest(data);
  assert.equal(dataBefore, "fbe10d0345f26b8d38cb1c17ae4f0b575c7bc2ae1d712bb09a327fca471f50f3");
  const decoded = renderSourceCpu({
    binary,
    data,
    decoder: adapter,
    perf: options.perf,
    out: join(options.out, "decoded"),
    totalSamples: original.sampleCount,
    reportTimeoutMs: 30_000,
  });
  assert.equal(digest(binary), before);
  assert.equal(digest(data), dataBefore);
  const result = {
    schemaVersion: 1,
    qualified: decoded.qualified,
    observerSource: process.env.GITHUB_SHA ?? null,
    compiledSource: sourceSha,
    binarySha256: before,
    dataSha256: dataBefore,
    carrier: { binarySha256: carrierSha256, originalReceipt: carrierReceipt, probes: tiny },
    normalProbes: normal,
    independentWitness: { addresses, reference, adapted },
    sourceLocations: frozenSourceLocations(decoded.reports.self.rows, build.sourceRoot, sourceSha),
    originalCapture: original,
    build,
    decoded,
    metric: "retained software CPU self samples; offline decoding only",
    newCheckExecution: false,
    newCapture: false,
    newBuild: false,
    hardwareInstructions: null,
    coldQualified: false,
  };
  writeFileSync(join(options.out, "retained-source.json"), JSON.stringify(result, null, 2) + "\n");
  assert.ok(result.qualified, "retained source lines remain unqualified");
  console.log(
    JSON.stringify({
      qualified: result.qualified,
      compiledSource: sourceSha,
      binarySha256: before,
      totalSamples: original.sampleCount,
      census: decoded.reports.self.census,
    }),
  );
  return result;
}
if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      perf: { type: "string" },
      root: { type: "string" },
      "carrier-root": { type: "string" },
      out: { type: "string" },
      decoder: { type: "string", default: "/usr/bin/llvm-addr2line" },
    },
  });
  assert.ok(values.perf && values.root && values["carrier-root"] && values.out);
  replayRetainedSource({
    perf: values.perf,
    root: resolve(values.root),
    carrierRoot: resolve(values["carrier-root"]),
    out: resolve(values.out),
    decoder: values.decoder,
  });
}
