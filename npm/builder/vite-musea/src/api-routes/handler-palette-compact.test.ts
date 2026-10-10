import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import os from "node:os";
import path from "node:path";
import { Readable } from "node:stream";
import test from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import type { ResolvedConfig } from "vite";
import { compactPropsArtSource, compactPropsFixture } from "../compact-props-fixtures.ts";
import { processMuseaArtFile } from "../plugin/art-processing.ts";
import { createApiMiddleware, type ApiRoutesContext } from "./index.ts";

void test("actual native Art analysis and palette retain complete compact type-literal metadata", async () => {
  const fixture = await compactPropsFixture();
  const expectations = new Map<string, unknown>(
    (
      await Promise.all(
        ["compact-props-palette-literals.json", "compact-props-palette-types.json"].map(
          async (name) =>
            JSON.parse(
              await readFile(
                new URL(
                  `../../../../../tests/_fixtures/differential/musea/${name}`,
                  import.meta.url,
                ),
                "utf8",
              ),
            ) as Array<{ name: string; expected: unknown }>,
        ),
      )
    )
      .flat()
      .map((item) => [item.name, item.expected]),
  );
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-compact-api-"));
  const output = fileURLToPath(
    new URL("../../../../../artifacts/musea-native-inline/compact-api", import.meta.url),
  );
  const observations: Record<string, unknown>[] = [];
  try {
    await mkdir(path.join(root, "src"));
    for (const [index, vector] of fixture.cases.entries()) {
      const filename = `probe${index}.vue`;
      const artPath = path.join(root, "src", `probe${index}.art.vue`);
      const artSource = compactPropsArtSource({
        name: vector.name,
        filename,
        title: "Compact Probe",
        source: vector.source,
        names: vector.expected.props.map((prop) => prop.name),
      });
      await writeFile(path.join(root, "src", filename), vector.source);
      await writeFile(artPath, artSource);
      const art = await processMuseaArtFile(artPath, { root, command: "build" });
      assert.ok(art, vector.name);
      const ctx: ApiRoutesContext = {
        config: { root } as ResolvedConfig,
        artFiles: new Map([[artPath, art]]),
        scanRoots: [root],
        tokensPath: undefined,
        basePath: "/__musea__",
        resolvedPreviewCss: [],
        resolvedPreviewSetup: null,
        devSessionToken: "compact-props-api-test",
        processArtFile: async () => {},
        getDevServerPort: () => 5173,
      };
      const observed: Record<string, unknown> = { vector, artSource, art };
      observations.push(observed);
      for (const endpoint of ["analysis", "palette"] as const) {
        const req = Readable.from([]) as IncomingMessage;
        req.method = "GET";
        req.url = `/arts/${encodeURIComponent(artPath)}/${endpoint}`;
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
        observed[endpoint] = { body, parsed: JSON.parse(body) };
        assert.deepEqual(
          JSON.parse(body),
          endpoint === "analysis"
            ? vector.expected
            : {
                ...(expectations.get(vector.name) as Record<string, unknown>),
                ...(vector.expected.props.some((prop) => prop.name === "__proto__")
                  ? { unsupportedProps: ["__proto__"] }
                  : {}),
              },
          vector.name,
        );
        if (endpoint === "palette") {
          const result = ts.transpileModule(JSON.parse(body).typescript, {
            reportDiagnostics: true,
          });
          observed.typescriptDiagnostics = result.diagnostics;
          assert.deepEqual(result.diagnostics, []);
        }
      }
    }
  } finally {
    try {
      await mkdir(output, { recursive: true });
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify(
          { fixture, expectations: Object.fromEntries(expectations), observations },
          null,
          2,
        ),
      );
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  }
});
