import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { injectNuxtComponentImports, type NuxtComponentImport } from "./components.ts";

const directory = new URL(
  "../../../../tests/_fixtures/differential/compiler/vuefes-app-component-order/",
  import.meta.url,
);
const corpus = JSON.parse(fs.readFileSync(new URL("corpus.json", directory), "utf8"));
const source = fs.readFileSync(new URL(corpus.input.file, directory), "utf8");
const resolve = (components: Record<string, NuxtComponentImport>) => (name: string) =>
  components[name] ?? null;

void test("whole pinned VueFes app keeps global CSS before explicit Nuxt components", () => {
  assert.equal(Buffer.byteLength(source), corpus.input.bytes);
  assert.equal(createHash("sha256").update(source).digest("hex"), corpus.input.sha256);
  const script = source.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)?.[1];
  assert(script, "the entire original authored script must be present");
  const transformed = injectNuxtComponentImports(
    script,
    resolve({
      NuxtPage: { filePath: "nuxt/dist/pages/runtime/page.js", exportName: "default" },
      NuxtRouteAnnouncer: {
        filePath: "nuxt/dist/app/components/nuxt-route-announcer.js",
        exportName: "default",
        mode: "client",
      },
    }),
  );
  assert.equal(
    transformed,
    script.replace(
      'import { NuxtPage, NuxtRouteAnnouncer } from "#components";',
      'import NuxtPage from "nuxt/dist/pages/runtime/page.js";\n' +
        'import NuxtRouteAnnouncer from "nuxt/dist/app/components/nuxt-route-announcer.js";',
    ),
  );
  assert(
    transformed.indexOf('import "~/assets/styles/main.css";') <
      transformed.indexOf("import NuxtPage from"),
  );
});

void test("explicit aliases and leftover types retain their position; implicit bindings prepend", () => {
  const source =
    'import "main.css";\n' +
    'import { Foo as Bar, type Shape, Unknown } from "#components";\n' +
    'const layout = resolveComponent("NuxtLayout");\nconst use = [Bar, Unknown];\n';
  const transformed = injectNuxtComponentImports(
    source,
    resolve({
      Foo: { filePath: "/components/Foo.vue", exportName: "default" },
      NuxtLayout: { filePath: "/nuxt/nuxt-layout.js", exportName: "default" },
    }),
  );
  assert.equal(
    transformed,
    'import __nuxt_component_0 from "/nuxt/nuxt-layout.js";\n' +
      'import "main.css";\nimport Bar from "/components/Foo.vue";\n' +
      'import { type Shape, Unknown } from "#components";\n' +
      "const layout = __nuxt_component_0;\nconst use = [Bar, Unknown];\n",
  );
});

void test("explicit client and lazy bindings keep their helpers and initialize before use", () => {
  const source =
    'import "main.css";\n' +
    'import { Client, LazyClient } from "#components";\nconst use = [Client, LazyClient];\n';
  const transformed = injectNuxtComponentImports(
    source,
    resolve({
      Client: { filePath: "/components/Client.vue", exportName: "named", mode: "client" },
      LazyClient: {
        filePath: "/components/LazyClient.vue",
        exportName: "default",
        mode: "client",
        lazy: true,
      },
    }),
  );
  assert.match(
    transformed,
    /^import \{ defineAsyncComponent as __nuxt_define_async_component \} from "vue";/,
  );
  assert.match(
    transformed,
    /import \{ createClientOnly as __nuxt_create_client_only \} from "#app\/components\/client-only";/,
  );
  assert(
    transformed.indexOf('import "main.css";') <
      transformed.indexOf("import { named as __nuxt_import_component_0_raw }"),
  );
  assert.match(
    transformed,
    /const Client = __nuxt_create_client_only\(__nuxt_import_component_0_raw\);/,
  );
  assert.match(
    transformed,
    /const LazyClient = __nuxt_define_async_component\(\(\) => import\("\/components\/LazyClient\.vue"\)\.then\(\(module\) => __nuxt_create_client_only\(module\.default\)\)\);/,
  );
  assert(transformed.indexOf("const LazyClient =") < transformed.indexOf("const use ="));
});
