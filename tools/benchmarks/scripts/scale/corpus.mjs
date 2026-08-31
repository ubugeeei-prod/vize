/**
 * Corpus generator for the large-scale Vue build benchmark.
 *
 * Shape is copied from `rolldown/benchmarks` `apps/{1000,3000,5000,10000}`:
 * a `src/` tree where each directory holds `FILES_PER_DIR` components and each
 * component imports the components of its own child directory, so the module
 * graph is a deep tree rather than a flat fan-out from the entry.
 *
 * Deliberate differences from the reference, and why:
 *
 * - Components are Vue SFCs with `<script setup lang="ts">`, scoped styles,
 *   CSS Modules, and real directives instead of React JSX. Vize compiles SFCs;
 *   a JSX corpus would measure the bundler, not Vize.
 * - The third-party half of the module graph (the reference uses
 *   `@iconify-icons/*` + `@iconify/react`, 1.4k-9k node_modules modules) is
 *   generated as a local `node_modules/@tools/ui` package instead of a real npm
 *   dependency. The benchmark must run from a clean checkout without mutating
 *   `pnpm-lock.yaml`, and what the reference is actually exercising is graph
 *   size: many small bare-specifier modules, one distinct module per component.
 *   `vue` itself is a real dependency and is bundled, not externalized.
 * - No `rome`/`three10x` cases: those are TypeScript/JS-only bundling shapes
 *   with no Vue surface.
 */

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

import { renderComponent } from "./component.mjs";

const FILES_PER_DIR = 9;

/**
 * Assign `total` components to a directory tree, breadth-first.
 *
 * Returns a map of directory (posix, relative to `src`) to the number of files
 * in it. `""` is `src` itself.
 */
export function planTree(total) {
  const dirFileCounts = new Map();
  const queue = [""];
  let created = 0;

  while (created < total && queue.length > 0) {
    const dir = queue.shift();
    const count = Math.min(FILES_PER_DIR, total - created);
    dirFileCounts.set(dir, count);
    created += count;
    for (let index = 0; index < count; index++) {
      queue.push(dir === "" ? `d${index}` : `${dir}/d${index}`);
    }
  }

  return dirFileCounts;
}

function componentPaths(dirFileCounts) {
  const paths = [];
  for (const [dir, count] of dirFileCounts) {
    for (let index = 0; index < count; index++) {
      paths.push({ dir, file: `f${index}.vue` });
    }
  }
  return paths;
}

function writeVendorPackage(appDir, iconCount) {
  const pkgDir = join(appDir, "node_modules", "@bench", "ui");
  mkdirSync(join(pkgDir, "icons"), { recursive: true });
  mkdirSync(join(pkgDir, "internal"), { recursive: true });

  writeFileSync(
    join(pkgDir, "package.json"),
    `${JSON.stringify(
      {
        name: "@tools/ui",
        version: "1.0.0",
        type: "module",
        main: "./index.js",
        exports: {
          ".": "./index.js",
          "./icons/*": "./icons/*",
        },
      },
      null,
      2,
    )}\n`,
  );

  for (let index = 0; index < 4; index++) {
    writeFileSync(
      join(pkgDir, "internal", `part${index}.js`),
      `export const SALT_${index} = ${index * 7 + 3}\nexport function mix${index}(value) {\n  return (value * SALT_${index}) % 97\n}\n`,
    );
  }

  writeFileSync(
    join(pkgDir, "index.js"),
    `import { SALT_0, mix0 } from './internal/part0.js'
import { mix1 } from './internal/part1.js'
import { mix2 } from './internal/part2.js'
import { mix3 } from './internal/part3.js'

export function formatScore(score) {
  return (mix0(score) + mix1(score)).toFixed(1)
}

export function makeRows(token, count) {
  return Array.from({ length: count }, (_, index) => ({
    id: token + '-' + index,
    label: 'row-' + token + '-' + index,
    score: mix2(index + SALT_0),
  }))
}

export function clampWeight(value) {
  return Math.max(1, Math.min(9, mix3(value) % 10))
}

export function listEntries(token, count) {
  return Array.from({ length: count }, (_, index) => 'entry-' + token + '-' + index)
}

export function escapeMarkup(markup) {
  return markup.replace(/&/g, '&amp;').replace(/</g, '&lt;')
}
`,
  );

  for (let index = 0; index < iconCount; index++) {
    writeFileSync(
      join(pkgDir, "icons", `i${index}.js`),
      `export default {\n  name: 'icon-${index}',\n  width: 24,\n  height: 24,\n  body: '<path d="M${index % 24} 0h24v24H0z" />',\n}\n`,
    );
  }

  return iconCount + 5;
}

/**
 * Write the shared asset every component references.
 *
 * Deliberately larger than Vite's 4 KB `assetsInlineLimit` so it is emitted as
 * a real file with a hashed name instead of being inlined as a data URI. An
 * inlined asset would not exercise asset-URL rewriting in templates at all,
 * which is one of the things minification is most likely to break.
 */
function writeAssets(appDir) {
  const assetsDir = join(appDir, "assets");
  mkdirSync(assetsDir, { recursive: true });
  const rings = [];
  for (let index = 0; index < 220; index++) {
    rings.push(
      `<circle cx="12" cy="12" r="${(index % 10) + 1}" fill="none" stroke="#42b883" stroke-width="0.1" opacity="0.${index % 10}" />`,
    );
  }
  writeFileSync(
    join(assetsDir, "logo.svg"),
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">${rings.join("")}</svg>\n`,
  );
}

function writeEntry(appDir, rootCount) {
  const srcDir = join(appDir, "src");
  const imports = [];
  const names = [];
  for (let index = 0; index < rootCount; index++) {
    const name = `Root${index}`;
    imports.push(`import ${name} from './f${index}.vue'`);
    names.push(name);
  }

  writeFileSync(
    join(srcDir, "index.ts"),
    `import { createApp, h } from 'vue'
import './index.css'
${imports.join("\n")}

const app = createApp({
  render() {
    return h('div', { class: 'bench-root' }, [${names.map((name) => `h(${name})`).join(", ")}])
  },
})
app.mount('#app')
`,
  );

  // The generated app is real TypeScript, so `vize check` / `vue-tsc` can be
  // pointed at the same corpus without 2 errors per file for the untyped
  // vendor package.
  writeFileSync(
    join(srcDir, "env.d.ts"),
    `declare module '@tools/ui' {
  export function formatScore(score: number): string
  export function makeRows(token: string, count: number): { id: string; label: string; score: number }[]
  export function clampWeight(value: number): number
  export function listEntries(token: string, count: number): string[]
  export function escapeMarkup(markup: string): string
}
declare module '@tools/ui/icons/*' {
  const icon: { name: string; width: number; height: number; body: string }
  export default icon
}
declare module '*.svg' {
  const url: string
  export default url
}
`,
  );

  writeFileSync(
    join(srcDir, "index.css"),
    `:root { --bench-gap: 8px; }
.bench-root { display: grid; gap: var(--bench-gap); font-family: system-ui, sans-serif; }
.bench-root button { cursor: pointer; }
`,
  );

  writeFileSync(
    join(appDir, "index.html"),
    `<!doctype html>
<html>
  <head><meta charset="utf-8" /><title>vize scale bench</title></head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/index.ts"></script>
  </body>
</html>
`,
  );
}

/**
 * Generate one app.
 *
 * @returns `{ dir, componentCount, vendorModuleCount }`
 */
export function generateApp({ appDir, componentCount, clean = true }) {
  if (clean) {
    rmSync(appDir, { recursive: true, force: true });
  }
  mkdirSync(join(appDir, "src"), { recursive: true });

  const dirFileCounts = planTree(componentCount);
  const paths = componentPaths(dirFileCounts);

  paths.forEach(({ dir, file }, index) => {
    const id = String(index).padStart(5, "0");
    const fileIndex = Number.parseInt(file.slice(1), 10);
    const childDir = dir === "" ? `d${fileIndex}` : `${dir}/d${fileIndex}`;
    const childCount = dirFileCounts.get(childDir) ?? 0;
    const children = [];
    for (let childIndex = 0; childIndex < childCount; childIndex++) {
      children.push({
        name: `C${childIndex}`,
        specifier: `./d${fileIndex}/f${childIndex}.vue`,
      });
    }

    const target = join(appDir, "src", dir, file);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, renderComponent(index, id, children, `@tools/ui/icons/i${index}.js`));
  });

  writeAssets(appDir);
  writeEntry(appDir, dirFileCounts.get("") ?? 0);
  const vendorModuleCount = writeVendorPackage(appDir, componentCount);

  return { dir: appDir, componentCount, vendorModuleCount };
}
