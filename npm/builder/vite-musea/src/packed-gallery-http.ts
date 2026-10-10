import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Response } from "playwright";

export interface PackedGalleryHttpRecord {
  url: string;
  status: number;
  bytes: number;
  sha256: string;
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
  return (response: Response) => {
    if (!response.url().startsWith(origin + "/")) return;
    pending.push(
      (async () => {
        const body = await response.body();
        const digest = sha256(body);
        await mkdir(path.join(caseOutput, "http-bodies"), { recursive: true });
        await writeFile(path.join(caseOutput, "http-bodies", `${digest}.bin`), body);
        responses.push({
          url: response.url(),
          status: response.status(),
          bytes: body.length,
          sha256: digest,
        });
      })().catch((error) => {
        httpErrors.push(String(error));
      }),
    );
  };
}
