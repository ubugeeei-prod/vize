import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { Readable } from "node:stream";
import os from "node:os";
import path from "node:path";
import type { ResolvedConfig } from "vite";

import { createApiMiddleware, type ApiRoutesContext } from "./index.ts";
import type { ArtFileInfo } from "../types/art.ts";

interface CapturedResponse {
  body: string;
  headers: Record<string, string>;
  nextCalled: boolean;
  statusCode: number;
}

function createArt(pathname: string): ArtFileInfo {
  return {
    path: pathname,
    metadata: {
      title: "Escape",
      tags: [],
      status: "ready",
    },
    variants: [],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
}

function createContext(root: string, artFiles = new Map<string, ArtFileInfo>()): ApiRoutesContext {
  return {
    config: { root } as ResolvedConfig,
    artFiles,
    scanRoots: [root],
    tokensPath: undefined,
    basePath: "/__musea__",
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    devSessionToken: "test-session",
    processArtFile: async () => {},
    getDevServerPort: () => 5173,
  };
}

function authorizedJsonHeaders(ctx: ApiRoutesContext): IncomingMessage["headers"] {
  return {
    host: "localhost:5173",
    origin: "http://localhost:5173",
    "content-type": "application/json",
    "x-musea-session": ctx.devSessionToken,
  };
}

async function invokeApi(
  ctx: ApiRoutesContext,
  init: {
    body?: string;
    headers?: IncomingMessage["headers"];
    method: string;
    url: string;
  },
): Promise<CapturedResponse> {
  return await new Promise((resolve, reject) => {
    const captured: CapturedResponse = {
      body: "",
      headers: {},
      nextCalled: false,
      statusCode: 200,
    };

    const req = Readable.from(init.body === undefined ? [] : [init.body]) as IncomingMessage;
    req.method = init.method;
    req.url = init.url;
    req.headers = init.headers ?? {};

    const res = {
      get statusCode() {
        return captured.statusCode;
      },
      set statusCode(value: number) {
        captured.statusCode = value;
      },
      setHeader(name: string, value: number | string | string[]) {
        captured.headers[name.toLowerCase()] = Array.isArray(value)
          ? value.join(", ")
          : String(value);
      },
      end(chunk?: Buffer | string) {
        if (chunk) {
          captured.body += Buffer.isBuffer(chunk) ? chunk.toString("utf-8") : chunk;
        }
        resolve(captured);
      },
    } as ServerResponse;

    const next = () => {
      captured.nextCalled = true;
      resolve(captured);
    };

    Promise.resolve(createApiMiddleware(ctx)(req, res, next)).catch(reject);
  });
}

void test("createApiMiddleware returns 400 for malformed encoded art paths", async () => {
  const response = await invokeApi(createContext(process.cwd()), {
    method: "GET",
    url: "/arts/%E0%A4%A/source",
  });

  assert.equal(response.statusCode, 400);
  assert.match(response.body, /art path is not valid URL encoding/);
});

void test("createApiMiddleware blocks source writes outside the configured root", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-security-"));
  const root = path.join(tempDir, "root");
  const outside = path.join(tempDir, "outside");
  const outsideArt = path.join(outside, "Escape.art.vue");

  try {
    await fs.promises.mkdir(root);
    await fs.promises.mkdir(outside);
    await fs.promises.writeFile(outsideArt, "original", "utf-8");

    const ctx = createContext(root, new Map([[outsideArt, createArt(outsideArt)]]));
    const response = await invokeApi(ctx, {
      method: "PUT",
      url: `/arts/${encodeURIComponent(outsideArt)}/source`,
      headers: {
        host: "localhost:5173",
        origin: "http://localhost:5173",
        "content-type": "application/json",
        "x-musea-session": ctx.devSessionToken,
      },
      body: JSON.stringify({ source: "escaped" }),
    });

    assert.equal(response.statusCode, 400);
    assert.match(response.body, /art path escapes the allowed directory/);
    assert.equal(await fs.promises.readFile(outsideArt, "utf-8"), "original");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("createApiMiddleware returns 400 for malformed JSON mutation bodies", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-json-"));

  try {
    const ctx = createContext(tempDir);
    ctx.tokensPath = "tokens.json";

    for (const [method, url] of [
      ["POST", "/preview-with-props"],
      ["POST", "/generate"],
      ["POST", "/run-vrt"],
      ["POST", "/tokens"],
      ["PUT", "/tokens"],
      ["DELETE", "/tokens"],
    ] as const) {
      const response = await invokeApi(ctx, {
        method,
        url,
        headers: authorizedJsonHeaders(ctx),
        body: "{",
      });

      assert.equal(response.statusCode, 400, `${method} ${url}`);
      assert.deepEqual(JSON.parse(response.body), { error: "Malformed JSON body" });
    }
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("createApiMiddleware returns 400 for malformed art source JSON", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-source-json-"));
  const artPath = "src/Escape.art.vue";
  const artFilePath = path.join(tempDir, artPath);

  try {
    await fs.promises.mkdir(path.dirname(artFilePath), { recursive: true });
    await fs.promises.writeFile(artFilePath, "original", "utf-8");

    const ctx = createContext(tempDir, new Map([[artPath, createArt(artPath)]]));
    const response = await invokeApi(ctx, {
      method: "PUT",
      url: `/arts/${encodeURIComponent(artPath)}/source`,
      headers: authorizedJsonHeaders(ctx),
      body: "{",
    });

    assert.equal(response.statusCode, 400);
    assert.deepEqual(JSON.parse(response.body), { error: "Malformed JSON body" });
    assert.equal(await fs.promises.readFile(artFilePath, "utf-8"), "original");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("createApiMiddleware resolves tokensPath inside external scan roots", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-tokens-root-"));
  const appRoot = path.join(tempDir, "app");
  const designRoot = path.join(tempDir, "design");
  const tokensDir = path.join(designRoot, "tokens");

  try {
    await fs.promises.mkdir(appRoot, { recursive: true });
    await fs.promises.mkdir(tokensDir, { recursive: true });
    await fs.promises.writeFile(
      path.join(tokensDir, "colors.tokens.json"),
      JSON.stringify({
        color: {
          primitive: {
            gray: {
              50: { value: "#f7f7f7" },
            },
          },
        },
      }),
      "utf-8",
    );

    const ctx = createContext(appRoot);
    ctx.scanRoots = [appRoot, designRoot];
    ctx.tokensPath = "../design/tokens";

    const response = await invokeApi(ctx, {
      method: "GET",
      url: "/tokens",
    });

    assert.equal(response.statusCode, 200, response.body);
    const body = JSON.parse(response.body);
    assert.equal(body.error, undefined);
    assert.equal(body.meta.filePath, tokensDir);
    assert.equal(body.tokenMap["color.primitive.gray.50"].value, "#f7f7f7");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("createApiMiddleware populates inline art palette controls from host component props", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-inline-palette-"));
  const componentPath = path.join(tempDir, "src", "InlineButton.vue");

  try {
    await fs.promises.mkdir(path.dirname(componentPath), { recursive: true });
    await fs.promises.writeFile(
      componentPath,
      `<script setup lang="ts">
defineProps<{
  tone?: "brand" | "neutral";
  disabled?: boolean;
}>()
</script>

<template>
  <button :disabled="disabled">{{ tone }}</button>
</template>

<art title="Inline Button">
  <variant name="Default">
    <Self tone="brand" :disabled="false" />
  </variant>
</art>
`,
      "utf-8",
    );

    const art = createArt(componentPath);
    art.isInline = true;
    art.componentPath = componentPath;

    const ctx = createContext(tempDir, new Map([[componentPath, art]]));
    const response = await invokeApi(ctx, {
      method: "GET",
      url: `/arts/${encodeURIComponent(componentPath)}/palette`,
    });

    assert.equal(response.statusCode, 200, response.body);

    const body = JSON.parse(response.body) as {
      controls: Array<{
        name: string;
        control: string;
        required: boolean;
        options: Array<{ label: string; value: unknown }>;
      }>;
    };
    assert.deepEqual(
      body.controls.map((control) => ({
        name: control.name,
        control: control.control,
        required: control.required,
        options: control.options,
      })),
      [
        {
          name: "tone",
          control: "select",
          required: false,
          options: [
            { label: "brand", value: "brand" },
            { label: "neutral", value: "neutral" },
          ],
        },
        {
          name: "disabled",
          control: "boolean",
          required: false,
          options: [],
        },
      ],
    );
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});
