import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { boundedBytes } from "../release/quota/http.ts";
import { sriSha512, publicUrl } from "../public_install/registry.ts";
import { object, sha256, text } from "./value.ts";
import { discard, type Sink } from "./evidence.ts";
export function githubReader(sink: Sink) {
  const cache = new Map<string, unknown>();
  return async (endpoint: string): Promise<unknown> => {
    if (cache.has(endpoint)) return cache.get(endpoint);
    const url = "https://api.github.com/repos/ubugeeei-prod/vize/" + endpoint;
    const response = await fetch(url, {
      redirect: "error",
      signal: AbortSignal.timeout(15000),
      headers: {
        Accept: "application/vnd.github+json",
        ...(process.env.GITHUB_TOKEN
          ? { Authorization: `Bearer ${process.env.GITHUB_TOKEN}` }
          : {}),
      },
    });
    sink("github-status", { url, status: response.status, date: response.headers.get("date") });
    const raw = await boundedBytes(response, 2 * 1024 * 1024);
    sink("github-body", {
      url,
      status: response.status,
      bytes: raw.length,
      sha256: sha256(raw),
      rawBase64: raw.toString("base64"),
    });
    assert.equal(response.status, 200);
    const value: unknown = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw));
    sink("github-json", { url, value });
    cache.set(endpoint, value);
    return value;
  };
}
export const github = githubReader(discard);
interface PublicArchive {
  resolved: string;
  integrity: string;
  body: Buffer;
  metadata: unknown;
}
const cache = new Map<string, PublicArchive>();
export async function publicArchive(
  name: string,
  version: string,
  sink: Sink = discard,
  fetcher: typeof fetch = fetch,
): Promise<PublicArchive> {
  const key = name + "@" + version,
    previous = cache.get(key);
  if (previous) return previous;
  const metadataUrl = `https://registry.npmjs.org/${encodeURIComponent(name)}/${version}`;
  const response = await fetcher(metadataUrl, { signal: AbortSignal.timeout(30_000) });
  sink("registry-status", {
    name,
    version,
    url: metadataUrl,
    status: response.status,
    date: response.headers.get("date"),
  });
  let raw: Buffer;
  try {
    raw = await boundedBytes(response, 1024 * 1024);
  } catch (error) {
    sink("registry-body-refusal", {
      name,
      version,
      url: metadataUrl,
      status: response.status,
      error: String(error),
    });
    throw error;
  }
  sink("registry-body", {
    name,
    version,
    url: metadataUrl,
    status: response.status,
    bytes: raw.length,
    sha256: sha256(raw),
    rawBase64: raw.toString("base64"),
  });
  assert.equal(response.status, 200);
  publicUrl(response.url);
  const metadata = object(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(raw)));
  sink("registry-metadata", { name, version, metadata });
  assert.equal(metadata.name, name);
  assert.equal(metadata.version, version);
  const dist = object(metadata.dist),
    resolved = publicUrl(text(dist.tarball)),
    integrity = text(dist.integrity);
  const expected = sriSha512(integrity);
  const payload = await fetcher(resolved, { signal: AbortSignal.timeout(120_000) });
  sink("registry-archive-status", {
    name,
    version,
    resolved,
    status: payload.status,
    date: payload.headers.get("date"),
  });
  assert.equal(payload.status, 200);
  publicUrl(payload.url);
  assert(payload.body);
  const chunks: Uint8Array[] = [];
  let total = 0;
  for await (const chunk of payload.body) {
    total += chunk.length;
    assert(total <= 128 * 1024 * 1024, "bounded actual package archive required");
    chunks.push(chunk);
  }
  const body = Buffer.concat(chunks);
  sink("registry-archive-hash", {
    name,
    version,
    resolved,
    bytes: body.length,
    sha256: sha256(body),
    sha512: createHash("sha512").update(body).digest("hex"),
    expectedIntegrity: integrity,
  });
  assert(createHash("sha512").update(body).digest().equals(expected));
  const result = { resolved, integrity, body, metadata };
  cache.set(key, result);
  return result;
}
