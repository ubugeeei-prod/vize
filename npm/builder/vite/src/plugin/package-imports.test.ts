import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { resolveIdHook } from "./resolve.ts";
import { toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";

const testRoot = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), "vize-package-imports-"));

function writeFixtureFile(filePath: string, content = ""): void {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

function createState(root: string): VizePluginState {
  return {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root,
    clientViteBase: "/",
    serverViteBase: "/",
    server: {} as never,
    filter: () => true,
    scanPatterns: ["**/*.vue"],
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {},
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: {
      log() {},
      info() {},
      warn() {},
      error() {},
    } as never,
  };
}

function expectResolvedId(resolved: Awaited<ReturnType<typeof resolveIdHook>>): string {
  assert.notEqual(resolved, null);
  assert.notEqual(resolved, undefined);
  return typeof resolved === "string" ? resolved : resolved.id;
}

{
  const root = fs.mkdtempSync(path.join(testRoot, "extensionless-"));
  const source = path.join(root, "app", "pages", "index.vue");
  const resolvedType = path.join(root, "types", "index.ts");
  writeFixtureFile(
    path.join(root, "package.json"),
    JSON.stringify({
      private: true,
      imports: {
        "#types/*": "./types/*",
      },
    }),
  );
  writeFixtureFile(
    source,
    "<script setup lang=\"ts\">import type { User } from '#types/index'</script>",
  );
  writeFixtureFile(resolvedType, "export interface User { name: string }\n");

  let resolverImporter: string | undefined;
  const resolved = await resolveIdHook(
    {
      resolve: async (_id: string, importer?: string) => {
        resolverImporter = importer;
        throw new Error('Missing "#types/index" specifier in "nuxt" package');
      },
    },
    createState(root),
    "#types/index",
    toPluginVisibleVirtualId(source),
    undefined,
  );

  assert.equal(resolverImporter, source);
  assert.equal(
    expectResolvedId(resolved),
    resolvedType,
    "Package imports from plugin-visible Vize SFC modules should resolve against the source package",
  );
}

{
  const root = fs.mkdtempSync(path.join(testRoot, "virtual-"));
  const source = path.join(root, "app", "components", "Profile.vue");
  const resolvedRuntime = path.join(root, "runtime", "profile.ts");
  writeFixtureFile(
    path.join(root, "package.json"),
    JSON.stringify({
      private: true,
      imports: {
        "#runtime/*": {
          import: "./runtime/*",
        },
      },
    }),
  );
  writeFixtureFile(source, "<script setup>import profile from '#runtime/profile'</script>");
  writeFixtureFile(resolvedRuntime, "export default {};\n");

  const resolved = await resolveIdHook(
    { resolve: async () => null },
    createState(root),
    "#runtime/profile",
    toVirtualId(source),
    undefined,
  );

  assert.equal(
    expectResolvedId(resolved),
    resolvedRuntime,
    "Package imports from Rollup-internal Vize SFC modules should use the original source package",
  );
}
