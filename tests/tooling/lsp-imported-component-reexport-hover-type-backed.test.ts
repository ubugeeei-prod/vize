import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { requireTypecheckDependency } from "./support/typecheck-dependency.ts";

const appSource = `<script setup lang="ts">
import { BarrelChild, StarChild } from './components'
import { PackageChild as UiChild } from '@fixture/ui'

BarrelChild
StarChild
UiChild
</script>

<template>
  <BarrelChild :label="'barrel'" />
  <StarChild mode="wide" />
  <UiChild tone="info" />
</template>
`;

const barrelChildSource = `<script setup lang="ts">
defineProps<{ label: string }>()
defineEmits<{ save: [value: string] }>()
defineSlots<{ default(props: { value: string }): unknown }>()
defineModel<boolean>()
</script>
<template><span>{{ label }}</span></template>
`;

const starChildSource = `<script setup lang="ts">
defineProps<{ mode: 'wide' | 'narrow' }>()
</script>
<template><span>{{ mode }}</span></template>
`;

const packageChildSource = `<script setup lang="ts">
defineProps<{ tone: 'info' | 'warn' }>()
defineEmits<{ choose: [tone: 'info' | 'warn'] }>()
defineSlots<{ default(props: { tone: string }): unknown }>()
defineModel<string>('query')
</script>
<template><span>{{ tone }}</span></template>
`;

test("script hover describes re-exported and package SFC component contracts", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTsgoBinary(),
    "tsgo binary for re-exported imported SFC hover",
    "tsgo binary not found; skipping re-exported imported SFC hover test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-imported-component-reexport-hover");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();
  let initialized = false;

  try {
    const srcDir = path.join(workspaceDir, "src");
    const componentsDir = path.join(srcDir, "components");
    const packageDir = path.join(workspaceDir, "node_modules/@fixture/ui");
    fs.mkdirSync(componentsDir, { recursive: true });
    fs.mkdirSync(packageDir, { recursive: true });
    linkVuePackage(workspaceDir);
    fs.writeFileSync(
      path.join(workspaceDir, "vize.config.json"),
      JSON.stringify({
        lsp: { hover: true, lint: false, typecheck: true },
        typeChecker: { corsaPath },
      }),
      "utf8",
    );
    fs.writeFileSync(
      path.join(workspaceDir, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          lib: ["ES2022", "DOM", "DOM.Iterable"],
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          skipLibCheck: true,
          strict: true,
          target: "ES2022",
        },
        include: ["src/**/*.vue"],
      }),
      "utf8",
    );

    const appPath = path.join(srcDir, "App.vue");
    fs.writeFileSync(appPath, appSource, "utf8");
    fs.writeFileSync(path.join(componentsDir, "BarrelChild.vue"), barrelChildSource, "utf8");
    fs.writeFileSync(path.join(componentsDir, "StarChild.vue"), starChildSource, "utf8");
    fs.writeFileSync(
      path.join(componentsDir, "index.ts"),
      [
        "export { default as BarrelChild } from './BarrelChild.vue'",
        "export * from './nested'",
      ].join("\n") + "\n",
      "utf8",
    );
    fs.writeFileSync(
      path.join(componentsDir, "nested.ts"),
      "export { default as StarChild } from './StarChild.vue'\n",
      "utf8",
    );
    fs.writeFileSync(path.join(packageDir, "PackageChild.vue"), packageChildSource, "utf8");
    fs.writeFileSync(
      path.join(packageDir, "package.json"),
      JSON.stringify({ name: "@fixture/ui", exports: { ".": "./index.ts" } }),
      "utf8",
    );
    fs.writeFileSync(
      path.join(packageDir, "index.ts"),
      "export { default as PackageChild } from './PackageChild.vue'\n",
      "utf8",
    );

    const uri = pathToFileURL(appPath).href;
    await session.initialize(workspaceDir, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: true,
    });
    initialized = true;
    session.notify("textDocument/didOpen", {
      textDocument: { languageId: "vue", text: appSource, uri, version: 1 },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      120_000,
    );

    assertComponentHover(
      await hoverTextAt(session, uri, "BarrelChild,"),
      ["const BarrelChild: VueComponent", "props: { label: string };"],
      ["emits: { save: [value: string] };", "Vue component: BarrelChild.vue"],
    );
    assertComponentHover(
      await hoverTextAt(session, uri, "\nStarChild\n"),
      ["const StarChild: VueComponent", "props: { mode: 'wide' | 'narrow' };"],
      ["Vue component: StarChild.vue"],
    );
    assertComponentHover(
      await hoverTextAt(session, uri, "UiChild }"),
      ["const UiChild: VueComponent", "props: { tone: 'info' | 'warn' };"],
      [
        "emits: { choose: [tone: 'info' | 'warn'] };",
        "slots: { default(props: { tone: string }): unknown };",
        'model: "query": string;',
        "Vue component: PackageChild.vue",
      ],
    );
  } finally {
    if (initialized) {
      await session.shutdown();
    } else {
      await session.kill().catch(() => undefined);
    }
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

async function hoverTextAt(session: LspSession, uri: string, marker: string): Promise<string> {
  let offset = appSource.indexOf(marker);
  assert.ok(offset >= 0, `missing marker ${marker}`);
  if (marker.startsWith("\n")) {
    offset += 1;
  }
  const hover = await session.request(
    "textDocument/hover",
    {
      position: offsetToPosition(appSource, offset),
      textDocument: { uri },
    },
    120_000,
  );
  return hoverToText(hover as Parameters<typeof hoverToText>[0]);
}

function assertComponentHover(text: string, required: string[], extra: string[]): void {
  assert.match(text, /^```typescript\n/);
  for (const value of [...required, ...extra]) {
    assert.ok(text.includes(value), `missing ${value} in ${text}`);
  }
  assert.doesNotMatch(text, /__vizeComponentMarker|__vizeRawProps|__VizeComponentConstructor/);
}

function resolveTsgoBinary(): string | undefined {
  const candidates = [
    process.env.CORSA_BIN,
    path.join(root, "../corsa-bind/.cache/tsgo"),
    path.join(root, "node_modules/.bin/tsgo"),
    path.join(root, "tests/node_modules/.bin/tsgo"),
  ].filter((candidate): candidate is string => candidate != null && candidate.length > 0);
  return candidates.find((candidate) => fs.existsSync(candidate));
}

function linkVuePackage(workspaceDir: string): void {
  const vuePackage = [
    path.join(root, "node_modules/vue"),
    path.join(root, "tests/node_modules/vue"),
  ].find((candidate) => fs.existsSync(candidate));
  assert.ok(vuePackage, "Vue package is required for re-exported imported component hover test");
  const nodeModules = path.join(workspaceDir, "node_modules");
  fs.mkdirSync(nodeModules, { recursive: true });
  fs.symlinkSync(vuePackage, path.join(nodeModules, "vue"), dirLinkType());
  const vueNamespace = path.join(path.dirname(vuePackage), "@vue");
  if (fs.existsSync(vueNamespace)) {
    fs.symlinkSync(vueNamespace, path.join(nodeModules, "@vue"), dirLinkType());
  }
}

function dirLinkType(): fs.symlink.Type {
  return process.platform === "win32" ? "junction" : "dir";
}
