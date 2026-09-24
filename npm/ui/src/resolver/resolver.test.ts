import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { test } from "vite-plus/test";

import {
  buildUiResolverManifest,
  renderUiResolverManifest,
} from "../../scripts/resolver-manifest.ts";
import { uiFamilyCatalog } from "../catalog/family-catalog.ts";
import {
  UI_RESOLVER_COMPONENTS,
  UI_RESOLVER_COMPOSABLES,
  UI_RESOLVER_ENTRIES,
  VizeUiComposablesResolver,
  VizeUiResolver,
  createVizeUiComponentDeclarations,
  listVizeUiComponents,
  readVizeLibLockfile,
  vizeUiImports,
} from "./resolver.ts";

const packageRoot = path.resolve(".");

test("the generated manifest is in sync with the family catalog", () => {
  const expected = buildUiResolverManifest(packageRoot);
  const hint = "run `pnpm generate:resolver` after catalog changes";
  assert.deepEqual({ ...UI_RESOLVER_COMPONENTS }, expected.components, hint);
  assert.deepEqual({ ...UI_RESOLVER_COMPOSABLES }, expected.composables, hint);
  assert.deepEqual({ ...UI_RESOLVER_ENTRIES }, expected.entries, hint);
  assert.match(renderUiResolverManifest(expected), /export const UI_RESOLVER_COMPONENTS = \{/);
});

test("every catalogued SFC family contributes components that its package subpath exports", async () => {
  const manifest = JSON.parse(readFileSync(path.join(packageRoot, "package.json"), "utf8")) as {
    exports: Record<string, unknown>;
  };
  const families = new Set(Object.values(UI_RESOLVER_COMPONENTS));
  for (const entry of uiFamilyCatalog) {
    // Families whose entry publishes an SFC (some ship `.vue` examples that are not exported).
    if (!readFileSync(path.join(packageRoot, entry.entryFile), "utf8").includes('.vue"')) continue;
    assert.ok(
      families.has(entry.packageSubpath.slice(2)),
      `${entry.canonicalName} has no resolvable component`,
    );
  }
  for (const family of new Set([...families, ...Object.values(UI_RESOLVER_COMPOSABLES)])) {
    assert.ok(manifest.exports[`./${family}`], `./${family} must be a package export`);
  }
  // Spot-check real exports through the source entries (aliases, cross-family re-exports).
  const drawer = await import("../families/overlays/drawer/drawer.ts");
  assert.ok("DrawerTrigger" in drawer);
  assert.equal(UI_RESOLVER_COMPONENTS.DrawerTrigger, "drawer");
  assert.equal(
    UI_RESOLVER_COMPONENTS.DialogTrigger,
    "dialog",
    "the owning family wins over re-exports",
  );
  assert.equal(UI_RESOLVER_COMPONENTS.ActionSheetCancel, "action-sheet");
  assert.equal(UI_RESOLVER_COMPONENTS.TabBar, "bottom-navigation");
  assert.equal(UI_RESOLVER_COMPOSABLES.useSafeAreaInsets, "safe-area");
});

test("the component resolver maps tags to direct subpath imports with prefixes and filters", () => {
  const plain = VizeUiResolver();
  assert.equal(plain.type, "component");
  assert.deepEqual(plain.resolve("DialogTrigger"), {
    name: "DialogTrigger",
    from: "@vizejs/ui/dialog",
  });
  assert.deepEqual(plain.resolve("dialog-trigger"), {
    name: "DialogTrigger",
    from: "@vizejs/ui/dialog",
  });
  assert.equal(plain.resolve("RouterLink"), undefined);

  const prefixed = VizeUiResolver({ prefix: "Vz", exclude: ["tooltip"] });
  assert.deepEqual(prefixed.resolve("VzButton"), {
    name: "Button",
    as: "VzButton",
    from: "@vizejs/ui/button",
  });
  assert.equal(prefixed.resolve("Button"), undefined, "unprefixed tags are left alone");
  assert.equal(prefixed.resolve("VzTooltipRoot"), undefined, "excluded families never resolve");

  const only = VizeUiResolver({ include: ["switch"] });
  assert.ok(only.resolve("Switch"));
  assert.equal(only.resolve("Button"), undefined);
});

test("composables resolve for auto-import as a resolver and as an imports preset", () => {
  const resolve = VizeUiComposablesResolver({ exclude: ["locale"] });
  assert.deepEqual(resolve("useFieldWiring"), {
    name: "useFieldWiring",
    from: "@vizejs/ui/field-wiring",
  });
  assert.equal(resolve("useLocale"), undefined);
  assert.equal(resolve("useUnknown"), undefined);
  const imports = vizeUiImports({ include: ["safe-area"] });
  assert.deepEqual(imports, { "@vizejs/ui/safe-area": ["useSafeAreaInsets"] });
});

test("local mode resolves pulled families from vize-lib.lock.json and falls back for the rest", () => {
  const root = mkdtempSync(path.join(tmpdir(), "vize-ui-resolver-"));
  writeFileSync(
    path.join(root, "vize-lib.lock.json"),
    JSON.stringify({
      lockfileVersion: 1,
      items: [
        {
          name: "button",
          kind: "ui",
          dir: "src/components/vize",
          package: "@vizejs/ui",
          version: "1.0.0",
        },
        { name: "use-toggle", kind: "composable", dir: "src/composables/vize" },
      ],
    }),
  );
  const local = VizeUiResolver({ source: "local", root });
  assert.deepEqual(local.resolve("Button"), {
    name: "Button",
    from: `${root.split(path.sep).join("/")}/src/components/vize/families/actions/button/button.ts`,
  });
  assert.deepEqual(local.resolve("Switch"), { name: "Switch", from: "@vizejs/ui/switch" });
  const strict = VizeUiResolver({
    source: "local",
    fallback: false,
    lockfile: { lockfileVersion: 1, items: [{ name: "switch", kind: "ui", dir: "lib/ui" }] },
    root: "/project",
  });
  assert.deepEqual(strict.resolve("Switch"), {
    name: "Switch",
    from: "/project/lib/ui/families/selection/switch/switch.ts",
  });
  assert.equal(
    strict.resolve("Button"),
    undefined,
    "without fallback, unpulled families stay unresolved",
  );

  writeFileSync(path.join(root, "broken.json"), "{}");
  assert.throws(
    () => readVizeLibLockfile(path.join(root, "broken.json")),
    /VIZE_UI_RESOLVER_LOCKFILE/,
  );
  assert.throws(
    () => VizeUiResolver({ source: "local", root, lockfile: "missing.json" }),
    /VIZE_UI_RESOLVER_LOCKFILE/,
  );
});

test("generated GlobalComponents declarations reference each component's subpath", () => {
  const declarations = createVizeUiComponentDeclarations({
    prefix: "Vz",
    include: ["switch", "dialog"],
  });
  assert.match(declarations, /^\/\/ Generated by @vizejs\/ui\/resolver/);
  assert.match(declarations, /declare module "vue" \{\n {2}export interface GlobalComponents \{/);
  assert.match(declarations, / {4}VzSwitch: typeof import\("@vizejs\/ui\/switch"\)\["Switch"\];/);
  assert.match(
    declarations,
    / {4}VzDialogTrigger: typeof import\("@vizejs\/ui\/dialog"\)\["DialogTrigger"\];/,
  );
  assert.doesNotMatch(declarations, /Button/);
  assert.equal(listVizeUiComponents().length, Object.keys(UI_RESOLVER_COMPONENTS).length);
});
