import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Response } from "playwright";

export interface PackedGalleryHttpRecord {
  url: string;
  status: number;
  headers: Record<string, string>;
  requestHeaders: Record<string, string>;
  bytes?: number;
  sha256?: string;
  cacheValidation?: { ifNoneMatch: string; responseEtag: string | null };
  cachedBody?: {
    url: string;
    status: 200;
    etag: string;
    bytes: number;
    sha256: string;
  };
}

export interface PackedGalleryHttpContext {
  origin: string;
  caseOutput: string;
  responses: PackedGalleryHttpRecord[];
  pending: Promise<void>[];
  httpErrors: string[];
  sha256: (body: Buffer) => string;
}

export function createPackedGalleryHttpObserver(context: PackedGalleryHttpContext) {
  const { origin, caseOutput, responses, pending, httpErrors, sha256 } = context;
  const owners = new Map<string, Promise<PackedGalleryHttpRecord | undefined>>();
  return (response: Response) => {
    if (!response.url().startsWith(origin + "/")) return;
    const status = response.status();
    const priorOwner = status === 304 ? owners.get(response.url()) : undefined;
    const work = (async () => {
      const headers = await response.allHeaders();
      const requestHeaders = await response.request().allHeaders();
      const record: PackedGalleryHttpRecord = {
        url: response.url(),
        status,
        headers,
        requestHeaders,
      };
      responses.push(record);
      assert.ok(status === 200 || status === 304, `Unexpected HTTP ${status}: ${response.url()}`);
      if (status === 304) {
        assert.equal(headers.location, undefined);
        assert.equal(response.request().redirectedTo(), null);
        assert.ok(priorOwner, `Unowned cached response: ${response.url()}`);
        const prior = await priorOwner;
        assert.ok(prior && prior.status === 200 && prior.bytes !== undefined && prior.sha256);
        assert.equal(prior.url, response.url());
        assert.ok(prior.headers.etag);
        assert.equal(requestHeaders["if-none-match"], prior.headers.etag);
        if (headers.etag !== undefined) assert.equal(headers.etag, prior.headers.etag);
        record.cacheValidation = {
          ifNoneMatch: requestHeaders["if-none-match"],
          responseEtag: headers.etag ?? null,
        };
        record.cachedBody = {
          url: prior.url,
          status: 200,
          etag: prior.headers.etag,
          bytes: prior.bytes,
          sha256: prior.sha256,
        };
        return;
      }
      const body = await response.body();
      const digest = sha256(body);
      await mkdir(path.join(caseOutput, "http-bodies"), { recursive: true });
      await writeFile(path.join(caseOutput, "http-bodies", `${digest}.bin`), body);
      record.bytes = body.length;
      record.sha256 = digest;
      return record;
    })();
    if (status === 200) owners.set(response.url(), work);
    pending.push(
      work
        .then(() => undefined)
        .catch((error) => {
          httpErrors.push(String(error));
        }),
    );
  };
}
