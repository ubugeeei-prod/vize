import assert from "node:assert/strict";
import test from "node:test";

import {
  planVizeLibraryIntegration,
  setupVizeLibraries,
  type ComposableResolverModule,
  type UiResolverModule,
  type VizeLibraryLoaders,
} from "./libraries.ts";

// The real catalog-driven resolvers, loaded from the workspace sources.
const loaders: VizeLibraryLoaders = {
  // Computed specifiers keep the sibling packages out of this package's type program.
  ui: async () =>
    (await import(
      new URL("../../../ui/src/resolver/resolver.ts", import.meta.url).href
    )) as UiResolverModule,
  composables: async () =>
    (await import(
      new URL("../../../compose/core/src/resolver.ts", import.meta.url).href
    )) as ComposableResolverModule,
};

void test("does nothing (and loads nothing) unless opted in", async () => {
  let loaded = false;
  const plan = await setupVizeLibraries({}, "/app", async () => {
    loaded = true;
    return { addComponent: () => undefined, addImports: () => undefined };
  });
  assert.deepEqual(plan, { components: [], imports: [] });
  assert.equal(loaded, false);
});

void test("plans every ui component as a prefixed direct-subpath registration", async () => {
  const plan = await planVizeLibraryIntegration({ ui: { prefix: "Vz" } }, "/app", loaders);
  const ui = await loaders.ui();
  assert.equal(plan.components.length, ui.listVizeUiComponents({}).length);
  assert.deepEqual(
    plan.components.find((component) => component.export === "DialogTrigger"),
    { name: "VzDialogTrigger", export: "DialogTrigger", filePath: "@vizejs/ui/dialog" },
  );
  assert.ok(plan.components.every((component) => component.filePath.startsWith("@vizejs/ui/")));
  assert.deepEqual(
    plan.imports.find((item) => item.name === "useSafeAreaInsets"),
    { name: "useSafeAreaInsets", from: "@vizejs/ui/safe-area" },
  );
});

void test("family include/exclude and composables filters narrow registrations", async () => {
  const plan = await planVizeLibraryIntegration(
    {
      ui: { include: ["switch", "dialog"], exclude: ["dialog"], composables: false },
      composables: { include: ["use-toggle", "use-counter"] },
    },
    "/app",
    loaders,
  );
  assert.deepEqual(
    plan.components.map((component) => component.name),
    ["Switch"],
  );
  assert.deepEqual(plan.imports, [
    { name: "useCounter", from: "@vizejs/composable/use-counter" },
    { name: "useToggle", from: "@vizejs/composable/use-toggle" },
  ]);
});

void test("registers components and imports through Nuxt kit and keeps the first owner of a name", async () => {
  const components: unknown[] = [];
  const imports: unknown[] = [];
  const plan = await setupVizeLibraries(
    { ui: { include: ["locale"] }, composables: true },
    "/app",
    async () => ({
      addComponent: (component) => components.push(component),
      addImports: (items) => imports.push(...items),
    }),
    loaders,
  );
  assert.equal(components.length, plan.components.length);
  assert.equal(imports.length, plan.imports.length);
  const names = plan.imports.map((item) => item.name);
  assert.equal(new Set(names).size, names.length, "one registration per name");
  assert.deepEqual(
    plan.imports.find((item) => item.name === "useLocale"),
    { name: "useLocale", from: "@vizejs/ui/locale" },
    "ui helpers win over same-named composables",
  );
  assert.ok(plan.imports.some((item) => item.from === "@vizejs/composable/use-toggle"));
});

void test("missing libraries fail with an actionable message", async () => {
  const { createProjectLibraryLoaders } = await import("./libraries.ts");
  const project = createProjectLibraryLoaders("/nonexistent-project");
  await assert.rejects(project.ui(), /`vize\.ui` needs @vizejs\/ui installed/);
  await assert.rejects(project.composables(), /`vize\.composables` needs @vizejs\/composable/);
});
