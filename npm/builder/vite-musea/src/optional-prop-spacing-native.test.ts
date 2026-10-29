import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { appendFileSync, realpathSync } from "node:fs";
import { appendFile, copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import ts from "typescript";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";

void test(
  "actual native and HTTP optional flags follow parser question tokens with LF and CRLF",
  {
    skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1",
    timeout: 120_000,
  },
  async () => {
    const repository = fileURLToPath(new URL("../../../../", import.meta.url));
    const output = path.join(repository, "artifacts/musea-native-optional-spacing");
    await mkdir(output, { recursive: true });
    const phaseFile = path.join(output, "phases.jsonl");
    await writeFile(phaseFile, "");
    const trace = async (phase: string, details: Record<string, unknown> = {}) => {
      const entry = { phase, ...details };
      console.log(JSON.stringify({ phase: "optional-prop-spacing", ...entry }));
      await appendFile(phaseFile, JSON.stringify(entry) + "\n");
    };
    const processTrace = (phase: string, code: number) =>
      appendFileSync(phaseFile, JSON.stringify({ phase, code }) + "\n");
    process.once("beforeExit", (code) => processTrace("process-before-exit", code));
    process.once("exit", (code) => processTrace("process-exit", code));
    await trace("test-start");
    const root = path.join(output, "project");
    const component = path.join(root, "src/SpacingProbe.vue");
    const art = path.join(root, "src/SpacingProbe.art.vue");
    await mkdir(path.dirname(component), { recursive: true });
    await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
    await writeFile(
      art,
      `<script setup lang="ts">defineArt("./SpacingProbe.vue", { title: "Optional Spacing" })</script><art><variant name="Default"><SpacingProbe :required="1" /></variant></art>`,
    );
    const members =
      'spaced? : string;\ntabbed?\t: number;\ncommented? /* question */ : boolean;\ncompact?: string;\nrequired : number;\nreadonly read? : string;\n"scope.name"? : string;\ncafé? : number;\n"why?:"? /* trivia */ : boolean;';
    const typeFile = ts.createSourceFile(
      "truth.ts",
      `type Props = { ${members} }`,
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.TS,
    );
    const alias = typeFile.statements[0];
    assert.ok(alias);
    assert.ok(ts.isTypeAliasDeclaration(alias) && ts.isTypeLiteralNode(alias.type));
    const expected = alias.type.members.map((member) => {
      assert.ok(ts.isPropertySignature(member) && member.type);
      return {
        name: ts.isStringLiteral(member.name) ? member.name.text : member.name.getText(typeFile),
        type: member.type.getText(typeFile),
        required: member.questionToken === undefined,
      };
    });
    const nativeRequire = createRequire(import.meta.url);
    await trace("native-load-begin");
    const binding = nativeRequire("@vizejs/native");
    assert.equal(typeof binding.analyzeSfc, "function");
    const loadedImages = () =>
      Object.entries(nativeRequire.cache)
        .filter(([filename, module]) => filename.endsWith(".node") && module?.exports === binding)
        .map(([filename]) => realpathSync(filename));
    assert.equal(loadedImages().length, 1);
    const loadedImage = loadedImages()[0]!;
    const loadedBytes = await readFile(loadedImage);
    const loadedSha256 = createHash("sha256").update(loadedBytes).digest("hex");
    const loadedToken = binding.getNativeBuildToken();
    assert.match(loadedToken, /^[0-9a-f]{32}$/);
    const retainedImage = path.join(output, "loaded-native-image.node");
    await copyFile(loadedImage, retainedImage);
    assert.equal(
      createHash("sha256")
        .update(await readFile(retainedImage))
        .digest("hex"),
      loadedSha256,
    );
    const nativeImage = {
      issue: "https://github.com/ubugeeei-prod/vize/issues/10582",
      loadedImage,
      retainedImage,
      sha256: loadedSha256,
      buildToken: loadedToken,
      source: process.env.GITHUB_SHA ?? null,
      run: process.env.GITHUB_RUN_ID ?? null,
      attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    };
    await writeFile(path.join(output, "native-image.json"), JSON.stringify(nativeImage, null, 2));
    const verifyNativeImage = async () => {
      assert.deepEqual(loadedImages(), [loadedImage]);
      assert.equal(
        createHash("sha256")
          .update(await readFile(loadedImage))
          .digest("hex"),
        loadedSha256,
      );
      assert.equal(binding.getNativeBuildToken(), loadedToken);
    };
    await trace("native-loaded", nativeImage);
    const packets: unknown[] = [];
    const initial = `<script setup lang="ts">const props = defineProps<{ ${members} }>();</script><template><div>{{ props.required }}</div></template>`;
    await writeFile(component, initial);
    await trace("create-server-begin");
    const server = await createServer({
      root,
      configFile: false,
      resolve: {
        alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
      },
      plugins: [vize(), musea({ include: ["src/**/*.art.vue"] })],
      server: { host: "127.0.0.1", port: 0 },
    });
    await trace("create-server-end");
    try {
      await trace("listen-begin");
      await server.listen();
      await trace("listen-end");
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const endpoint = `http://127.0.0.1:${address.port}/__musea__/api/arts/${encodeURIComponent(art)}`;
      for (const newline of ["\n", "\r\n"]) {
        const source = initial.replaceAll("\n", newline);
        await trace("write-begin", { newline });
        await writeFile(component, source);
        await trace("write-end", { newline });
        await trace("native-begin", { newline });
        const native = binding.analyzeSfc(source, {
          filename: component,
          cache: false,
          profile: false,
        });
        await trace("native-end", { newline });
        assert.deepEqual(native, { emits: [], props: expected });
        await trace("http-analysis-begin", { newline });
        const analysisResponse = await fetch(`${endpoint}/analysis`);
        assert.equal(analysisResponse.status, 200);
        const analysis = await analysisResponse.json();
        await trace("http-analysis-end", { newline });
        assert.deepEqual(analysis, native);
        await trace("http-palette-begin", { newline });
        const paletteResponse = await fetch(`${endpoint}/palette`);
        assert.equal(paletteResponse.status, 200);
        const palette = await paletteResponse.json();
        await trace("http-palette-end", { newline });
        assert.deepEqual(
          palette.controls.map(({ name, required }: { name: string; required: boolean }) => ({
            name,
            required,
          })),
          expected.map(({ name, required }) => ({ name, required })),
        );
        assert.deepEqual(JSON.parse(palette.json).controls, palette.controls);
        const exported = ts.createSourceFile(
          "palette.ts",
          palette.typescript,
          ts.ScriptTarget.Latest,
          true,
          ts.ScriptKind.TS,
        );
        const declaration = exported.statements[0];
        assert.ok(declaration);
        assert.ok(ts.isInterfaceDeclaration(declaration));
        assert.deepEqual(
          declaration.members.map((member) => {
            assert.ok(ts.isPropertySignature(member));
            return {
              name: ts.isStringLiteral(member.name)
                ? member.name.text
                : member.name.getText(exported),
              required: member.questionToken === undefined,
            };
          }),
          expected.map(({ name, required }) => ({ name, required })),
        );
        packets.push({ newline, source, native, analysis, palette });
      }
    } finally {
      await trace("packets-write-begin");
      await writeFile(
        path.join(output, "packets.json"),
        JSON.stringify({ parser: ts.version, nativeImage, expected, packets }, null, 2),
      );
      await trace("packets-write-end");
      await verifyNativeImage();
      await trace("server-close-begin");
      await server.close();
      await trace("server-close-end");
      await verifyNativeImage();
    }
    await trace("test-complete");
  },
);
