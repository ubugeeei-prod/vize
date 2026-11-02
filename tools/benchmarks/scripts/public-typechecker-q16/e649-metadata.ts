import assert from "node:assert/strict";
import { asRecord, asString, type JsonRecord } from "../../../support/json-value.ts";
import { boundedBytes } from "../../../support/release/release/quota/http.ts";
import { publicUrl, sriSha512 } from "../../../support/release/public_install/registry.ts";
import { sha256, type Profile } from "./contract.ts";
import type { Recorder } from "./execute.ts";

export const e649MetadataUrl = "https://registry.npmjs.org/vize";
export const e649MetadataAccept = "application/vnd.npm.install-v1+json";
export function usesE649Packument(profile: Profile, name: string, version: string) {
  return (
    profile.subset === "E90-91" &&
    profile.version === "0.649.0" &&
    name === "vize" &&
    version === "0.649.0"
  );
}
export function selectE649Metadata(value: unknown, lock: JsonRecord, capsuleEntry: JsonRecord) {
  const packument = asRecord(value);
  assert.equal(packument.name, "vize");
  const versions = asRecord(packument.versions);
  assert(Object.hasOwn(versions, "0.649.0"));
  const selected = asRecord(versions["0.649.0"]);
  assert.equal(selected.name, "vize");
  assert.equal(selected.version, "0.649.0");
  assert.equal(capsuleEntry.metadataOrigin, e649MetadataUrl);
  const dist = asRecord(selected.dist);
  sriSha512(dist.integrity);
  publicUrl(asString(dist.tarball));
  assert.equal(dist.integrity, lock.integrity);
  assert.equal(dist.integrity, capsuleEntry.integrity);
  assert.equal(dist.tarball, lock.resolved);
  assert.equal(dist.tarball, capsuleEntry.tarball);
  assert.equal(
    publicUrl(asString(asRecord(dist.attestations).url)),
    "https://registry.npmjs.org/-/npm/v1/attestations/vize@0.649.0",
  );
  return selected;
}
export async function readE649Metadata(
  profile: Profile,
  lock: JsonRecord,
  capsuleEntry: JsonRecord,
  record: Pick<Recorder, "raw" | "json">,
  cache: Map<string, Buffer>,
  fetchImpl: typeof fetch = globalThis.fetch,
) {
  assert(usesE649Packument(profile, "vize", asString(lock.version)));
  const key = "E649_INSTALL_PACKUMENT_RAW:" + e649MetadataUrl;
  let raw = cache.get(key);
  const cached = raw !== undefined;
  if (raw)
    record.json("vize-install-packument.http.json", {
      url: e649MetadataUrl,
      accept: e649MetadataAccept,
      source: "SAME_RUN_SHARED_RAW_CACHE",
    });
  else {
    const response = await fetchImpl(e649MetadataUrl, {
      redirect: "error",
      signal: AbortSignal.timeout(15_000),
      headers: {
        Accept: e649MetadataAccept,
        "User-Agent": "VizeReleaseVerification/1 (https://github.com/ubugeeei-prod/vize)",
      },
    });
    record.json("vize-install-packument.http.json", {
      url: response.url,
      requestedUrl: e649MetadataUrl,
      accept: e649MetadataAccept,
      status: response.status,
      date: response.headers.get("date"),
      contentType: response.headers.get("content-type"),
      source: "ACTUAL_HTTP_RESPONSE",
    });
    raw = await boundedBytes(response, 4 * 1024 * 1024);
    record.raw("vize-install-packument.raw.json", raw);
    assert.equal(response.url, e649MetadataUrl);
    assert.equal(response.status, 200);
    cache.set(key, raw);
  }
  assert(raw.length <= 4 * 1024 * 1024);
  if (cached) record.raw("vize-install-packument.raw.json", raw);
  const selected = selectE649Metadata(
    JSON.parse(new TextDecoder("utf8", { fatal: true }).decode(raw)),
    lock,
    capsuleEntry,
  );
  record.json("vize-install-packument-selection.json", {
    url: e649MetadataUrl,
    accept: e649MetadataAccept,
    rawBytes: raw.length,
    rawSha256: sha256(raw),
    selectedVersion: "0.649.0",
    selectedEntry: selected,
    representation: "EXACT_ABBREVIATED_INSTALL_PACKUMENT_MEMBER",
    canonicalVersionHttpStatusClaimed: false,
  });
  return selected;
}
