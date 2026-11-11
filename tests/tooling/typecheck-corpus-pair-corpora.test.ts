import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import {
  BASE_PACKAGES,
  CORPORA,
  packageClosure,
} from "../../tools/benchmarks/scripts/typecheck-corpus-pair-corpora.ts";
import {
  appConfigTypesTemplate,
  kebabCase,
  themeTemplate,
  uiTypesTemplate,
} from "../../tools/benchmarks/scripts/typecheck-corpus-nuxt-ui-theme.ts";
import { root } from "./support/github-workflows.ts";

void test("misskey and n8n resolve Vue and Node types; nuxt-ui maps #build to its generated .nuxt", () => {
  const byId = Object.fromEntries(CORPORA.map((corpus) => [corpus.id, corpus]));
  assert.deepEqual(byId.misskey!.packages, BASE_PACKAGES);
  assert.deepEqual(byId.n8n!.packages, BASE_PACKAGES);
  const nuxt = byId["nuxt-ui"]!;
  assert.ok("generate" in nuxt && nuxt.generate.endsWith("typecheck-corpus-nuxt-ui-theme.ts"));
  assert.deepEqual(nuxt.tsconfig.compilerOptions.paths, {
    "#build/*": ["./.nuxt/*"],
    "#imports": ["./src/runtime/vue/stubs/vue-router.ts"],
  });
  assert.ok(nuxt.tsconfig.include.includes("./.nuxt/types/*.d.ts"));
});

const lockfile = () =>
  JSON.parse(
    readFileSync(join(root, "tools/benchmarks/typecheck-corpora/package-lock.json"), "utf8"),
  ) as Parameters<typeof packageClosure>[0];

void test("each corpus resolves only its own pinned package closure", () => {
  const lock = lockfile();
  const closure = (id: string) =>
    packageClosure(lock, CORPORA.find((corpus) => corpus.id === id)!.packages);
  // misskey and n8n keep exactly the Vue and Node types they always resolved.
  const base = closure("misskey");
  assert.deepEqual(closure("n8n"), base);
  assert.ok(base.includes("node_modules/vue") && base.includes("node_modules/undici-types"));
  for (const name of ["reka-ui", "@vueuse/core", "tailwind-variants", "vue-router"])
    assert.ok(!base.includes(`node_modules/${name}`), name);
  // reka-ui gets its own dependencies, never the published reka-ui build.
  const reka = closure("reka-ui");
  for (const name of ["@vueuse/core", "@floating-ui/vue", "@internationalized/date", "vue-demi"])
    assert.ok(reka.includes(`node_modules/${name}`), name);
  assert.ok(!reka.includes("node_modules/reka-ui"));
  // nuxt-ui resolves reka-ui, tv() and @nuxt/schema's AppConfig, and vaul-vue's own @vueuse/core.
  const nuxt = closure("nuxt-ui");
  for (const name of [
    "reka-ui",
    "tailwind-variants",
    "tailwind-merge",
    "tailwindcss",
    "@nuxt/schema",
  ])
    assert.ok(nuxt.includes(`node_modules/${name}`), name);
  assert.ok(nuxt.includes("node_modules/vaul-vue/node_modules/@vueuse/core"));
  // Every closure entry is an installed lockfile path.
  for (const path of [...base, ...reka, ...nuxt]) assert.ok(lock.packages[path], path);
});

void test("the package closure resolves nested copies as Node does and skips absent optional peers", () => {
  const lock = {
    packages: {
      "": { dependencies: { a: "1", b: "1" } },
      "node_modules/a": { dependencies: { c: "2" }, peerDependencies: { typescript: "*" } },
      "node_modules/a/node_modules/c": { dependencies: { d: "1" } },
      "node_modules/b": { dependencies: { c: "1" } },
      "node_modules/c": {},
      "node_modules/d": {},
    },
  };
  assert.deepEqual(packageClosure(lock, ["a"]), [
    "node_modules/a",
    "node_modules/a/node_modules/c",
    "node_modules/d",
  ]);
  assert.deepEqual(packageClosure(lock, ["b"]), ["node_modules/b", "node_modules/c"]);
  assert.throws(() => packageClosure(lock, ["e"]), /does not install e/u);
  assert.throws(
    () => packageClosure({ packages: { "node_modules/x": { dependencies: { y: "1" } } } }, ["x"]),
    /dependency y is not installed/u,
  );
});

void test("nuxt-ui theme templates keep tv() variant keys literal, as the module writes them", () => {
  const text = themeTemplate({
    slots: { base: "inline-flex" },
    variants: {
      color: { primary: "", neutral: "" },
      block: { true: { base: "w-full" } },
    },
    compoundVariants: [{ color: "primary", class: "x" }],
    defaultVariants: { color: "primary" },
  });
  assert.match(
    text,
    /^const color = \[\n {2}"primary",\n {2}"neutral"\n\] as const\n\nexport default \{/u,
  );
  assert.doesNotMatch(text, /const block/u);
  assert.match(text, /"color": "primary" as typeof color\[number\]/u);
  assert.equal(kebabCase("contentSearchButton"), "content-search-button");
  assert.equal(kebabCase("pageCTA"), "page-cta");
  const ui = uiTypesTemplate(["primary", "secondary"], ["close", "check"]);
  assert.match(ui, /type IconsConfig = Record<"close" \| "check" \| \(string & \{\}\), string>/u);
  assert.match(ui, /'primary'\?: Color\n\t\t'secondary'\?: Color/u);
  assert.match(ui, /interface CustomAppConfig \{\n {4}ui: AppConfigRuntimeUI/u);
  const appConfig = appConfigTypesTemplate({ nuxt: {}, ui: { colors: { primary: "green" } } });
  assert.match(appConfig, /declare const inlineConfig = \{\n {2}"nuxt": \{\},/u);
  assert.match(
    appConfig,
    /interface AppConfig extends MergedAppConfig<ResolvedAppConfig, CustomAppConfig>/u,
  );
});
