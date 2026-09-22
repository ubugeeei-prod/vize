// Renders one template three ways through the real Vue 3.5 server renderer:
// the upstream `@vue/compiler-ssr` output (via `vue/compiler-sfc`), the
// current vize SSR output, and a pinned historical vize output. Reads
// `{ cases: [...] }` on stdin and prints one JSON result per case.
//
// Case fields: `name`, `template`, `vize` (current code, preamble included),
// `legacy` (pinned historical code), `data` (the `_ctx` state), `setup`
// (names exposed from `setup()`), `bindings` (compiler binding metadata),
// `attrs` (fallthrough attrs the parent passes).
//
// Each result carries the rendered HTML (or the thrown error) per lane, with
// the attributes inside every start tag sorted: HTML attribute order carries
// no meaning, and vize orders merged `class` / `style` keys last.
//
// Vue is loaded from the `vue-stable` catalog through a workspace package
// that depends on it (`npm/ui`), so the oracle is the pinned 3.5 line rather
// than the beta the `tests` package tracks. `VIZE_SSR_VUE_ORACLE` overrides
// the package the resolution starts from.

import assert from "node:assert/strict";
import { createRequire } from "node:module";
import process from "node:process";
import { fileURLToPath } from "node:url";

const oracle =
  process.env.VIZE_SSR_VUE_ORACLE ??
  fileURLToPath(new URL("../../../npm/ui/package.json", import.meta.url));
const require = createRequire(oracle);
const vue = require("vue");
const { compileTemplate } = require("vue/compiler-sfc");
const serverRenderer = require("vue/server-renderer");
assert.match(vue.version, /^3\.5\./, `the SSR oracle must be Vue 3.5, got ${vue.version}`);
const { createSSRApp, h } = vue;

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
// `check`: exit non-zero unless every case renders exactly like upstream on
// the current lane while the pinned historical output threw or differed.
const { cases, check } = JSON.parse(Buffer.concat(chunks).toString("utf8"));
const failures = [];

// Import an ES-module SSR render function with its `vue` and server-renderer
// imports bound to the oracle's single Vue instance (a `data:` module cannot
// resolve bare specifiers, so they read the instance from `globalThis`).
globalThis.__vizeSsrOracle = { vue, server: serverRenderer };
async function load(code) {
  const body = code.replace(
    /import \{([^}]*)\} from "(vue|@vue\/server-renderer|vue\/server-renderer)"\n?/g,
    (_, names, source) => {
      const binding = source === "vue" ? "vue" : "server";
      return `const {${names.replace(/ as /g, ": ")}} = globalThis.__vizeSsrOracle.${binding};\n`;
    },
  );
  // Upstream exports `ssrRender`; vize's module declares it without `export`.
  const source = /export function ssrRender/.test(body) ? body : `${body}\nexport { ssrRender };`;
  const module = await import(`data:text/javascript,${encodeURIComponent(source)}`);
  return module.ssrRender;
}

// Directives exercising every `getSSRProps` channel the compilers route.
const directives = {
  focus: {
    getSSRProps: (binding) => ({
      "data-value": String(binding.value),
      "data-arg": String(binding.arg),
      "data-mods": Object.keys(binding.modifiers ?? {}).join(","),
      id: "from-directive",
    }),
  },
  content: { getSSRProps: () => ({ textContent: "<text from directive>" }) },
  markup: { getSSRProps: () => ({ innerHTML: "<b>html from directive</b>" }) },
  val: { getSSRProps: () => ({ value: "value from directive" }) },
};

const Foo = { render: () => h("i", "foo") };

const normalize = (html) =>
  html.replace(
    /<([a-zA-Z][^\s/>]*)((?:\s+[^\s=>]+(?:="[^"]*")?)+)(\s*\/?)>/g,
    (_, tag, attrs, end) => {
      const sorted = attrs
        .trim()
        .match(/[^\s=]+(?:="[^"]*")?/g)
        .sort();
      return `<${tag} ${sorted.join(" ")}${end}>`;
    },
  );

async function render(code, fixture) {
  try {
    const ssrRender = await load(code);
    // `setup` names are exposed from `setup()` (read as `$setup.x` under
    // binding metadata); `vFocus` is the setup-bound `focus` directive.
    const setupNames = fixture.setup ?? [];
    const data = fixture.data ?? {};
    const setupState = Object.fromEntries(
      setupNames.map((name) => [name, name === "vFocus" ? directives.focus : data[name]]),
    );
    const component = {
      ssrRender,
      directives,
      components: { Foo },
      data: () =>
        Object.fromEntries(Object.entries(data).filter(([key]) => !setupNames.includes(key))),
      setup: setupNames.length ? () => setupState : undefined,
    };
    const parent = {
      render: () =>
        h(component, fixture.attrs ?? null, {
          default: (props) => h("em", JSON.stringify(props)),
        }),
    };
    return { html: normalize(await serverRenderer.renderToString(createSSRApp(parent))) };
  } catch (error) {
    return { error: `${error?.name}: ${error?.message}` };
  }
}

for (const fixture of cases) {
  const upstream = compileTemplate({
    source: fixture.template,
    filename: `${fixture.name}.vue`,
    id: fixture.name,
    ssr: true,
    ssrCssVars: [],
    compilerOptions: fixture.bindings ? { bindingMetadata: fixture.bindings } : {},
  });
  const result = { name: fixture.name, vue_version: vue.version };
  if (upstream.errors.length) {
    result.upstream_errors = upstream.errors.map(String);
  } else {
    result.vue = await render(upstream.code, fixture);
    result.vize = await render(fixture.vize, fixture);
    result.legacy = await render(fixture.legacy, fixture);
  }
  process.stdout.write(`${JSON.stringify(result)}\n`);
  const upstreamHtml = result.vue?.html;
  const aligned = upstreamHtml !== undefined && result.vize?.html === upstreamHtml;
  const legacyWrong = result.legacy?.error !== undefined || result.legacy?.html !== upstreamHtml;
  if (!aligned || !legacyWrong) failures.push(fixture.name);
}

if (check && failures.length) {
  process.stderr.write(`SSR render differential failed for: ${failures.join(", ")}\n`);
  process.exit(1);
}
