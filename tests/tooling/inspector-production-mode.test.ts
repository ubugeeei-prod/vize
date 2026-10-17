import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { compileFunction } from "node:vm";
import ts from "typescript";

const fixture = new URL(
  "../_fixtures/differential/compiler/inspector-production-mode/",
  import.meta.url,
);
const read = (name: string) => readFileSync(new URL(name, fixture), "utf8");
const source = read("Mutes.vue.txt");
const origin = JSON.parse(read("source.json"));
const requireVue = createRequire(new URL("../package.json", import.meta.url));
const vue: typeof import("vue") = requireVue("vue");
const server: typeof import("vue/server-renderer") = requireVue("vue/server-renderer");
const rust = readFileSync(
  new URL("../../crates/vize/src/commands/inspector.rs", import.meta.url),
  "utf8",
);
const marker = 'const OFFICIAL_COMPILER_NODE_SCRIPT: &str = r#"';
assert.equal(rust.split(marker).length, 2);
const embedded = rust.slice(rust.indexOf(marker) + marker.length).split('"#;')[0];
const cwd = fileURLToPath(new URL("../", import.meta.url));

function compare(target: "dom" | "ssr", parentMode: string | undefined): string {
  const env = { ...process.env };
  if (parentMode === undefined) delete env.NODE_ENV;
  else env.NODE_ENV = parentMode;
  const result = spawnSync(process.execPath, ["--input-type=module", "-e", embedded], {
    cwd,
    env,
    input: JSON.stringify({
      target,
      moduleRoot: cwd,
      files: [{ path: origin.path, source, vizeCode: "" }],
    }),
    encoding: "utf8",
    timeout: 30_000,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stderr, "");
  const { files } = JSON.parse(result.stdout);
  assert.equal(files.length, 1);
  assert.equal(files[0].path, origin.path);
  assert.equal(files[0].official.error, null);
  assert.deepEqual(files[0].official.warnings, []);
  return files[0].official.code;
}

void test("Inspector's production reference is independent of its parent Node environment", () => {
  assert.equal(Buffer.byteLength(source), origin.sourceBytes);
  assert.equal(createHash("sha256").update(source).digest("hex"), origin.sourceSha256);
  assert.equal(requireVue("vue/compiler-sfc").version, origin.officialVersion);
  for (const target of ["dom", "ssr"] as const) {
    const expected = read(`Mutes.production.${target}.expected.ts.txt`);
    assert.notEqual(expected, read(`Mutes.development.${target}.expected.ts.txt`));
    for (const mode of [undefined, "development", "production"]) {
      assert.equal(compare(target, mode), expected, `${target}, parent NODE_ENV=${mode}`);
    }
  }
});

type Push = (html: string) => void;
type Render = (
  ctx: object,
  push: Push,
  parent: unknown,
  attrs: object,
  props: object,
  setup: object,
  data: object,
  options: object,
) => void;

function loadRender(code: string): Render {
  const ast = ts.createSourceFile("Mutes.ts", code, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  const imports = ast.statements.filter(ts.isImportDeclaration);
  const render = ast.statements.find(
    (node) => ts.isFunctionDeclaration(node) && node.name?.text === "ssrRender",
  );
  assert.ok(render, "retain the complete original emitted render function");
  const extracted =
    imports.map((node) => node.getFullText(ast)).join("\n") +
    "\n" +
    render.getFullText(ast).replace(/export\s+function\s+ssrRender/, "function ssrRender") +
    "\nexports.render = ssrRender;";
  const output = ts.transpileModule(extracted, {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS },
  }).outputText;
  const exports: { render?: Render } = {};
  compileFunction(output, ["require", "exports"])((specifier: string) => {
    assert.ok(["vue", "vue/server-renderer"].includes(specifier));
    return specifier === "vue" ? vue : server;
  }, exports);
  assert.ok(exports.render);
  return exports.render;
}

async function renderOriginal(
  code: string,
  route: "push" | "vnode",
  hydrated: boolean,
): Promise<string> {
  const generated = loadRender(code);
  const app = vue.createSSRApp({
    ssrRender(_ctx: unknown, push: Push, parent: unknown, attrs: object) {
      generated(
        { isHydrated: hydrated, $t: (key: string) => key },
        push,
        parent,
        attrs,
        {},
        {},
        {},
        {},
      );
    },
  });
  const host =
    route === "vnode"
      ? vue.defineComponent({
          inheritAttrs: false,
          render() {
            return vue.h("main", null, [
              vue.h("header", null, this.$slots.title?.()),
              vue.h("section", null, this.$slots.default?.()),
            ]);
          },
        })
      : vue.defineComponent({
          inheritAttrs: false,
          ssrRender(ctx: { $slots: import("vue").Slots }, push: Push, parent: unknown) {
            push("<main><header>");
            ctx.$slots.title?.({}, push, parent, "");
            push("</header><section>");
            ctx.$slots.default?.({}, push, parent, "");
            push("</section></main>");
          },
        });
  app.component("MainContent", host);
  app.component(
    "MainTitle",
    vue.defineComponent({
      render() {
        return vue.h("h1", null, this.$slots.default?.());
      },
    }),
  );
  app.component("TimelineMutes", {
    render() {
      return vue.h("p", null, "hydrated list");
    },
  });
  return server.renderToString(app);
}

void test("the complete original SSR render preserves production comments in both slot routes", async () => {
  assert.equal(vue.version, origin.officialVersion);
  const actual = compare("ssr", "development");
  const previous = read("Mutes.development.ssr.expected.ts.txt");
  for (const route of ["push", "vnode"] as const) {
    for (const hydrated of [false, true]) {
      const content = hydrated ? "<p>hydrated list</p>" : "<!---->";
      const expected = `<main><header><h1>nav.muted_users</h1></header><section>${content}</section></main>`;
      assert.equal(await renderOriginal(actual, route, hydrated), expected);
      const oldHtml = await renderOriginal(previous, route, hydrated);
      if (route === "vnode" && !hydrated) {
        assert.equal(oldHtml, expected.replace("<!---->", "<!--v-if-->"));
        assert.notEqual(
          oldHtml,
          expected,
          "the previous development oracle must fail the original false branch",
        );
      } else
        assert.equal(oldHtml, expected, "true branch and direct SSR push are unchanged controls");
    }
  }
});
