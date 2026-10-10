import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import os from "node:os";
import path from "node:path";
import { Readable } from "node:stream";
import test from "node:test";
import type { ResolvedConfig } from "vite";
import { createApiMiddleware, type ApiRoutesContext } from "./index.ts";
import { processMuseaArtFile } from "../plugin/art-processing.ts";
import { usageArtSource, usageComponentFixture } from "../usage-component-fixtures.ts";

void test("native Art palette preserves display titles and actual component tags", async () => {
  const fixture = await usageComponentFixture();
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-palette-component-"));
  try {
    await mkdir(path.join(root, "src"));
    await writeFile(path.join(root, "src/base-button.vue"), fixture.componentSource);
    for (const vector of fixture.cases) {
      const artPath = path.join(root, "src", vector.filename);
      await writeFile(artPath, usageArtSource(vector, fixture.componentSource));
      const art = await processMuseaArtFile(artPath, { root, command: "build" });
      assert.ok(art, vector.name);
      assert.equal(art.metadata.title, vector.title);
      assert.equal(art.metadata.component, vector.inline ? undefined : "./base-button.vue");
      assert.equal(art.isInline, vector.inline);
      const ctx: ApiRoutesContext = {
        config: { root } as ResolvedConfig,
        artFiles: new Map([[artPath, art]]),
        scanRoots: [root],
        tokensPath: undefined,
        basePath: "/__musea__",
        resolvedPreviewCss: [],
        resolvedPreviewSetup: null,
        devSessionToken: "component-palette-test",
        processArtFile: async () => {},
        getDevServerPort: () => 5173,
      };
      const req = Readable.from([]) as IncomingMessage;
      req.method = "GET";
      req.url = `/arts/${encodeURIComponent(artPath)}/palette`;
      req.headers = {};
      const body = await new Promise<string>((resolve, reject) => {
        const res = {
          statusCode: 200,
          setHeader() {},
          end(chunk?: Buffer | string) {
            assert.equal(this.statusCode, 200);
            resolve(Buffer.isBuffer(chunk) ? chunk.toString("utf8") : String(chunk ?? ""));
          },
        } as ServerResponse;
        Promise.resolve(
          createApiMiddleware(ctx)(req, res, () => reject(new Error("No route"))),
        ).catch(reject);
      });
      const controls = ["label", "constructor", "hasOwnProperty"].map((name) => ({
        name,
        control: "text",
        required: false,
        options: [],
      }));
      assert.deepEqual(
        JSON.parse(body),
        {
          title: vector.title,
          componentTagName: vector.componentTagName,
          controls,
          groups: [],
          json: JSON.stringify({ title: vector.title, controls }, null, 2),
          typescript:
            `export interface ${vector.propsInterface}Props {\n` +
            "  label?: string;\n  constructor?: string;\n  hasOwnProperty?: string;\n}\n",
        },
        vector.name,
      );
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
