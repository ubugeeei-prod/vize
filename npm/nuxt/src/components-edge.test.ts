import assert from "node:assert/strict";
import test from "node:test";

import {
  createNuxtComponentResolver,
  injectNuxtComponentImports,
  type NuxtComponentImport,
} from "./components.ts";

// All injection tests below use a hand-written resolve stub so they are entirely
// filesystem-free: injectNuxtComponentImports never touches disk, it only calls
// back into the supplied resolver. The descriptor shape is exactly the
// NuxtComponentImport interface declared in components.ts.
function stubResolver(
  table: Record<string, NuxtComponentImport>,
): (name: string) => NuxtComponentImport | null {
  return (name) => table[name] ?? null;
}

void test("injection leaves code without component usage unchanged", () => {
  const code = "export const x = 1;\nconst y = x + 1;\n";
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  const transformed = injectNuxtComponentImports(code, resolve);

  assert.equal(
    transformed,
    code,
    "code with no resolveComponent() and no #components import should be returned verbatim",
  );
});

void test("injection replaces literal resolveComponent() but not a dynamic argument", () => {
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  const literal = injectNuxtComponentImports(`const c = resolveComponent("Foo");`, resolve);
  assert.match(
    literal,
    /import __nuxt_component_0 from "\/virtual\/Foo\.vue";/,
    "literal-string resolveComponent should become a direct import binding",
  );
  assert.equal(
    literal.includes('resolveComponent("Foo")'),
    false,
    "the resolveComponent() call should be replaced by the bound variable",
  );
  assert.match(literal, /const c = __nuxt_component_0;/, "the call site should use the binding");

  // A non-literal argument cannot be statically resolved, so it is left intact.
  const dynamicSource = `const c = resolveComponent(dynamicVar);`;
  const dynamic = injectNuxtComponentImports(dynamicSource, resolve);
  assert.equal(
    dynamic,
    dynamicSource,
    "resolveComponent(<identifier>) with a non-literal argument should be left untouched",
  );
});

void test("injection resolves _resolveComponent and a second-argument call", () => {
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  // The compiler often emits the private _resolveComponent helper name.
  const underscored = injectNuxtComponentImports(`const c = _resolveComponent("Foo");`, resolve);
  assert.match(
    underscored,
    /import __nuxt_component_0 from "\/virtual\/Foo\.vue";/,
    "_resolveComponent should be rewritten the same as resolveComponent",
  );

  // resolveComponent("Foo", maybeSelfReference) is also matched.
  const withSecondArg = injectNuxtComponentImports(
    `const c = resolveComponent("Foo", true);`,
    resolve,
  );
  assert.match(
    withSecondArg,
    /import __nuxt_component_0 from "\/virtual\/Foo\.vue";/,
    "resolveComponent() with a trailing argument should still be rewritten",
  );
  assert.equal(
    withSecondArg.includes("resolveComponent"),
    false,
    "the entire resolveComponent(...) call (including the second argument) should be replaced",
  );
});

void test("injection resolves a named (non-default) export via aliased import", () => {
  const resolve = stubResolver({
    Named: { exportName: "MyNamed", filePath: "/virtual/Named.js" },
  });

  const transformed = injectNuxtComponentImports(`const c = resolveComponent("Named");`, resolve);
  assert.match(
    transformed,
    /import \{ MyNamed as __nuxt_component_0 \} from "\/virtual\/Named\.js";/,
    "a non-default exportName should be imported as a named binding aliased to the component var",
  );
});

void test("injection dedupes repeated resolveComponent() of the same component", () => {
  const resolve = stubResolver({
    AppHeader: { exportName: "default", filePath: "/virtual/AppHeader.vue" },
  });

  const transformed = injectNuxtComponentImports(
    `const first = resolveComponent("AppHeader");\n` +
      `const second = resolveComponent("AppHeader");\n` +
      `const third = resolveComponent("AppHeader");\n`,
    resolve,
  );

  assert.equal(
    transformed.match(/import __nuxt_component_0 from "\/virtual\/AppHeader\.vue";/g)?.length,
    1,
    "a reused component should emit exactly one import statement",
  );
  assert.equal(
    transformed.match(/__nuxt_component_0\b/g)?.length,
    4,
    "the shared binding name appears once in the single import plus at each of the three call sites",
  );
});

void test("injection wraps a .client component with createClientOnly", () => {
  const resolve = stubResolver({
    Widget: {
      exportName: "default",
      filePath: "/virtual/Widget.client.vue",
      mode: "client",
    },
  });

  const transformed = injectNuxtComponentImports(`const c = resolveComponent("Widget");`, resolve);

  assert.match(
    transformed,
    /import \{ createClientOnly as __nuxt_create_client_only \} from "#app\/components\/client-only";/,
    "client-mode components should import the createClientOnly helper",
  );
  assert.match(
    transformed,
    /import __nuxt_component_0_raw from "\/virtual\/Widget\.client\.vue";/,
    "the raw default export should be imported under a _raw binding",
  );
  assert.match(
    transformed,
    /const __nuxt_component_0 = __nuxt_create_client_only\(__nuxt_component_0_raw\);/,
    "the bound component variable should be the wrapped client-only component",
  );
});

void test("injection does NOT wrap a NuxtRouteAnnouncer-style .js client component", () => {
  // Mode is client, but the file matches the nuxt-route-announcer special case,
  // so it must be imported directly to preserve its own scoped default slot.
  const resolve = stubResolver({
    NuxtRouteAnnouncer: {
      exportName: "default",
      filePath: "/pkg/nuxt/dist/app/components/nuxt-route-announcer.js",
      mode: "client",
    },
  });

  const transformed = injectNuxtComponentImports(
    `const c = resolveComponent("NuxtRouteAnnouncer");`,
    resolve,
  );

  assert.match(
    transformed,
    /import __nuxt_component_0 from ".*nuxt-route-announcer\.js";/,
    "NuxtRouteAnnouncer should be imported directly",
  );
  assert.equal(
    transformed.includes("__nuxt_create_client_only"),
    false,
    "NuxtRouteAnnouncer must not be wrapped with createClientOnly even though mode is client",
  );
});

void test("injection wraps a lazy component with defineAsyncComponent", () => {
  const resolve = stubResolver({
    Lazy: { exportName: "default", filePath: "/virtual/Lazy.vue", lazy: true },
  });

  const transformed = injectNuxtComponentImports(`const c = resolveComponent("Lazy");`, resolve);

  assert.match(
    transformed,
    /import \{ defineAsyncComponent as __nuxt_define_async_component \} from "vue";/,
    "lazy components should import the defineAsyncComponent helper",
  );
  assert.match(
    transformed,
    /const __nuxt_component_0 = __nuxt_define_async_component\(\(\) => import\("\/virtual\/Lazy\.vue"\)\.then\(\(module\) => module\.default\)\);/,
    "lazy resolution should defer the import and read the default export from the module",
  );
  assert.equal(
    transformed.includes("__nuxt_create_client_only"),
    false,
    "a lazy non-client component should not pull in the createClientOnly helper",
  );
});

void test("injection wraps a lazy client component with both async and client helpers", () => {
  const resolve = stubResolver({
    LazyWidget: {
      exportName: "default",
      filePath: "/virtual/Widget.client.vue",
      lazy: true,
      mode: "client",
    },
  });

  const transformed = injectNuxtComponentImports(
    `const c = resolveComponent("LazyWidget");`,
    resolve,
  );

  assert.match(
    transformed,
    /import \{ defineAsyncComponent as __nuxt_define_async_component \} from "vue";/,
    "a lazy client component should import defineAsyncComponent",
  );
  assert.match(
    transformed,
    /import \{ createClientOnly as __nuxt_create_client_only \} from "#app\/components\/client-only";/,
    "a lazy client component should import createClientOnly",
  );
  assert.match(
    transformed,
    /const __nuxt_component_0 = __nuxt_define_async_component\(\(\) => import\("\/virtual\/Widget\.client\.vue"\)\.then\(\(module\) => __nuxt_create_client_only\(module\.default\)\)\);/,
    "the async payload should be wrapped with createClientOnly inside the .then()",
  );
});

void test("injection rewrites import { Foo } from #components to a direct import", () => {
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  const transformed = injectNuxtComponentImports(
    `import { Foo } from "#components";\nconst use = Foo;\n`,
    resolve,
  );

  assert.match(
    transformed,
    /import Foo from "\/virtual\/Foo\.vue";/,
    "a resolved #components import should become a direct default import keeping the local name",
  );
  assert.equal(
    transformed.includes('from "#components"'),
    false,
    "when every imported component resolves, the #components import should be fully removed",
  );
});

void test("injection rewrites an aliased #components specifier (Foo as Bar)", () => {
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  const transformed = injectNuxtComponentImports(
    `import { Foo as Bar } from "#components";\nconst use = Bar;\n`,
    resolve,
  );

  assert.match(
    transformed,
    /import Bar from "\/virtual\/Foo\.vue";/,
    "the local alias (Bar) should be preserved as the direct import binding",
  );
  assert.equal(
    transformed.includes('"#components"'),
    false,
    "the fully-resolved aliased import should be removed",
  );
});

void test("injection leaves an empty #components import untouched", () => {
  const source = `import {} from "#components";\nconst z = 1;\n`;
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  const transformed = injectNuxtComponentImports(source, resolve);

  assert.equal(
    transformed,
    source,
    "an empty #components import has no specifiers to resolve, so nothing changes",
  );
});

void test("injection leaves a type-only #components import untouched", () => {
  const source = `import type { Foo } from "#components";\nconst z = 1;\n`;
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  const transformed = injectNuxtComponentImports(source, resolve);

  assert.equal(
    transformed,
    source,
    "import type { ... } from #components is excluded by the import regex and stays as-is",
  );
});

void test("injection keeps an all-unresolved #components import unchanged", () => {
  const source = `import { Unknown } from "#components";\nconst z = 1;\n`;
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  const transformed = injectNuxtComponentImports(source, resolve);

  assert.equal(
    transformed,
    source,
    "if no specifier resolves, the #components import (and the whole module) is returned verbatim",
  );
});

void test("injection partially rewrites a mixed #components import, keeping the rest", () => {
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  const transformed = injectNuxtComponentImports(
    `import { Foo, Unknown } from "#components";\nconst z = 1;\n`,
    resolve,
  );

  // Resolved specifier becomes a direct import emitted in the preamble.
  assert.match(
    transformed,
    /import Foo from "\/virtual\/Foo\.vue";/,
    "the resolved specifier should be lifted to a direct import",
  );
  // Unresolved specifier is re-emitted as a (regenerated) #components import.
  assert.match(
    transformed,
    /import \{ Unknown \} from "#components";/,
    "the unresolved specifier should remain a #components import",
  );
  // The regenerated leftover import always uses double quotes for #components,
  // and the lifted direct import precedes the leftover import.
  assert.ok(
    transformed.indexOf('import Foo from "/virtual/Foo.vue";') <
      transformed.indexOf('from "#components"'),
    "the lifted direct import should precede the leftover #components import",
  );
});

void test("injection re-emits a single-quoted #components import with double quotes", () => {
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
  });

  const transformed = injectNuxtComponentImports(
    `import { Foo, Unknown } from '#components';\nconst z = 1;\n`,
    resolve,
  );

  assert.match(
    transformed,
    /import \{ Unknown \} from "#components";/,
    "the regenerated leftover import normalizes the quote style to double quotes",
  );
  assert.equal(
    transformed.includes("'#components'"),
    false,
    "the original single-quoted #components import should no longer be present",
  );
});

void test("injection handles an inline-type specifier mixed with a value specifier", () => {
  const resolve = stubResolver({
    Foo: { exportName: "default", filePath: "/virtual/Foo.vue" },
    Named: { exportName: "MyNamed", filePath: "/virtual/Named.js" },
  });

  const transformed = injectNuxtComponentImports(
    `import { type Foo, Named } from "#components";\nconst z = 1;\n`,
    resolve,
  );

  // `type Foo` is treated as type-only and left in #components; `Named` is lifted.
  assert.match(
    transformed,
    /import \{ MyNamed as Named \} from "\/virtual\/Named\.js";/,
    "the value specifier should be lifted to a direct named import",
  );
  assert.match(
    transformed,
    /import \{ type Foo \} from "#components";/,
    "the inline type-only specifier should remain in the #components import",
  );
});

void test("createNuxtComponentResolver register/resolve works without any filesystem", () => {
  // buildDir/rootDir point at a path that does not exist; loadDtsComponents and
  // loadRuntimeComponents simply find nothing, so register()/resolve() of a
  // directly-registered component never touches disk.
  const resolver = createNuxtComponentResolver({
    buildDir: "/nonexistent-vize-test/.nuxt",
    rootDir: "/nonexistent-vize-test",
  });

  resolver.register([
    { pascalName: "AppHeader", filePath: "/virtual/AppHeader.vue", export: "default" },
    {
      pascalName: "MyWidget",
      filePath: "/virtual/MyWidget.client.vue",
      export: "default",
      mode: "client",
    },
    { pascalName: "Detected", filePath: "/virtual/Detected.client.vue", export: "default" },
  ]);

  assert.deepEqual(
    resolver.resolve("AppHeader"),
    { exportName: "default", filePath: "/virtual/AppHeader.vue" },
    "a registered component resolves by its pascal name",
  );
  assert.deepEqual(
    resolver.resolve("app-header"),
    { exportName: "default", filePath: "/virtual/AppHeader.vue" },
    "register() should add a kebab-case alias",
  );
  assert.deepEqual(
    resolver.resolve("LazyAppHeader"),
    { exportName: "default", filePath: "/virtual/AppHeader.vue", lazy: true },
    "register() should add a Lazy-prefixed alias flagged lazy",
  );
  assert.deepEqual(
    resolver.resolve("MyWidget"),
    { exportName: "default", filePath: "/virtual/MyWidget.client.vue", mode: "client" },
    "an explicit client mode should be preserved on the registered import",
  );
  assert.deepEqual(
    resolver.resolve("Detected"),
    { exportName: "default", filePath: "/virtual/Detected.client.vue", mode: "client" },
    "a .client.vue filePath should auto-detect client mode even without an explicit mode",
  );

  // Unknown lowercase names short-circuit to null before any filesystem fallback.
  assert.equal(
    resolver.resolve("zz-unknown"),
    null,
    "an unknown lowercase name should resolve to null without a runtime/dts lookup",
  );
  // Unknown uppercase names trigger the runtime fallback, which fails to require
  // from the nonexistent rootDir and returns null (it must not throw).
  assert.equal(
    resolver.resolve("ZZUnknownComp"),
    null,
    "an unknown uppercase name should resolve to null when no runtime package is found",
  );
});
