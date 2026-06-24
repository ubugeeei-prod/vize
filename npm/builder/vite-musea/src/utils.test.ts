import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  resolveScanRoots,
  rewriteStorybookComponentImport,
  rewriteStorybookScriptImports,
  scanArtFiles,
} from "./utils.ts";
import { loadNative } from "./native-loader.ts";

void test("resolveScanRoots preserves include bases outside the Vite root", () => {
  const root = "/workspace/apps/website";
  const roots = resolveScanRoots(root, ["../../packages/ui/src/**/*.art.vue"]);

  assert.deepEqual(roots, ["/workspace/packages/ui/src"]);
});

void test("scanArtFiles discovers art files outside the Vite root when include points upward", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-scan-"));
  const root = path.join(tempDir, "apps", "website");
  const externalDir = path.join(tempDir, "packages", "ui", "src");
  const artFile = path.join(externalDir, "MfButton.art.vue");

  await fs.promises.mkdir(root, { recursive: true });
  await fs.promises.mkdir(externalDir, { recursive: true });
  await fs.promises.writeFile(artFile, "<art><template><div /></template></art>\n", "utf-8");

  const files = await scanArtFiles(root, ["../../packages/ui/src/**/*.art.vue"], [], false);

  assert.deepEqual(files, [artFile]);

  await fs.promises.rm(tempDir, { recursive: true, force: true });
});

void test("rewriteStorybookComponentImport rebases component path from story output", () => {
  const root = "/workspace";
  const artPath = path.join(root, "src", "AfsButton.art.vue");
  const outputPath = path.join(root, ".storybook", "stories", "AfsButton.stories.ts");
  const code =
    "import type { Meta } from '@storybook/vue3';\nimport __museaComponent from './AfsButton.vue';\n";

  const result = rewriteStorybookComponentImport(
    code,
    {
      path: artPath,
      metadata: {
        title: "AfsButton",
        component: "./AfsButton.vue",
        tags: [],
        status: "ready",
      },
      variants: [],
      hasScriptSetup: false,
      hasScript: false,
      styleCount: 0,
    },
    artPath,
    outputPath,
  );

  assert.match(result, /from '\.\.\/\.\.\/src\/AfsButton\.vue';/);
});

void test("rewriteStorybookScriptImports rebases relative imports lifted from script setup", () => {
  const root = "/workspace";
  const artPath = path.join(root, "src", "AfsButton.art.vue");
  const outputPath = path.join(root, ".storybook", "generated", "AfsButton.stories.ts");
  const code = [
    "import type { Meta, StoryObj } from '@storybook/vue3';",
    "import __museaComponent from '../../src/AfsButton.vue';",
    "import AfsButton from './AfsButton.vue';",
    'import { fixture } from "./fixtures";',
    "",
  ].join("\n");

  const result = rewriteStorybookScriptImports(code, artPath, outputPath);

  // `__museaComponent` is already correct — must be preserved.
  assert.match(result, /import __museaComponent from '\.\.\/\.\.\/src\/AfsButton\.vue';/);
  // Lifted script-setup imports must be rebased relative to the generated file.
  assert.match(result, /import AfsButton from '\.\.\/\.\.\/src\/AfsButton\.vue';/);
  assert.match(result, /import \{ fixture \} from "\.\.\/\.\.\/src\/fixtures";/);
  // Bare specifiers (npm packages) must be left untouched.
  assert.match(result, /import type \{ Meta, StoryObj \} from '@storybook\/vue3';/);
  // The stale, broken specifier must no longer appear.
  assert.doesNotMatch(result, /from '\.\/AfsButton\.vue';/);
});

void test("CSF output for an art file rewrites script-setup imports relative to the generated file (issue #2228)", () => {
  const binding = loadNative();
  const root = "/workspace";
  const artPath = path.join(root, "src", "AfsButton.art.vue");
  const outputPath = path.join(root, ".storybook", "generated", "AfsButton.stories.ts");

  const source = `
<script setup lang="ts">
import AfsButton from './AfsButton.vue'
</script>

<art title="AfsButton" category="Components" component="./AfsButton.vue">
  <variant name="Primary" default>
    <AfsButton color="primary">Primary</AfsButton>
  </variant>
</art>
`;

  const csf = binding.artToCsf(source, { filename: artPath });

  const art = {
    path: artPath,
    metadata: {
      title: "AfsButton",
      component: "./AfsButton.vue",
      tags: ["button", "action"],
      status: "ready" as const,
    },
    variants: [],
    hasScriptSetup: true,
    hasScript: false,
    styleCount: 0,
  };

  const rewritten = rewriteStorybookComponentImport(csf.code, art, artPath, outputPath);
  const code = rewriteStorybookScriptImports(rewritten, artPath, outputPath);

  // The stale `./AfsButton.vue` lifted from script-setup must be gone.
  assert.doesNotMatch(code, /from ['"]\.\/AfsButton\.vue['"]/);
  // The lifted import must be rebased relative to the generated file's location.
  assert.match(code, /import AfsButton from ['"]\.\.\/\.\.\/src\/AfsButton\.vue['"]/);
});
