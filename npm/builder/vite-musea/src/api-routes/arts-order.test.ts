import assert from "node:assert/strict";
import type { IncomingMessage, ServerResponse } from "node:http";
import { Readable } from "node:stream";
import test from "node:test";
import type { ResolvedConfig } from "vite";

import { createApiMiddleware, type ApiRoutesContext } from "./index.js";
import type { ArtFileInfo } from "../types/index.js";

function art(title: string, order?: number): ArtFileInfo {
  const pathname = `/repo/${title}.art.vue`;
  return {
    path: pathname,
    metadata: { title, tags: [], status: "ready", order },
    variants: [],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
}

async function fetchArts(artFiles: Map<string, ArtFileInfo>): Promise<ArtFileInfo[]> {
  const ctx: ApiRoutesContext = {
    config: { root: process.cwd() } as ResolvedConfig,
    artFiles,
    scanRoots: [process.cwd()],
    tokensPath: undefined,
    basePath: "/__musea__",
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    devSessionToken: "test-session",
    processArtFile: async () => {},
    getDevServerPort: () => 5173,
  };

  return await new Promise((resolve, reject) => {
    const req = Readable.from([]) as IncomingMessage;
    req.method = "GET";
    req.url = "/arts";
    req.headers = {};

    const res = {
      statusCode: 200,
      setHeader() {},
      end(chunk?: Buffer | string) {
        resolve(JSON.parse(String(chunk)) as ArtFileInfo[]);
      },
    } as unknown as ServerResponse;

    Promise.resolve(
      createApiMiddleware(ctx)(req, res, () => reject(new Error("next called"))),
    ).catch(reject);
  });
}

void test("arts API returns entries sorted by defineArt order", async () => {
  const high = art("High", 30);
  const fallback = art("Fallback");
  const low = art("Low", 10);

  const arts = await fetchArts(
    new Map([
      [high.path, high],
      [fallback.path, fallback],
      [low.path, low],
    ]),
  );

  assert.deepEqual(
    arts.map((item) => item.metadata.title),
    ["Low", "High", "Fallback"],
  );
});
