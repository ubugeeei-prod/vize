import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { optimizeDeps, resolveConfig } from "vite";

import { vize } from "./index.ts";

const root = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), "vize-dep-scan-"));

function write(relativePath: string, content: string): void {
  const filePath = path.join(root, relativePath);
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content);
}

try {
  write("index.html", '<script type="module" src="/src/main.ts"></script>');
  write("src/main.ts", 'import Counter from "./Counter.vue"; console.log(Counter);');
  write(
    "src/Counter.vue",
    '<script setup lang="ts">import mitt from "mitt"; import Child from "./Child.vue"; const emitter = mitt();</script><template><Child @click="emitter.emit(\'click\')" /></template>',
  );
  write(
    "src/Child.vue",
    '<script setup>import nestedOnly from "nested-only"; console.log(nestedOnly);</script><template><button>child</button></template>',
  );
  for (const name of ["mitt", "nested-only"]) {
    write(
      `node_modules/${name}/package.json`,
      JSON.stringify({ name, version: "1.0.0", type: "module", exports: "./index.js" }),
    );
    write(`node_modules/${name}/index.js`, "export default () => ({});\n");
  }

  const config = await resolveConfig(
    {
      root,
      configFile: false,
      cacheDir: path.join(root, "node_modules", ".vite"),
      logLevel: "silent",
      plugins: vize({ configMode: false }),
      optimizeDeps: { entries: ["index.html"] },
    },
    "serve",
  );
  const metadata = await optimizeDeps(config);
  assert.ok(metadata.optimized.mitt, "scan should pre-bundle an import used only by an SFC");
  assert.ok(
    metadata.optimized["nested-only"],
    "scan should crawl nested SFCs to find their bare imports",
  );
} finally {
  fs.rmSync(root, { recursive: true, force: true });
}

console.log("vite-plugin-vize dependency scan tests passed!");
