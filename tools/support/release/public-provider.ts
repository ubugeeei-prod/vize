import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { asArray, asNumber, asRecord, asString } from "../json-value.ts";

export interface NativeImage {
  bytes: number;
  sha256: string;
}

export function validatePublicProvider(
  receipt: unknown,
  version: string,
  source: string,
  image: NativeImage,
) {
  const record = asRecord(receipt);
  assert.equal(asString(record.version), version);
  assert.equal(asString(record.source), source);
  const providers = asArray(asRecord(record.provider).files)
    .map((file) => asRecord(file))
    .filter((file) => asString(file.path).endsWith(".node"));
  assert.equal(providers.length, 1);
  const provider = providers[0];
  assert.ok(provider);
  assert.equal(asNumber(provider.bytes), image.bytes);
  assert.equal(asString(provider.sha256), image.sha256);
  return { version, source, provider };
}

export async function verifyPublicProvider(version: string, source: string, image: NativeImage) {
  assert.match(version, /^[0-9]+\.[0-9]+\.[0-9]+$/);
  assert.match(source, /^[a-f0-9]{40}$/);
  const target = (process.platform === "win32" ? "win32" : process.platform) + "-" + process.arch;
  const url =
    "https://github.com/ubugeeei-prod/vize/releases/download/v" +
    version +
    "/vize-" +
    target +
    ".vsix.receipt.json";
  const response = await fetch(url);
  assert.equal(response.status, 200, url);
  const bytes = Buffer.from(await response.arrayBuffer());
  const receipt: unknown = JSON.parse(bytes.toString("utf8"));
  return {
    url,
    receiptSha256: createHash("sha256").update(bytes).digest("hex"),
    ...validatePublicProvider(receipt, version, source, image),
  };
}
