import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { test } from "vite-plus/test";

import { loadConfig } from "../src/config.ts";
import { importConfigModule } from "../src/config/module-runtime.mjs";
import { resolveViteConfigExport } from "../src/config/vite-runtime.mjs";

const require = createRequire(import.meta.url);
const vite = path.dirname(require.resolve("vite/package.json"));
const native = require("@vizejs/native") as { normalizeVizeConfig(value: unknown): unknown };
const original = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../../tests/_fixtures/differential/config/public-vite-module-8371/project.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as { files: Record<string, string> };

async function project(run: (root: string) => Promise<void>, withVite = true): Promise<void> {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-config-")));
  try {
    for (const [name, source] of Object.entries(original.files)) {
      fs.writeFileSync(path.join(root, name), source);
    }
    if (withVite) {
      fs.mkdirSync(path.join(root, "node_modules"));
      fs.symlinkSync(vite, path.join(root, "node_modules/vite"), "junction");
    }
    await run(root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

test("source and packed public APIs match native on the complete unchanged helper-import project", async () => {
  await project(async (root) => {
    const selected = path.join(root, "vite.config.ts");
    const module = await importConfigModule(selected);
    const expected = native.normalizeVizeConfig(
      await resolveViteConfigExport(module.default, undefined, root),
    );
    const packed = await import("../dist/config.mjs");
    assert.deepEqual(await loadConfig(root), expected);
    assert.deepEqual(await packed.loadConfig(root), expected);
    for (const [name, source] of Object.entries(original.files)) {
      assert.equal(fs.readFileSync(path.join(root, name), "utf8"), source);
    }
    assert.equal(
      fs.readdirSync(root).some((name) => name.startsWith("vize.config.")),
      false,
    );
  });
});

test("the selected project Vite resolves JSON, extensionless TS and JS-to-TS imports with file scope", async () => {
  await project(async (root) => {
    fs.mkdirSync(path.join(root, "tools"));
    fs.writeFileSync(path.join(root, "metadata.json"), '{"width":80}');
    fs.writeFileSync(path.join(root, "tools/width.ts"), "export const width: number = 10;");
    fs.writeFileSync(path.join(root, "tools/scope.ts"), "export const scope = __dirname;");
    const selected = path.join(root, "vite.config.ts");
    fs.writeFileSync(
      selected,
      `import pkg from './metadata.json';
      import {width} from './tools/width.js'; import {scope} from './tools/scope';
      if (scope !== ${JSON.stringify(path.join(root, "tools"))} || __dirname !== ${JSON.stringify(root)} || import.meta.url !== ${JSON.stringify(pathToFileURL(selected).href)}) throw new Error('file scope changed');
      export default {vize:{formatter:{printWidth:pkg.width+width}}};`,
    );
    const result = await loadConfig(root);
    assert.equal(result?.formatter.printWidth, 90);
  });
});

test("Vite+ source and module evaluate once with the caller's complete environment", async () => {
  await project(async (root) => {
    const counter = path.join(root, "evaluations.json");
    const selected = path.join(root, "vite.config.ts");
    fs.writeFileSync(
      selected,
      `import {existsSync,readFileSync,writeFileSync} from 'node:fs';
      const counter=${JSON.stringify(counter)};
      const counts=existsSync(counter)?JSON.parse(readFileSync(counter,'utf8')):{module:0,source:0};
      counts.module++; writeFileSync(counter, JSON.stringify(counts));
      const wrapper=()=>{throw new Error('Vite+ wrapper executed');};
      wrapper[Symbol.for('@vizejs/vite-plugin/vite-plus/source')]=async(env)=>{
        const counts=JSON.parse(readFileSync(counter,'utf8')); counts.source++;
        writeFileSync(counter,JSON.stringify(counts));
        if (env.command!=='serve'||env.mode!=='production'||env.isSsrBuild!==true) throw new Error('environment changed');
        return {vize:{formatter:{printWidth:91}}};
      }; export default wrapper;`,
    );
    const env = Object.freeze({ command: "fmt", mode: "production", isSsrBuild: true });
    assert.equal((await loadConfig(root, { env }))?.formatter.printWidth, 91);
    assert.deepEqual(JSON.parse(fs.readFileSync(counter, "utf8")), { module: 1, source: 1 });
    assert.deepEqual(env, { command: "fmt", mode: "production", isSsrBuild: true });
  });
});

test("dedicated precedence and explicit selection preserve their distinct factory environments", async () => {
  await project(async (root) => {
    fs.writeFileSync(
      path.join(root, "vize.config.ts"),
      `export default ({command,mode})=>{
      if(command!=='lint'||mode!=='production') throw new Error('dedicated environment changed');
      return {formatter:{printWidth:99}};
    };`,
    );
    const env = { command: "lint", mode: "production" };
    assert.equal((await loadConfig(root, { env }))?.formatter.printWidth, 99);
    assert.equal(
      (await loadConfig(root, { env, configFile: "vite.config.ts" }))?.formatter.singleQuote,
      true,
    );
    fs.writeFileSync(path.join(root, "vize.config.ts"), "throw new Error('dedicated refuses');");
    await assert.rejects(loadConfig(root, { env }), /dedicated refuses/);
  });
});

test("without project Vite the existing TypeScript transform and fresh direct module loading remain", async () => {
  await project(async (root) => {
    fs.writeFileSync(
      path.join(root, "vite.config.ts"),
      "export default {vize:{formatter:{printWidth:90}}} satisfies Record<string,unknown>;",
    );
    assert.equal((await loadConfig(root))?.formatter.printWidth, 90);
    fs.writeFileSync(
      path.join(root, "vite.config.ts"),
      "export default {vize:{formatter:{printWidth:91}}} satisfies Record<string,unknown>;",
    );
    assert.equal((await loadConfig(root))?.formatter.printWidth, 91);
    fs.rmSync(path.join(root, "vite.config.ts"));
    fs.writeFileSync(
      path.join(root, "vite.config.mjs"),
      "export default {vize:{formatter:{printWidth:92}}};",
    );
    assert.equal((await loadConfig(root))?.formatter.printWidth, 92);
  }, false);
});

test("selected config and dependency failures never fall through to a sibling config", async () => {
  for (const source of [
    "import value from './missing'; export default {vize:value};",
    "import value from './broken.json'; export default {vize:value};",
    "throw new Error('selected factory refuses');",
    "export default ()=>{throw new Error('selected factory refuses');};",
  ]) {
    await project(async (root) => {
      fs.writeFileSync(path.join(root, "vite.config.ts"), source);
      fs.writeFileSync(path.join(root, "broken.json"), "{broken");
      fs.writeFileSync(path.join(root, "vite.config.mjs"), "throw new Error('sibling selected');");
      await assert.rejects(loadConfig(root), (error: Error) => {
        assert.doesNotMatch(error.message, /sibling selected/);
        return true;
      });
    });
  }
});

test("the selected project Vite is authoritative even when the public API has another Vite installed", async () => {
  await project(async (root) => {
    fs.mkdirSync(path.join(root, "node_modules/vite"), { recursive: true });
    fs.writeFileSync(
      path.join(root, "node_modules/vite/package.json"),
      '{"name":"vite","type":"module","exports":"./index.mjs"}',
    );
    fs.writeFileSync(
      path.join(root, "node_modules/vite/index.mjs"),
      "export function loadConfigFromFile(){throw new Error('selected project Vite refuses');}",
    );
    assert.equal(await loadConfig(root, { mode: "none" }), null);
    await assert.rejects(loadConfig(root), /selected project Vite refuses/);
    fs.writeFileSync(path.join(root, "node_modules/vite/package.json"), "{broken");
    await assert.rejects(loadConfig(root));
  }, false);
});

test("a present empty nearest Vite cannot select the otherwise legitimate hoisted loader", async () => {
  await project(async (root) => {
    const nested = path.join(root, "nested");
    fs.mkdirSync(path.join(nested, "node_modules/vite"), { recursive: true });
    for (const [name, source] of Object.entries(original.files)) {
      fs.writeFileSync(path.join(nested, name), source);
    }
    await assert.rejects(loadConfig(nested), /escapes selected project Vite/);
    for (const [name, source] of Object.entries(original.files)) {
      assert.equal(fs.readFileSync(path.join(nested, name), "utf8"), source);
    }
  });
});
