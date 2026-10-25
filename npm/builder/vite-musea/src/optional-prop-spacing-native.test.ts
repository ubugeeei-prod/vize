import assert from "node:assert/strict";
import { appendFile, mkdir, writeFile } from "node:fs/promises";
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
    const trace = async (phase: string) => {
      console.log(JSON.stringify({ phase: "optional-prop-spacing", step: phase }));
      await appendFile(path.join(output, "phases.jsonl"), JSON.stringify({ phase }) + "\n");
    };
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
    const binding = createRequire(import.meta.url)("@vizejs/native");
    assert.equal(typeof binding.analyzeSfc, "function");
    const packets: unknown[] = [];
    const initial = `<script setup lang="ts">const props = defineProps<{ ${members} }>();</script><template><div>{{ props.required }}</div></template>`;
    await writeFile(component, initial);
    const server = await createServer({
      root,
      configFile: false,
      resolve: {
        alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
      },
      plugins: [vize(), musea({ include: ["src/**/*.art.vue"] })],
      server: { host: "127.0.0.1", port: 0 },
    });
    try {
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const endpoint = `http://127.0.0.1:${address.port}/__musea__/api/arts/${encodeURIComponent(art)}`;
      for (const newline of ["\n", "\r\n"]) {
        const source = initial.replaceAll("\n", newline);
        await trace(`write-${JSON.stringify(newline)}`);
        await writeFile(component, source);
        await trace("native-begin");
        const native = binding.analyzeSfc(source, {
          filename: component,
          cache: false,
          profile: false,
        });
        await trace("native-end");
        assert.deepEqual(native, { emits: [], props: expected });
        await trace("http-analysis-begin");
        const analysisResponse = await fetch(`${endpoint}/analysis`);
        assert.equal(analysisResponse.status, 200);
        const analysis = await analysisResponse.json();
        await trace("http-analysis-end");
        assert.deepEqual(analysis, native);
        await trace("http-palette-begin");
        const paletteResponse = await fetch(`${endpoint}/palette`);
        assert.equal(paletteResponse.status, 200);
        const palette = await paletteResponse.json();
        await trace("http-palette-end");
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
      await writeFile(
        path.join(output, "packets.json"),
        JSON.stringify({ parser: ts.version, expected, packets }, null, 2),
      );
      await server.close();
    }
  },
);
