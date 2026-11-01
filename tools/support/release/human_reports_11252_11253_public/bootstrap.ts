import assert from "node:assert/strict";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { github } from "./network.ts";
import { bytes, gitBlob, hash, object, text } from "./value.ts";
import { mainManualContext } from "./context.ts";
/** Materialize only a manually supplied, immutable Git blob. No arbitrary code,
 * executable, capsule, provider or original fixture can be selected by the blob. */
export async function bootstrap(
  blobSha: string,
  role: string,
  phase: string,
  output: string,
): Promise<void> {
  mainManualContext();
  hash(blobSha, 40);
  assert(["runtime-prop-null-11252", "imports-vue-ts-11253"].includes(role));
  assert(["before", "after"].includes(phase));
  mkdirSync(output);
  const blob = object(await github(`git/blobs/${blobSha}`));
  assert.equal(blob.encoding, "base64");
  assert(Number.isSafeInteger(blob.size) && Number(blob.size) <= 1024 * 1024);
  const raw = bytes(text(blob.content).replace(/\n/gu, ""));
  assert.equal(gitBlob(raw), blobSha);
  const packet = object(JSON.parse(raw.toString("utf8")));
  assert.equal(packet.schema, "vize.humanReportPublic.manualRequest.v1");
  const request = object(packet.request);
  assert.equal(request.role, role);
  assert.equal(object(request.release).phase, phase);
  if (packet.oracleBase64 !== null) {
    const oracle = bytes(text(packet.oracleBase64));
    const oracleFile = resolve(output, "source-owned-oracles.json");
    writeFileSync(oracleFile, oracle, { flag: "wx" });
    object(request.oracleOwnerReceipt).path = oracleFile;
  }
  writeFileSync(resolve(output, "request.json"), JSON.stringify(request, null, 2) + "\n", {
    flag: "wx",
  });
  writeFileSync(resolve(output, "manual-request-blob.json"), raw, { flag: "wx" });
  process.stdout.write("manual-request-authenticated\n");
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [blob, role, phase, output] = process.argv.slice(2);
  if (!blob || !role || !phase || !output) throw new Error("blob role phase output required");
  bootstrap(blob, role, phase, resolve(output)).catch((error: unknown) => {
    process.stderr.write(String(error) + "\n");
    process.exitCode = 1;
  });
}
