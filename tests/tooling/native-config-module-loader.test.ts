import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import {
  access,
  mkdtemp,
  mkdir,
  copyFile,
  readFile,
  readdir,
  realpath,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { createRequire, registerHooks } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { promisify } from "node:util";

import {
  importConfigModule,
  importConfigModuleWithDependencies,
} from "../../crates/vize_carton/src/config/loader/module-runtime.mjs";
import { resolveViteConfigExport } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

const require = createRequire(join(process.cwd(), "package.json"));
const vitePackage = dirname(require.resolve("vite/package.json"));
const execute = promisify(execFile);

async function nativeLoaderScript() {
  const loader = new URL("../../crates/vize_carton/src/config/loader/js.rs", import.meta.url);
  const source = await readFile(loader, "utf8");
  const includes = [...source.matchAll(/include_str!\("([^"]+)"\)/g)];
  const scripts = [...source.matchAll(/r#"([\s\S]*?)"#/g)];
  assert.equal(scripts.length, 1);
  const bodies = await Promise.all(
    includes.map((match) => readFile(new URL(match[1], loader), "utf8")),
  );
  return [...bodies, scripts[0][1]].join("");
}

async function project(
  run: (directory: string) => Promise<void>,
  withVite = true,
  installedVite = vitePackage,
) {
  const directory = await realpath(await mkdtemp(join(tmpdir(), "vize-config-original-")));
  try {
    await writeFile(join(directory, "package.json"), '{"type":"module","value":7}');
    if (withVite) {
      await mkdir(join(directory, "node_modules"));
      await symlink(installedVite, join(directory, "node_modules/vite"), "junction");
    }
    await run(directory);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

await test("Vite's real bundle loader preserves authored JSON, helper resolution and file scopes", async () => {
  await project(async (directory) => {
    await mkdir(join(directory, "tools"));
    await writeFile(
      join(directory, "tools/helper.ts"),
      "export const answer: number = 11; export const scope=__dirname;",
    );
    await writeFile(join(directory, "tools/typed.ts"), "export const typed: number = 13;");
    const selected = join(directory, "vite.config.mts");
    await writeFile(
      selected,
      `import pkg from './package.json';
       import {answer,scope} from './tools/helper';
       import {typed} from './tools/typed.js';
       export default {value:pkg.value,answer,typed,scope,dir:__dirname,url:import.meta.url};`,
    );
    await assert.rejects(import(pathToFileURL(selected).href));
    const loaded = await importConfigModule(selected);
    assert.deepEqual(loaded.default, {
      value: 7,
      answer: 11,
      typed: 13,
      scope: join(directory, "tools"),
      dir: directory,
      url: pathToFileURL(selected).href,
    });
  });
});

await test("raw Vize factory authority survives bundling and its source evaluates exactly once", async () => {
  await project(async (directory) => {
    const counter = join(directory, "evaluations.json");
    const selected = join(directory, "vite.config.ts");
    await writeFile(
      selected,
      `import {writeFileSync,readFileSync} from 'node:fs';
       const counter=${JSON.stringify(counter)};
       writeFileSync(counter, JSON.stringify({module:1,source:0}));
       const wrapper=()=>{throw new Error('Vite+ wrapper must not run');};
       wrapper[Symbol.for('@vizejs/vite-plugin/vite-plus/source')]=async(env)=>{
         const counts=JSON.parse(readFileSync(counter,'utf8')); counts.source++;
         writeFileSync(counter,JSON.stringify(counts));
         return {vue:{formatter:{printWidth:env.command==='serve'?91:92}}};
       };
       export default wrapper;`,
    );
    const loaded = await importConfigModule(selected);
    assert.equal(typeof loaded.default, "function");
    const config = await resolveViteConfigExport(loaded.default);
    assert.equal(config.formatter.printWidth, 91);
    assert.deepEqual(JSON.parse(await readFile(counter, "utf8")), { module: 1, source: 1 });
  });
});

await test("selected malformed JSON and missing imports remain errors without loading siblings", async () => {
  for (const [dependency, content, statement] of [
    ["broken.json", "{ broken JSON", "import broken from './broken.json';"],
    ["unrelated.ts", "export const value=7;", "import broken from './missing.js';"],
  ]) {
    await project(async (directory) => {
      const selected = join(directory, "vite.config.ts");
      await writeFile(join(directory, dependency), content);
      await writeFile(selected, `${statement} export default {value:broken};`);
      await writeFile(join(directory, "vize.config.mjs"), "throw new Error('sibling selected');");
      await assert.rejects(importConfigModule(selected));
    });
  }
});

await test("a present broken Vite installation cannot silently select native loading", async () => {
  await project(async (directory) => {
    await mkdir(join(directory, "node_modules/vite"), { recursive: true });
    await writeFile(join(directory, "node_modules/vite/package.json"), "{broken manifest");
    const selected = join(directory, "vite.config.mjs");
    await writeFile(selected, "export default {formatter:{printWidth:90}};");
    await assert.rejects(importConfigModule(selected));
  }, false);
});

await test("dependency-free native configurations retain their original module contract", async () => {
  await project(async (directory) => {
    const selected = join(directory, "vize.config.mjs");
    await writeFile(selected, "export default {formatter:{printWidth:90}};");
    assert.deepEqual((await importConfigModule(selected)).default, {
      formatter: { printWidth: 90 },
    });
  }, false);
});

await test("the loader removes its private adapters after both successful and rejected imports", async () => {
  const adapters = async () =>
    (await readdir(tmpdir())).filter((entry) => entry.startsWith("vize-config-module-")).sort();
  const before = await adapters();
  await project(async (directory) => {
    const selected = join(directory, "vite.config.mjs");
    await writeFile(selected, "export default {formatter:{printWidth:90}};");
    await importConfigModule(selected);
    await writeFile(selected, "throw new Error('actual selected config failed');");
    await assert.rejects(importConfigModule(selected), /actual selected config failed/);
  });
  assert.deepEqual(await adapters(), before);
});

await test("the exact script embedded by native Carton loads and projects the original selected config once", async () => {
  const script = await nativeLoaderScript();
  await project(async (directory) => {
    const selected = join(directory, "vite.config.mts");
    const counter = join(directory, "native-evaluations.txt");
    const authored = `import pkg from './package.json';
       import {appendFileSync} from 'node:fs';
       appendFileSync(${JSON.stringify(counter)}, ${JSON.stringify("module\n")});
       export default () => {
         appendFileSync(${JSON.stringify(counter)}, ${JSON.stringify("source\n")});
         return {vue:{formatter:{printWidth:pkg.value+83}},fmt:{vue:{singleQuote:true}}};
       };`;
    await writeFile(selected, authored);
    const loaded = await execute(
      process.execPath,
      ["--input-type=module", "-e", script, selected],
      {
        cwd: dirname(selected),
        timeout: 10_000,
      },
    );
    const { config, dependencies } = JSON.parse(loaded.stdout);
    assert.ok(dependencies.includes(selected));
    assert.deepEqual(config.formatter, { printWidth: 90, singleQuote: true });
    assert.equal(config.typeChecker.jsxTypecheck, true);
    assert.equal(await readFile(counter, "utf8"), "module\nsource\n");
    assert.equal(await readFile(selected, "utf8"), authored);
  });
});

await test("Vite 7's real esbuild loader bundles the original module rather than externalizing its file URL", async () => {
  const dependencies = await realpath(await mkdtemp(join(tmpdir(), "vize-config-vite7-")));
  try {
    for (const file of ["package.json", "package-lock.json"])
      await copyFile(
        new URL(`../_fixtures/_projects/native-config-loader-vite7/${file}`, import.meta.url),
        join(dependencies, file),
      );
    await execute(
      process.platform === "win32" ? "npm.cmd" : "npm",
      ["ci", "--ignore-scripts", "--no-audit", "--no-fund", "--prefix", dependencies],
      { timeout: 60_000 },
    );
    const installed = join(dependencies, "node_modules/vite");
    const version = JSON.parse(await readFile(join(installed, "package.json"), "utf8"));
    assert.equal(version.version, "7.3.1");
    await project(
      async (directory) => {
        const selected = join(directory, "vite.config.ts");
        const authored = "import pkg from './package.json'; export default {value:pkg.value};";
        await writeFile(selected, authored);
        assert.deepEqual((await importConfigModule(selected)).default, { value: 7 });
        assert.equal(await readFile(selected, "utf8"), authored);
        const helper = join(directory, "policy.mjs");
        await writeFile(helper, "export default {formatter:{printWidth:94}};");
        await writeFile(
          selected,
          `export default async()=>{
          const name='policy';
          return {vue:(await import(new URL('./'+name+'.mjs',import.meta.url))).default};
        };`,
        );
        const evaluated = await importConfigModuleWithDependencies(
          selected,
          undefined,
          undefined,
          (namespace: Record<string, unknown>) => resolveViteConfigExport(namespace.default),
        );
        assert.deepEqual(evaluated.value, {
          typeChecker: { jsxTypecheck: true },
          formatter: { printWidth: 94 },
        });
        assert.ok(evaluated.dependencies.includes(helper));
      },
      true,
      installed,
    );
  } finally {
    await rm(dependencies, { recursive: true, force: true });
  }
});

await test("nearest present Vite owns loading while legitimate symlinks and hoisting retain one evaluation", async () => {
  for (const installation of ["empty", "types-only", "broken", "dangling", "symlink", "hoisted"]) {
    await project(async (directory) => {
      const nested = join(directory, "nested");
      await mkdir(join(nested, "node_modules"), { recursive: true });
      await writeFile(join(nested, "package.json"), '{"type":"module"}');
      const candidate = join(nested, "node_modules/vite");
      if (installation === "symlink") await symlink(vitePackage, candidate, "junction");
      else if (installation === "dangling")
        await symlink(join(directory, "absent-vite"), candidate, "junction");
      else if (installation !== "hoisted") {
        await mkdir(candidate);
        if (installation === "broken") await writeFile(join(candidate, "package.json"), "{broken");
        if (installation === "types-only") {
          await writeFile(join(candidate, "package.json"), '{"name":"vite","types":"client.d.ts"}');
          await writeFile(join(candidate, "client.d.ts"), "");
        }
      }
      const counter = join(nested, "evaluations.json");
      const selected = join(nested, "vite.config.ts");
      const authored = `import {existsSync,readFileSync,writeFileSync} from 'node:fs';
        const counter=${JSON.stringify(counter)};
        const counts=existsSync(counter)?JSON.parse(readFileSync(counter,'utf8')):{module:0,source:0};
        counts.module++; writeFileSync(counter,JSON.stringify(counts));
        const wrapper=()=>{throw new Error('Vite+ wrapper executed');};
        wrapper[Symbol.for('@vizejs/vite-plugin/vite-plus/source')]=(env)=>{
          const counts=JSON.parse(readFileSync(counter,'utf8'));counts.source++;
          writeFileSync(counter,JSON.stringify(counts));
          if(env.command!=='serve'||env.mode!=='development')throw new Error('environment changed');
          return {vue:{formatter:{printWidth:91}}};
        };export default wrapper;`;
      await writeFile(selected, authored);
      if (["empty", "types-only", "broken", "dangling"].includes(installation)) {
        await assert.rejects(importConfigModule(selected));
        await assert.rejects(access(counter));
      } else {
        const loaded = await importConfigModule(selected);
        assert.deepEqual(await resolveViteConfigExport(loaded.default), {
          typeChecker: { jsxTypecheck: true },
          formatter: { printWidth: 91 },
        });
        assert.deepEqual(JSON.parse(await readFile(counter, "utf8")), { module: 1, source: 1 });
      }
      assert.equal(await readFile(selected, "utf8"), authored);
    });
  }
});

await test("global Vite search paths cannot replace a dependency-free project's native loading", async () => {
  await project(async (directory) => {
    const global = join(directory, "global");
    await mkdir(join(global, "vite"), { recursive: true });
    await writeFile(
      join(global, "vite/package.json"),
      '{"name":"vite","type":"module","exports":"./index.mjs"}',
    );
    await writeFile(
      join(global, "vite/index.mjs"),
      "export function loadConfigFromFile(){throw new Error('global Vite executed');}",
    );
    const selected = join(directory, "vite.config.mjs");
    const authored = "export default {vue:{formatter:{printWidth:92}}};";
    await writeFile(selected, authored);
    const loader = new URL(
      "../../crates/vize_carton/src/config/loader/module-runtime.mjs",
      import.meta.url,
    );
    const script = `import {importConfigModule} from ${JSON.stringify(loader.href)};
      const loaded=await importConfigModule(process.argv[1]);process.stdout.write(JSON.stringify(loaded.default));`;
    const output = await execute(
      process.execPath,
      ["--input-type=module", "-e", script, selected],
      {
        cwd: directory,
        env: { ...process.env, NODE_PATH: global },
        timeout: 10_000,
      },
    );
    assert.equal(output.stderr, "");
    assert.deepEqual(JSON.parse(output.stdout), { vue: { formatter: { printWidth: 92 } } });
    assert.equal(await readFile(selected, "utf8"), authored);
  }, false);
});

for (const withVite of [true, false]) {
  await test(`configuration graph retains imported helpers (Vite installed: ${withVite})`, async () => {
    await project(async (directory) => {
      const helper = join(directory, "policy.mjs");
      const selected = join(directory, "vite.config.mjs");
      await writeFile(helper, "export default { languageServer: { hover: false } };");
      await writeFile(selected, "import vue from './policy.mjs'; export default { vue };");
      const loaded = await importConfigModuleWithDependencies(selected);
      assert.deepEqual(loaded.namespace.default, { vue: { languageServer: { hover: false } } });
      assert.ok(loaded.dependencies.includes(selected));
      assert.ok(loaded.dependencies.includes(helper));
      assert.ok(loaded.dependencies.every((path) => !path.includes("vize-config-module-")));
    }, withVite);
  });
}

await test("older Node runtimes require project Vite to retain a complete configuration graph", async () => {
  await project(async (directory) => {
    const selected = join(directory, "vite.config.mjs");
    await writeFile(selected, "export default {};");
    await assert.rejects(
      importConfigModuleWithDependencies(selected, null),
      /Node\.js 22\.15.*install Vite/,
    );
  }, false);
  await project(async (directory) => {
    const selected = join(directory, "vite.config.mjs");
    await writeFile(selected, "export default {};");
    const loaded = await importConfigModuleWithDependencies(selected, null);
    assert.ok(loaded.dependencies.includes(selected));
  });
});

for (const withVite of [true, false]) {
  for (const form of ["factory", "fluent", "top-level"]) {
    await test(`native async export retains runtime helpers exactly once (Vite: ${withVite}, form: ${form})`, async () => {
      const script = await nativeLoaderScript();
      await project(async (directory) => {
        const selected = join(directory, "vite.config.mjs");
        const helper = join(directory, "policy.mjs");
        const counter = join(directory, "evaluations.txt");
        await writeFile(
          helper,
          `import {appendFileSync} from 'node:fs';
          appendFileSync(${JSON.stringify(counter)},'helper\\n');
          export default {languageServer:{hover:false}};`,
        );
        const authored = `import {appendFileSync} from 'node:fs';
          appendFileSync(${JSON.stringify(counter)},'module\\n');
          const source=async()=>{
            appendFileSync(${JSON.stringify(counter)},'source\\n');
            await new Promise(resolve=>setTimeout(resolve,1));
            const name='policy';
            const helper=await import(new URL('./'+name+'.mjs',import.meta.url));
            return {vue:helper.default};
          };
          ${
            form === "fluent"
              ? `const wrapper=()=>{throw new Error('fluent wrapper evaluated');};
            wrapper[Symbol.for('@vizejs/vite-plugin/vite-plus/source')]=source;
            export default wrapper;`
              : form === "top-level"
                ? "export default await source();"
                : "export default source;"
          }
        `;
        await writeFile(selected, authored);
        const output = await execute(
          process.execPath,
          ["--input-type=module", "-e", script, selected],
          {
            cwd: directory,
            timeout: 10_000,
          },
        );
        const loaded = JSON.parse(output.stdout);
        assert.equal(loaded.config.languageServer.hover, false);
        assert.ok(loaded.dependencies.includes(selected));
        assert.ok(
          loaded.dependencies.includes(helper),
          "runtime-imported policy must invalidate the snapshot",
        );
        assert.deepEqual(
          [...loaded.dependencies].sort((left: string, right: string) => left.localeCompare(right)),
          [selected, helper].sort((left, right) => left.localeCompare(right)),
        );
        assert.ok(
          loaded.dependencies.every((file: string) => !file.includes("vize-config-module-")),
        );
        assert.equal(await readFile(counter, "utf8"), "module\nsource\nhelper\n");
        assert.equal(await readFile(selected, "utf8"), authored);
      }, withVite);
    });
  }
}

await test("tracked export evaluation releases hooks after success and rejection", async () => {
  for (const stage of ["success", "import", "evaluate"]) {
    await project(async (directory) => {
      const selected = join(directory, "vite.config.mjs");
      await writeFile(
        selected,
        stage === "import" ? "throw new Error('import failed');" : "export default {};",
      );
      let active = 0;
      let evaluations = 0;
      const tracked = (hooks: Parameters<typeof registerHooks>[0]) => {
        const registration = registerHooks(hooks);
        active++;
        return {
          deregister() {
            active--;
            registration.deregister();
          },
        };
      };
      const evaluation = importConfigModuleWithDependencies(
        selected,
        tracked,
        undefined,
        async (namespace: unknown) => {
          evaluations++;
          assert.equal(active, 1);
          await Promise.resolve();
          if (stage === "evaluate") throw new Error("evaluation failed");
          return namespace;
        },
      );
      if (stage !== "success") await assert.rejects(evaluation, /(?:import|evaluation) failed/);
      else await evaluation;
      assert.equal(active, 0);
      assert.equal(evaluations, stage === "import" ? 0 : 1);
    }, false);
  }
});

await test("installed Vite cannot replace runtime import tracking on older Node", async () => {
  await project(async (directory) => {
    const selected = join(directory, "vite.config.mjs");
    await writeFile(selected, "export default {};");
    let evaluated = false;
    await assert.rejects(
      importConfigModuleWithDependencies(selected, null, undefined, () => {
        evaluated = true;
      }),
      /Node\.js 22\.15.*Upgrade Node\.js/,
    );
    assert.equal(evaluated, false);
  });
});
