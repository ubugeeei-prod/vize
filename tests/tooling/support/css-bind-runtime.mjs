import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const require = createRequire(
  fileURLToPath(new URL("../../../npm/ui/package.json", import.meta.url)),
);
const vue = require("vue");
const server = require("vue/server-renderer");
const { parse, compileScript, compileTemplate, compileStyle } = require("vue/compiler-sfc");
const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const cases = JSON.parse(Buffer.concat(chunks).toString("utf8"));

async function load(code, runtime) {
  globalThis.__vizeCssBindRuntime = runtime;
  const body = code.replace(
    /import\s*\{([^}]*)\}\s*from\s*["'](vue|@vue\/server-renderer|vue\/server-renderer)["'];?/g,
    (_, names, source) =>
      `const {${names.replace(/ as /g, ": ")}} = globalThis.__vizeCssBindRuntime.${source === "vue" ? "vue" : "server"};`,
  );
  return (await import(`data:text/javascript,${encodeURIComponent(body)}`)).default;
}

for (const fixture of cases) {
  const names = [...fixture.css.matchAll(/var\(--([a-zA-Z0-9_-]+)\)/g)].map((match) => match[1]);
  assert.equal(names.length, 2, `${fixture.name}: both declarations survive`);
  assert.deepEqual(
    [...fixture.delegated.matchAll(/var\(--([a-zA-Z0-9_-]+)\)/g)].map((match) => match[1]),
    names,
    `${fixture.name}: delegated stylesheet uses every runtime name`,
  );
  if (!fixture.isProd) {
    for (const name of names) assert.ok(fixture.pipeline.includes(`var(--${name})`));
    assert.equal(names[1], "f82a533a-fogOpacity");
  }
  const [complexName, fogName] = names;
  assert.match(complexName, /^[a-zA-Z][a-zA-Z0-9_-]*$/);

  let getter;
  const client = await load(fixture.client, {
    vue: {
      ...vue,
      useCssVars: (callback) => {
        getter = callback;
      },
    },
    server,
  });
  await server.renderToString(vue.createSSRApp(client));
  assert.equal(typeof getter, "function");
  const values = getter({});
  assert.equal(values[complexName], fixture.expected, fixture.name);
  assert.equal(values[fogName], 0.5);

  const ssr = await load(fixture.ssr, { vue, server });
  const html = await server.renderToString(vue.createSSRApp(ssr));
  assert.ok(html.includes(`--${complexName}:${fixture.expected}`), `${html}\n${fixture.ssr}`);
  assert.ok(html.includes(`--${fogName}:0.5`), html);

  // The official compiler confirms the expression value on an SSR root too.
  const { descriptor } = parse(fixture.source);
  const script = compileScript(descriptor, { id: "f82a533a" });
  const template = compileTemplate({
    source: descriptor.template.content,
    filename: "app/pages/index.vue",
    id: "f82a533a",
    scoped: true,
    ssr: true,
    ssrCssVars: descriptor.cssVars,
    compilerOptions: { bindingMetadata: script.bindings },
  });
  assert.deepEqual(template.errors, []);
  const css = compileStyle({
    source: descriptor.styles[0].content,
    id: "data-v-f82a533a",
    scoped: true,
  });
  assert.deepEqual(css.errors, []);
  if (fixture.isProd) {
    // Vue's filename-based production naming independently owns the full vector.
    const productionCss = compileStyle({
      source: descriptor.styles[0].content,
      filename: fixture.filename,
      id: fixture.filename,
      isProd: true,
    });
    assert.deepEqual(productionCss.errors, []);
    assert.deepEqual(
      [...productionCss.code.matchAll(/var\(--([a-zA-Z0-9_-]+)\)/g)].map((match) => match[1]),
      names,
    );
    let officialGetter;
    const productionScript = compileScript(descriptor, {
      id: fixture.filename,
      isProd: true,
      inlineTemplate: true,
    });
    const official = await load(productionScript.content, {
      vue: {
        ...vue,
        useCssVars: (callback) => {
          officialGetter = callback;
        },
      },
      server,
    });
    await server.renderToString(vue.createSSRApp(official));
    assert.equal(typeof officialGetter, "function");
    assert.deepEqual(officialGetter({}), values, "complete official JS variable map");
  }
  const upstream = await load(
    `${script.content.replace("export default", "const component =")}\n${template.code}\ncomponent.ssrRender = ssrRender; export default component;`,
    { vue, server },
  );
  const upstreamHtml = await server.renderToString(vue.createSSRApp(upstream));
  assert.ok(upstreamHtml.includes(fixture.expected), upstreamHtml);
  process.stdout.write(`${fixture.name}: client values and SSR CSS references resolve\n`);
}
