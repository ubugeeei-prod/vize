import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import type { Plugin } from "vite-plus";

const prefix = "virtual:vize-svg-namespace-8624/";
const root = new URL("../../../tests/_fixtures/differential/compiler/", import.meta.url);
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
// Keep native resolution relative to its original entry, never Vite's bundled config.
const nativeEntry = new URL("../../../npm/native/index.js", import.meta.url);
const fromNative = createRequire(nativeEntry);
const native = fromNative(
  fileURLToPath(nativeEntry),
) as typeof import("../../../npm/native/index.js");
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const compiler = fromVue("@vue/compiler-sfc");
const runtime = fromUi.resolve("vue-vapor-runtime/dist/vue.runtime-with-vapor.esm-browser.js");
const hash = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");

/** The fresh source NAPI producer and independent stock VDOM share one real browser runtime. */
export function svgNamespaceCorpusPlugin(): Plugin {
  return {
    name: "vize:svg-namespace-8624-corpus",
    enforce: "pre",
    resolveId(id) {
      if (id.startsWith(prefix)) return `\0${id}`;
    },
    load(id) {
      if (!id.startsWith(`\0${prefix}`)) return;
      const name = id.slice(prefix.length + 1);
      assert.equal(compiler.version, "3.6.0-rc.9");
      assert.equal(fromVue("./package.json").version, compiler.version);
      if (name === "runtime") return readFileSync(runtime, "utf8");
      const packBytes = readFileSync(new URL("vapor-svg-namespace-8624.manifest.json", root));
      const registry = JSON.parse(readFileSync(new URL("manifest.json", root), "utf8"));
      assert.equal(
        registry.runtimePacks.find(
          (row: { path: string }) => row.path === "vapor-svg-namespace-8624.manifest.json",
        ).sha256,
        hash(packBytes),
      );
      const pack = JSON.parse(packBytes.toString()).cases[0];
      for (const file of pack.inputs.files) {
        assert.equal(
          hash(readFileSync(new URL(`${pack.inputs.root}/${file.path}`, root))),
          file.sha256,
        );
      }
      const controls = JSON.parse(readFileSync(new URL(pack.reference.path, root), "utf8"));
      assert.equal(controls.length, 1);
      if (name === "controls") {
        const bindings = Object.values(fromNative.cache).filter(
          (module) =>
            module?.filename.endsWith(".node") && module.exports.compileSfc === native.compileSfc,
        );
        assert.equal(bindings.length, 1, "one actual native compiler binding");
        const binding = realpathSync(bindings[0]!.filename);
        assert.equal(
          dirname(binding),
          realpathSync(new URL("../../../npm/native/", import.meta.url)),
          "the browser gate must use this checkout's freshly built NAPI artifact",
        );
        const checkout = execFileSync("git", ["rev-parse", "HEAD"], {
          cwd: fileURLToPath(new URL("../../../", import.meta.url)),
          encoding: "utf8",
        }).trim();
        if (process.env.GITHUB_SHA) assert.equal(checkout, process.env.GITHUB_SHA);
        return `export default ${JSON.stringify(controls[0])}; export const custody = ${JSON.stringify(
          {
            sourceSha256: pack.inputs.files.find(
              (file: { path: string }) => file.path === "App.vue.txt",
            ).sha256,
            controlsSha256: pack.reference.sha256,
            runtimeVersion: compiler.version,
            checkout,
            bindingSha256: hash(readFileSync(binding)),
            productProvider: "fresh-source-napi",
            oracleProvider: "independent-stock-vdom",
            publicationQualification: false,
          },
        )};`;
      }
      if (name === "state-vapor" || name === "state-vdom") {
        return `import { reactive } from '${prefix}runtime'; export const state = reactive(${JSON.stringify(controls[0].context)});`;
      }
      assert.ok(name === "vapor" || name === "vdom");
      const source = readFileSync(new URL(`${pack.inputs.root}/App.vue.txt`, root), "utf8");
      let code: string;
      if (name === "vapor") {
        const result = native.compileSfc(source, {
          filename: "App.vue",
          vapor: true,
          sourceMap: false,
          ssr: false,
        });
        assert.deepEqual(result.errors, []);
        assert.deepEqual(result.warnings, []);
        code = result.code;
      } else {
        const parsed = compiler.parse(source, { filename: "App.vue" });
        assert.deepEqual(parsed.errors, []);
        // The complete original source is parsed unchanged; only the independent backend changes.
        const descriptor = { ...parsed.descriptor, vapor: false };
        const script = compiler.compileScript(descriptor, {
          id: "namespace-8624",
          genDefaultAs: "__component",
        });
        const template = compiler.compileTemplate({
          source: descriptor.template.content,
          filename: "App.vue",
          id: "namespace-8624",
          compilerOptions: { bindingMetadata: script.bindings },
        });
        assert.deepEqual(template.errors, []);
        assert.deepEqual(template.tips, []);
        code = `${script.content}\n${template.code}\n__component.render = render;\nexport default __component;`;
      }
      assert.equal((code.match(/from ["']\.\/fixture["']/gu) ?? []).length, 1);
      const emittedSha256 = hash(code);
      code = code
        .replace(/from (["'])vue\1/gu, `from '${prefix}runtime'`)
        .replace(/from ["']\.\/fixture["']/u, `from '${prefix}state-${name}'`);
      return `${code}\nexport const emittedSha256 = ${JSON.stringify(emittedSha256)};`;
    },
  };
}
