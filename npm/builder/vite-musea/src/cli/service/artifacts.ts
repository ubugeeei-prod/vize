import { randomUUID } from "node:crypto";
import { readFile } from "node:fs/promises";
import type { ServerResponse } from "node:http";
import type { VrtResult } from "../../vrt/types.js";

/** Registrations contain frozen capture bytes, never request-derived filesystem paths. */
export class SessionArtifacts {
  private files = new Map<string, { bytes: Buffer; type: string }>();

  add(bytes: Buffer | string, type: string): string {
    const url = `/artifacts/${randomUUID()}`;
    this.files.set(url, { bytes: Buffer.from(bytes), type });
    return url;
  }

  clear(): void {
    this.files.clear();
  }

  send(url: string, response: ServerResponse): boolean {
    const artifact = this.files.get(url);
    if (!artifact) return false;
    response.setHeader("Content-Type", artifact.type);
    response.setHeader("X-Content-Type-Options", "nosniff");
    response.setHeader(
      "Content-Security-Policy",
      "default-src 'none'; style-src 'unsafe-inline'; img-src data:; sandbox",
    );
    response.end(artifact.bytes);
    return true;
  }

  async results(results: VrtResult[]) {
    return Promise.all(
      results.map(async (result) => {
        const urls: { snapshot?: string; current?: string; diff?: string } = {};
        for (const [key, file] of [
          ["snapshot", result.snapshotPath],
          ["current", result.currentPath],
          ["diff", result.diffPath],
        ] as const) {
          if (file && !result.error) urls[key] = this.add(await readFile(file), "image/png");
        }
        return {
          artPath: result.artPath,
          variantName: result.variantName,
          viewport: result.viewport.name,
          passed: result.passed,
          isNew: result.isNew,
          diffPercentage: result.diffPercentage,
          error: result.error,
          images: urls,
        };
      }),
    );
  }
}
