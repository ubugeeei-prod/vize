import assert from "node:assert/strict";
import { test } from "node:test";
import {
  admitVizeModulesInHookFilter,
  patchNuxtVueTypeGatedTransform,
  toNuxtVueTransformId,
  VIZE_VUE_MODULE_ID_RE,
} from "./vue-transform-bridge.ts";

void test("Vize generated modules satisfy Nuxt's script transform identity", () => {
  const isNuxtScriptTransform = (id: string): boolean => {
    const search = id.slice(id.indexOf("?"));
    const type = /[?&]setup(?:=|&|$)/.test(search)
      ? "script"
      : /[?&]type=([^&]*)/.exec(search)?.[1];
    return /[?&]vue(?:&|$)/.test(search) && type === "script";
  };
  assert.equal(isNuxtScriptTransform("/app/pages/index.vue?vue&vize"), false);
  assert.equal(
    toNuxtVueTransformId("/app/pages/index.vue?vue&vize"),
    "/app/pages/index.vue?vue&type=script&vize",
  );
  assert.equal(isNuxtScriptTransform(toNuxtVueTransformId("/app/pages/index.vue?vue&vize")), true);
  assert.equal(
    toNuxtVueTransformId("\0/app/pages/index.vue.ts"),
    "\0/app/pages/index.vue.ts?vue&type=script",
  );
  assert.equal(
    toNuxtVueTransformId("/app/pages/index.vue?vue&vize&used=true"),
    "/app/pages/index.vue?vue&type=script&vize&used=true",
  );
  for (const id of [
    "/app/pages/index.vue",
    "/app/pages/index.vue?vue&type=style&index=0",
    "/app/pages/index.vue?raw",
    "/app/pages/index.ts?vue&vize",
  ]) {
    assert.equal(toNuxtVueTransformId(id), id);
  }
});

void test("Nuxt type-gated transforms receive the compatible identity", () => {
  const calls: string[] = [];
  const plugin = {
    name: "nuxt:route-injection-plugin",
    transform(code: string, id: string) {
      calls.push(id);
      return `${id}:${code}`;
    },
  };
  patchNuxtVueTypeGatedTransform(plugin);
  assert.equal(
    plugin.transform("render", "/app/pages/index.vue?vue&vize"),
    "/app/pages/index.vue?vue&type=script&vize:render",
  );
  assert.deepEqual(calls, ["/app/pages/index.vue?vue&type=script&vize"]);

  patchNuxtVueTypeGatedTransform(plugin);
  plugin.transform("render", "/app/pages/about.vue?vue&vize");
  assert.deepEqual(calls, [
    "/app/pages/index.vue?vue&type=script&vize",
    "/app/pages/about.vue?vue&type=script&vize",
  ]);
});

void test("handler transforms and unrelated plugins preserve their contract", () => {
  assert.doesNotThrow(() => patchNuxtVueTypeGatedTransform(undefined));
  const calls: string[] = [];
  const routePlugin = {
    name: "nuxt:vue-async-context",
    transform: {
      handler(_code: string, id: string) {
        calls.push(id);
        return null;
      },
    },
  };
  patchNuxtVueTypeGatedTransform(routePlugin);
  routePlugin.transform.handler("code", "/app/Page.vue?vue&vize");
  routePlugin.transform.handler("code", "/app/Page.vue?vue&type=style&index=0");
  assert.deepEqual(calls, [
    "/app/Page.vue?vue&type=script&vize",
    "/app/Page.vue?vue&type=style&index=0",
  ]);

  const unrelated = {
    name: "third-party:transform",
    transform: (_code: string, id: string) => id,
  };
  const original = unrelated.transform;
  patchNuxtVueTypeGatedTransform(unrelated);
  assert.equal(unrelated.transform, original);
});

// Vite 8 `createFilterForTransform` semantics for RegExp patterns.
type Patterns = RegExp | RegExp[] | { include?: RegExp | RegExp[]; exclude?: RegExp | RegExp[] };
function matches(patterns: Patterns | undefined, value: string): boolean {
  if (!patterns) return true;
  const normalized =
    patterns instanceof RegExp || Array.isArray(patterns) ? { include: patterns } : patterns;
  const list = (p?: RegExp | RegExp[]) => (p === undefined ? undefined : [p].flat());
  const include = list(normalized.include);
  if (list(normalized.exclude)?.some((re) => re.test(value))) return false;
  if (include?.some((re) => re.test(value))) return true;
  return !(include && include.length > 0);
}
function hostCalls(filter: unknown, id: string, code: string): boolean {
  const { id: idFilter, code: codeFilter } = filter as { id?: Patterns; code?: Patterns };
  return matches(idFilter, id) && matches(codeFilter, code);
}

// Nuxt 4.6.4 `nuxt:prehydrate-transform` (dist/index.mjs).
const VUE_SCRIPT_ID_FILTER = [/\.vue$/, /[?&]macro=/, /[?&]vue&type=script\b/];
const JS_ID_RE = /\.(?:[cm]?[jt]s|[jt]sx)(?:\?|$)/;
const VUE_NON_SCRIPT_BLOCK_RE = /[?&]vue&type=(?:template|style|custom)\b/;
function nuxtPrehydratePlugin(calls: string[]) {
  return {
    name: "nuxt:prehydrate-transform",
    transform: {
      filter: {
        id: { include: [...VUE_SCRIPT_ID_FILTER, JS_ID_RE], exclude: VUE_NON_SCRIPT_BLOCK_RE },
        code: { include: /onPrehydrate\(/ },
      },
      handler(_code: string, id: string) {
        calls.push(id);
        return null;
      },
    },
  };
}

void test("#11455 Nuxt's prehydrate hook filter admits Vize modules", () => {
  const calls: string[] = [];
  const plugin = nuxtPrehydratePlugin(calls);
  const original = plugin.transform.filter;
  const vizeId = "/app/app.vue?vue&vize";
  const code = "onPrehydrate(() => {});";
  assert.equal(hostCalls(plugin.transform.filter, vizeId, code), false);

  patchNuxtVueTypeGatedTransform(plugin);
  const { filter } = plugin.transform;
  assert.notEqual(filter, original);
  assert.deepEqual(original.id.include, [...VUE_SCRIPT_ID_FILTER, JS_ID_RE]);
  assert.equal(hostCalls(filter, vizeId, code), true);
  assert.equal(hostCalls(filter, "/app/app.vue?vue&vize&used=true", code), true);
  assert.equal(hostCalls(filter, "/app/app.vue?vue&vize-ssr", code), true);
  plugin.transform.handler(code, vizeId);
  assert.deepEqual(calls, ["/app/app.vue?vue&type=script&vize"]);

  // The host's own matches and refusals are unchanged.
  assert.equal(hostCalls(filter, "/app/app.vue", code), true);
  assert.equal(hostCalls(filter, "/app/plugin.ts", code), true);
  assert.equal(hostCalls(filter, vizeId, "const x = 1;"), false);
  assert.equal(hostCalls(filter, "/app/app.vue?vue&type=style&index=0", code), false);
  assert.equal(hostCalls(filter, "/app/app.vue?vue&type=template&lang.js", code), false);
  assert.equal(hostCalls(filter, "/app/app.css", code), false);
  assert.equal(hostCalls(filter, "/app/app.vue?raw", code), false);
  assert.equal(hostCalls(filter, "/app/app.vue?vue&vizes", code), false);
});

void test("hook filter widening keeps every other filter shape intact", () => {
  const shared = [/\.vue$/];
  const cases: Array<[unknown, unknown]> = [
    [{ id: /\.vue$/ }, { id: [/\.vue$/, VIZE_VUE_MODULE_ID_RE] }],
    [{ id: "**/*.vue" }, { id: ["**/*.vue", VIZE_VUE_MODULE_ID_RE] }],
    [{ id: shared }, { id: [/\.vue$/, VIZE_VUE_MODULE_ID_RE] }],
    [
      { id: { include: shared, exclude: /node_modules/ }, code: /x/ },
      { id: { include: [/\.vue$/, VIZE_VUE_MODULE_ID_RE], exclude: /node_modules/ }, code: /x/ },
    ],
  ];
  for (const [input, expected] of cases) {
    assert.deepEqual(admitVizeModulesInHookFilter(input), expected);
  }
  assert.deepEqual(shared, [/\.vue$/]);

  // These already admit Vize ids, or have no shape to extend.
  for (const filter of [
    undefined,
    { code: /x/ },
    { id: [] },
    { id: { include: [] } },
    { id: { exclude: /\.css$/ } },
    [{ kind: "include" }],
  ]) {
    assert.equal(admitVizeModulesInHookFilter(filter), filter);
  }
});

void test("filterless hooks and unrelated plugins keep their filters", () => {
  const calls: string[] = [];
  const legacy = {
    name: "nuxt:prehydrate-transform",
    transform: {
      handler(_code: string, id: string) {
        calls.push(id);
      },
    },
  };
  patchNuxtVueTypeGatedTransform(legacy);
  assert.equal("filter" in legacy.transform, false);
  legacy.transform.handler("onPrehydrate(() => {})", "/app/app.vue?vue&vize");
  assert.deepEqual(calls, ["/app/app.vue?vue&type=script&vize"]);

  const filter = { id: { include: [/\.vue$/] } };
  const unrelated = { name: "third-party:transform", transform: { filter, handler: () => null } };
  patchNuxtVueTypeGatedTransform(unrelated);
  assert.equal(unrelated.transform.filter, filter);
  assert.equal(hostCalls(unrelated.transform.filter, "/app/app.vue?vue&vize", ""), false);
});

// Nuxt 4.6.4 `isVue(id, { type: ["script", "template"] })` (dist/index.mjs), the
// `transformInclude` of `nuxt:imports-transform` and other function-form gates.
const VUE_SCRIPT_TEMPLATE_ID_FILTER = [
  /\.vue$/,
  /[?&]macro=/,
  /[?&]vue&type=script\b/,
  /[?&]vue&type=template\b/,
];
const isNuxtVueScriptOrTemplate = (id: string) =>
  VUE_SCRIPT_TEMPLATE_ID_FILTER.some((re) => re.test(id));

void test("#11592 Nuxt's regex gates read the script identity of a Vize module", () => {
  const calls: string[] = [];
  const plugin = {
    name: "nuxt:imports-transform",
    transform(_code: string, id: string) {
      if (!isNuxtVueScriptOrTemplate(id)) return null;
      calls.push(id);
      return "import { unref as __unimport_unref } from 'vue';";
    },
  };
  patchNuxtVueTypeGatedTransform(plugin);
  for (const id of ["/app/Foo/Index.vue?vue&vize", "/app/Foo/Index.vue?vue&vize-ssr"]) {
    assert.match(String(plugin.transform("_ctx.unref(x)", id)), /__unimport_unref/, id);
  }
  assert.deepEqual(calls, [
    "/app/Foo/Index.vue?vue&type=script&vize",
    "/app/Foo/Index.vue?vue&type=script&vize-ssr",
  ]);
  assert.equal(
    toNuxtVueTransformId("/app/pages/index.vue?vue=&vize&used=true"),
    "/app/pages/index.vue?vue&type=script&vize&used=true",
  );
  // Negative control: a style block keeps its own identity, which the gate refuses.
  assert.equal(plugin.transform("x", "/app/Foo/Index.vue?vue&type=style&index=0"), null);
});

void test("#11592 Nuxt's navigateTo early return also sees Vize modules", () => {
  const calls: string[] = [];
  const plugin = {
    name: "nuxt:navigate-to-early-return",
    transform: {
      filter: {
        id: { include: VUE_SCRIPT_ID_FILTER, exclude: /node_modules\// },
        code: { include: /withAsyncContext/ },
      },
      handler(_code: string, id: string) {
        calls.push(id);
        return null;
      },
    },
  };
  patchNuxtVueTypeGatedTransform(plugin);
  const vizeId = "/app/pages/index.vue?vue&vize";
  assert.equal(hostCalls(plugin.transform.filter, vizeId, "withAsyncContext()"), true);
  plugin.transform.handler("withAsyncContext()", vizeId);
  assert.deepEqual(calls, ["/app/pages/index.vue?vue&type=script&vize"]);
  // Negative control: the host's node_modules exclusion still applies.
  assert.equal(
    hostCalls(plugin.transform.filter, "/node_modules/x/A.vue?vue&vize", "withAsyncContext()"),
    false,
  );
});

// Nuxt 4.6.4 SFC source transforms (dist/index.mjs), ordered before the compiler.
const VUE_ID_FILTER = [/\.vue$/, /[?&]macro=/, /[?&]vue(?:&|$)/];
const VUE_TEMPLATE_ID_FILTER = [/\.vue$/, /\?macro=true$/, /[?&]vue&type=template\b/];
const DEVONLY_COMP_SINGLE_RE =
  /<(?:dev-only|DevOnly|lazy-dev-only|LazyDevOnly)(?:\s(?:[^>"']|"[^"]*"|'[^']*')*)?>[\s\S]*?<\/(?:dev-only|DevOnly|lazy-dev-only|LazyDevOnly)>/;
const TEMPLATE_WITH_LAZY_HYDRATION_RE =
  /<template>[\s\S]*\bhydrate-on-visible\b[\s\S]*<\/template>/;

void test("#11641 Nuxt's DevOnly strip reads the whole-file identity of a Vize module", () => {
  const calls: string[] = [];
  const plugin = {
    name: "nuxt:server-devonly:transform",
    enforce: "pre",
    transform: {
      filter: {
        id: { include: VUE_TEMPLATE_ID_FILTER },
        code: { include: DEVONLY_COMP_SINGLE_RE },
      },
      handler(_code: string, id: string) {
        calls.push(id);
        return null;
      },
    },
  };
  const source = "<template><DevOnly><p>dev</p></DevOnly></template>";
  const vizeId = "/app/pages/index.vue?vue&vize";
  assert.equal(hostCalls(plugin.transform.filter, vizeId, source), false);

  patchNuxtVueTypeGatedTransform(plugin);
  const { filter } = plugin.transform;
  for (const id of [
    vizeId,
    "/app/pages/index.vue?vue&vize-ssr",
    "/app/pages/index.vue?vue&vize&used",
  ]) {
    assert.equal(hostCalls(filter, id, source), true, id);
    plugin.transform.handler(source, id);
  }
  // @vitejs/plugin-vue hands a whole-SFC source transform the file itself.
  assert.deepEqual(calls, ["/app/pages/index.vue", "/app/pages/index.vue", "/app/pages/index.vue"]);
  // Negative controls: the host's own refusals stand.
  assert.equal(hostCalls(filter, vizeId, "<template><p>prod</p></template>"), false);
  assert.equal(hostCalls(filter, "/app/pages/index.vue?vue&type=style&index=0", source), false);
  assert.equal(hostCalls(filter, "/app/pages/index.vue?vue&type=script&vize", source), false);
  assert.equal(hostCalls(filter, "/app/pages/index.vue?raw", source), false);
});

void test("#11641 a function-form DevOnly gate also reads the whole-file identity", () => {
  const isNuxtVueTemplate = (id: string) => VUE_TEMPLATE_ID_FILTER.some((re) => re.test(id));
  const plugin = {
    name: "nuxt:server-devonly:transform",
    transform(code: string, id: string) {
      return isNuxtVueTemplate(id) ? code.replace(DEVONLY_COMP_SINGLE_RE, "") : null;
    },
  };
  patchNuxtVueTypeGatedTransform(plugin);
  const source = "<template><main><DevOnly><p>dev</p></DevOnly></main></template>";
  assert.equal(
    plugin.transform(source, "/app/A.vue?vue&vize"),
    "<template><main></main></template>",
  );
  assert.equal(plugin.transform(source, "/app/A.vue?vue&type=style&index=0"), null);
});

void test("#11641 Nuxt's isVue-gated SFC source transforms admit Vize modules unpatched", () => {
  // `nuxt:components-loader-pre` (lazy hydration) and
  // `nuxt:server-only-component-transform` (islands) gate on `VUE_ID_FILTER`.
  const source = "<template><LazyCard hydrate-on-visible /></template>";
  for (const id of ["/app/A.vue?vue&vize", "/app/A.vue?vue&vize-ssr"]) {
    assert.equal(
      hostCalls(
        { id: { include: VUE_ID_FILTER }, code: TEMPLATE_WITH_LAZY_HYDRATION_RE },
        id,
        source,
      ),
      true,
      id,
    );
  }
  for (const name of ["nuxt:components-loader-pre", "nuxt:server-only-component-transform"]) {
    const handler = (_code: string, id: string) => id;
    const plugin = { name, transform: { filter: { id: { include: VUE_ID_FILTER } }, handler } };
    patchNuxtVueTypeGatedTransform(plugin);
    assert.equal(plugin.transform.handler, handler, name);
  }
  // Negative control: a style block is still not an SFC source.
  assert.equal(
    hostCalls(
      { id: { include: VUE_ID_FILTER }, code: TEMPLATE_WITH_LAZY_HYDRATION_RE },
      "/app/A.vue?vue&type=style&index=0",
      "p{}",
    ),
    false,
  );
});
