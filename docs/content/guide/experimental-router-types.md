---
title: Experimental Router Types
description: Opt in to published Vue Router experimental types across checking and editor features.
---

# Experimental Router Types

Vize recognizes the project-local typed-router opt-in used by Vue Router. Keep
the router's generated declarations and the opt-in in the same TypeScript project
for `vize check`, content mappers, and the Vize language server.

This recipe uses **`vue-router@5.4.0`**. Experimental APIs can change between
versions. The [upstream guide](https://router.vuejs.org/experimental/router-resolver)
can describe changes that are not in this published package.

## Generate route declarations

Configure Vue Router's file scanner before the Vue compiler plugin:

```ts
// vite.config.ts
import { defineConfig } from "vite";
import VueRouter from "vue-router/vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    VueRouter({
      routesFolder: "src/pages",
      dts: "typed-router.d.ts",
      experimental: { paramParsers: { dir: "src/params" } },
    }),
    vize(),
  ],
});
```

Start Vite once to generate `typed-router.d.ts`. For CI that checks before
starting Vite, the same published scanner can generate it explicitly:

```js
// scripts/generate-routes.mjs
import { createRoutesContext, resolveOptions } from "vue-router/unplugin";

const context = createRoutesContext(
  resolveOptions({
    root: process.cwd(),
    routesFolder: "src/pages",
    dts: "typed-router.d.ts",
    experimental: { paramParsers: { dir: "src/params" } },
  }),
);
try {
  await context.scanPages(false);
  context.writeConfigFiles();
} finally {
  context.stopWatcher();
}
```

```bash
node scripts/generate-routes.mjs
```

Commit the generated declaration or generate it before checking. Regenerate it
when page paths or parser declarations change. Vize consumes the real declaration;
it does not replace the router's scanner.

## Opt in per project

Add the following to the actual project containing your pages. Preserve other
`compilerOptions`, `types`, and plugins already in that project:

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "rootDir": ".",
    "types": ["vue-router/auto-routes"]
  },
  "vueCompilerOptions": {
    "plugins": ["vue-router/volar/sfc-typed-router"]
  },
  "include": ["typed-router.d.ts", "src/**/*.ts", "src/**/*.vue"]
}
```

The plugin name enables Vize's built-in typed-router projection. It does not run
arbitrary Volar plugins. Removing it disables that projection. A monorepo can
enable it in one project without enabling another project.

This published-version recipe uses `skipLibCheck: true` for the router's
experimental declaration compatibility. Authored scripts and templates are still
checked.

## Register the experimental router type

```ts
// src/router/index.ts
import { createWebHistory } from "vue-router";
import { experimental_createRouter as createRouter } from "vue-router/experimental";
import { resolver } from "vue-router/auto-resolver";

export const router = createRouter({ history: createWebHistory(), resolver });

declare module "vue-router" {
  interface TypesConfig {
    Router: typeof router;
  }
}
```

Register `RouterLink` and `RouterView` from `vue-router` on your Vue app when using
the experimental router, as described in the upstream setup guide.

## Use parsed values

Custom parser modules export `parser`. For example:

```ts
// src/params/date.ts
import { defineParamParser, miss } from "vue-router/experimental";

export const parser = defineParamParser<Date>({
  get(value) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) miss();
    return date;
  },
  set: (value) => value.toISOString(),
});
```

```vue
<!-- src/pages/products/[id=int].vue -->
<script setup lang="ts">
import { useRoute } from "vue-router";
import { definePage } from "vue-router/experimental";

definePage({
  params: {
    query: {
      page: { parser: "int", format: "value", default: 1 },
      at: { parser: "date", format: "value" },
    },
  },
});

const route = useRoute();
const id: number = route.params.id;
const page: number = route.params.page;
const at: Date | undefined = route.params.at;
</script>

<template>{{ id.toFixed() }} / {{ page.toFixed() }} / {{ at?.getFullYear() }}</template>
```

In published 5.4.0, import `useRoute` and `useRouter` from `vue-router`, and
`definePage` and parser helpers from `vue-router/experimental`.

## Use the same selected project

```bash
vize check --tsconfig tsconfig.app.json
tsc --runExternalCode --noEmit -p tsconfig.app.json
```

The second command additionally needs the
[content mapper configuration](./content-mapper.md) and a mapper-capable compiler.
Both commands must select the project that includes `typed-router.d.ts` and the
plugin opt-in above. For the language server, open the same project and enable
type checking with the recommended profile. If the project is not `tsconfig.json`,
set `vue.typeChecker.tsconfig` in your Vite settings to that same file; see
[Configuration](./configuration-reference.md#static-analysis-options).

The VS Code and Art extensions use this language-server project configuration;
there is no separate experimental-router setting in the extensions.

## Published-version limits

The 5.4.0 package does not export `_RoutePropsForFilePath` and does not provide the
new automatic `props: true` augmentation of `defineProps()`. Do not copy an
unpublished helper declaration into your project to enable it. Newer upstream
contracts require their own provider version and compatibility validation.

Support for experimental route types in additional script and template languages
is being qualified separately. The setup above covers Vue SFCs with TypeScript
scripts and HTML templates; it does not promise parity for every preprocessor.
