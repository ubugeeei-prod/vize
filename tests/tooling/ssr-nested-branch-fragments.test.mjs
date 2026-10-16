import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { compileFunction } from "node:vm";
import { gunzipSync } from "node:zlib";

const capture = process.env.VIZE_NESTED_BRANCH_CAPTURE;

await test(
  "nested conditional SSR fragments keep parent attributes on the actual root",
  { skip: !capture && "source-built SSR modules execute in the scoped SSR action" },
  async () => {
    const fromTests = createRequire(new URL("../package.json", import.meta.url));
    const vue = fromTests("vue");
    const fromVue = createRequire(fromTests.resolve("vue/package.json"));
    const server = fromVue("@vue/server-renderer");
    const compiler = fromVue("@vue/compiler-sfc");
    assert.equal(vue.version, "3.6.0-beta.10");
    assert.equal(compiler.version, vue.version);
    const modules = JSON.parse(readFileSync(capture, "utf8"));
    assert.equal(modules.length, 40);

    function load(preamble, body) {
      const imports = preamble.replace(
        /import \{([^}]+)\} from ["']([^"']+)["'];?/g,
        (_, bindings, name) => {
          assert.ok(["vue", "@vue/server-renderer", "vue/server-renderer"].includes(name));
          return `const {${bindings.replaceAll(/\bas\b/g, ":")}} = require(${JSON.stringify(name)});`;
        },
      );
      return compileFunction(
        `${imports}\n${body.replace("export function ssrRender", "function ssrRender")}\nreturn ssrRender;`,
        ["require"],
      )((name) => (name === "vue" ? vue : server));
    }

    async function render(renderFunction, outer, inner, itemKeys) {
      const app = vue.createSSRApp(
        { props: ["outer", "inner"], ssrRender: renderFunction },
        { outer, inner, "data-owner": "parent" },
      );
      app.component("RuntimeChild", {
        props: ["n"],
        render() {
          return vue.h("b", { "data-item": this.n }, String(this.n));
        },
      });
      app.component("RuntimeSlotHost", {
        render() {
          // Calling without an SSR push function forces the generated VNode
          // fallback. Capture the actual item keys before Vue renders them.
          const children = this.$slots.default?.() ?? [];
          if (itemKeys) itemKeys.push(...(children[0]?.children ?? []).map((child) => child.key));
          return vue.h("section", null, children);
        },
      });
      return server.renderToString(app);
    }

    const observations = [];
    for (const entry of modules) {
      assert.equal(entry.lane, "s4");
      assert.deepEqual(entry.selected, entry.legacy);
      // Vize's Vapor TransitionGroup retains nested ranges, like a physical
      // section. Official Vue SSR has only the VDOM flattening policy.
      const referenceSource =
        entry.vapor && entry.name === "transition-group-loop-conditional"
          ? entry.source
              .replace('<TransitionGroup tag="section">', "<section>")
              .replace("</TransitionGroup>", "</section>")
          : entry.source;
      const reference = compiler.compileTemplate({
        source: referenceSource,
        filename: "NestedBranch.vue",
        id: "nested-branch",
        ssr: true,
        ssrCssVars: [],
        isProd: true,
      });
      assert.deepEqual(reference.errors, []);
      // Official template code contains its own import declarations.
      const official = load(
        reference.code.match(/^import[^\n]+/gm)?.join("\n") ?? "",
        reference.code.replace(/^import[^\n]+\n/gm, ""),
      );
      for (const outer of [false, true]) {
        for (const inner of [false, true]) {
          const officialItemKeys = [];
          const officialHtml = await render(official, outer, inner, officialItemKeys);
          const lanes = {};
          for (const lane of ["selected", "legacy"]) {
            const output = entry[lane];
            const itemKeys = [];
            const html = await render(load(output.preamble, output.code), outer, inner, itemKeys);
            if (["fallback-keyed-component", "fallback-keyed-slot"].includes(entry.name)) {
              assert.deepEqual(itemKeys, officialItemKeys, `${entry.name}: actual loop item keys`);
              assert.deepEqual(
                itemKeys,
                entry.name === "fallback-keyed-component" ? [1, 2] : ["1_fb", "2_fb"],
              );
            }
            assert.equal(html, officialHtml, `${entry.name}: ${lane}, vapor=${entry.vapor}`);
            const fragment = entry.name === "nested-root" ? outer : !outer;
            if (entry.name.startsWith("nested-") && fragment) {
              assert.equal(html, `<!--[--><button>${inner ? "a" : "b"}</button><!--]-->`);
            }
            if (entry.name === "single-template-root" && outer) {
              assert.equal(html, '<button data-owner="parent">b</button>');
            }
            if (entry.name === "plain-root-chain") assert.ok(html.includes('data-owner="parent"'));
            if (entry.name === "loop-owns-fragment" && outer) {
              assert.equal(html, "<!--[--><button>1</button><button>2</button><!--]-->");
            }
            if (entry.name === "loop-conditional-boundary") {
              const item = outer
                ? `<!--[--><button>${inner ? "a" : "b"}</button><!--]-->`
                : "<!---->";
              assert.equal(html, `<!--[--><!--[-->${item}<!--]--><!--[-->${item}<!--]--><!--]-->`);
            }
            if (entry.name === "loop-single-element") {
              assert.equal(html, "<!--[--><button>1</button><button>2</button><!--]-->");
            }
            lanes[lane] = { html, itemKeys };
          }
          observations.push({
            name: entry.name,
            vapor: entry.vapor,
            outer,
            inner,
            referenceSource,
            officialHtml,
            officialItemKeys,
            lanes,
          });
        }
      }
    }
    const published = JSON.parse(
      gunzipSync(
        readFileSync(
          new URL(
            "../../docs/davinci/decisions/fixtures/ssr-loop-fragments-20261016/published-0.457.0.json.gz",
            import.meta.url,
          ),
        ),
      ),
    );
    assert.equal(published.vueVersion, vue.version);
    const publicCode = published.modules.vize;
    const publicRender = load(
      publicCode.match(/^import[^\n]+/gm)?.join("\n") ?? "",
      publicCode.replace(/^import[^\n]+\n/gm, "").replace(/^export default[^\n]+$/gm, ""),
    );
    const publishedObservations = [];
    for (const retained of published.observations) {
      const html = await render(publicRender, retained.outer, retained.inner);
      assert.equal(html, retained.vize, "retained public module reproduces its original HTML");
      const current = observations.find(
        (entry) =>
          entry.name === "loop-conditional-boundary" &&
          !entry.vapor &&
          entry.outer === retained.outer &&
          entry.inner === retained.inner,
      );
      assert.equal(current.officialHtml, retained.official);
      assert.notEqual(
        html,
        current.officialHtml,
        "public compiler must fail the anchor regression",
      );
      publishedObservations.push({ ...retained, publicHtml: html });
    }
    const destination = process.env.VIZE_NESTED_BRANCH_RUNTIME_CAPTURE;
    assert.ok(destination, "retain complete actual runtime outcomes");
    writeFileSync(
      destination,
      JSON.stringify({
        vueVersion: vue.version,
        compilerVersion: compiler.version,
        observations,
        publishedObservations,
      }),
    );
  },
);
