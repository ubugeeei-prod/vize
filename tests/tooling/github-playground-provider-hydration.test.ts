import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { runInNewContext } from "node:vm";
import { parse, parseAllDocuments, stringify } from "yaml";

import { runToolCommand } from "./_helpers/tool-commands.ts";
import { checkWorkflowJobSources, readRepoFile } from "./support/github-workflows.ts";

const workspacePaths = ["playground", "npm/builder/vite", "npm/cli", "npm/native", "npm/ui"];
const oldFilters = workspacePaths.slice(0, 4).map((entry) => `./${entry}...`);
const workflow = parse(checkWorkflowJobSources());
const steps = workflow.jobs["playground-test"].steps;
const install = steps.find((step: { name?: string }) => step.name === "Install JS dependencies");
const filters = [...install.run.matchAll(/--filter '([^']+)'/gu)].map((match) => match[1]);
const manifest = JSON.parse(readRepoFile("npm", "ui", "package.json"));
const alias = "vue-vapor-runtime";
const version = "3.6.0-rc.9";
const plugin = readRepoFile("playground", "e2e", "support", "svg-namespace-corpus-plugin.ts");

void test("full playground keeps its installer and browser gates while hydrating the UI provider", () => {
  assert.deepEqual(filters, [...oldFilters, "./npm/ui..."]);
  assert.equal(
    install.run,
    "node tools/support/github/vp-install.ts " +
      filters.map((filter) => `--filter '${filter}'`).join(" "),
  );
  assert.equal(
    steps.find((step: { name?: string }) => step.name === "Run playground tests").run,
    "vp run --filter './playground' test:browser",
  );
  assert.equal(
    steps.find((step: { name?: string }) => step.name === "Run VRT").run,
    "vp run --filter './playground' test:vrt",
  );
  assert.equal(manifest.devDependencies[alias], `npm:vue@${version}`);
  assert.match(plugin, /assert\.equal\(compiler\.version, "3\.6\.0-rc\.9"\)/u);
  assert.match(plugin, /svgNamespaceCorpusPlugin/u);
  assert.match(
    readRepoFile("playground", "vite.test.config.ts"),
    /plugins: \[vueTestUtilsOptimizerWorkaround, svgNamespaceCorpusPlugin\(\), vize\(\)\]/u,
  );
});

// Execute the unchanged provider declarations from the real browser plugin.
// Its native producer and whole browser oracles remain the full Actions gate.
function providerDeclarations(baseUrl: URL) {
  const ui = plugin.match(/^const fromUi = .+;$/mu)?.[0];
  const vue = plugin.match(
    /^const fromVue = .+;\nconst compiler = .+;\nconst runtime = .+;$/mu,
  )?.[0];
  assert.ok(ui && vue, "the actual plugin's provider declarations must remain observable");
  assert.equal(
    ui,
    'const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));',
  );
  return runInNewContext(
    `${ui}\n${vue}\n({ fromUi, fromVue, compiler, runtime });`.replaceAll(
      "import.meta.url",
      "baseUrl",
    ),
    { createRequire, URL, baseUrl },
  );
}

void test("the actual filtered installer reproduces missing UI and resolves the unchanged pinned provider", (t) => {
  const directory = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-playground-provider-")),
  );
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const rootPackage = JSON.parse(readRepoFile("package.json"));
  fs.writeFileSync(
    path.join(directory, "package.json"),
    JSON.stringify({
      private: true,
      packageManager: rootPackage.packageManager,
      devEngines: rootPackage.devEngines,
    }),
  );
  fs.writeFileSync(
    path.join(directory, "pnpm-workspace.yaml"),
    stringify({ packages: workspacePaths }),
  );
  const importers: Record<string, unknown> = { ".": {} };
  for (const entry of workspacePaths) {
    const actual = JSON.parse(readRepoFile(entry, "package.json"));
    fs.mkdirSync(path.join(directory, entry), { recursive: true });
    fs.writeFileSync(
      path.join(directory, entry, "package.json"),
      JSON.stringify({
        name: actual.name,
        private: true,
        ...(entry === "npm/ui"
          ? { devDependencies: { [alias]: manifest.devDependencies[alias] } }
          : {}),
      }),
    );
    importers[entry] = {};
  }
  // Vite+'s first lock document owns the package manager; the second owns dependencies.
  // Retain authenticated resolution/snapshot bodies and the alias's original exact lock entry.
  const locks = parseAllDocuments(readRepoFile("pnpm-lock.yaml")).map((document) =>
    document.toJSON(),
  );
  assert.equal(locks.length, 2);
  importers["."] = { devDependencies: { node: locks[1].importers["."].devDependencies.node } };
  const lockedAlias = locks[1].importers["npm/ui"].devDependencies[alias];
  assert.equal(lockedAlias.specifier, manifest.devDependencies[alias]);
  assert.equal(lockedAlias.version, `vue@${version}(typescript@6.0.3)`);
  importers["npm/ui"] = { devDependencies: { [alias]: lockedAlias } };
  fs.writeFileSync(
    path.join(directory, "pnpm-lock.yaml"),
    "---\n" +
      stringify(locks[0]) +
      "---\n" +
      stringify({
        lockfileVersion: locks[1].lockfileVersion,
        settings: locks[1].settings,
        importers,
        packages: locks[1].packages,
        snapshots: locks[1].snapshots,
      }),
  );
  const baseUrl = pathToFileURL(
    path.join(directory, "playground/e2e/support/svg-namespace-corpus-plugin.ts"),
  );
  const frozenLock = fs.readFileSync(path.join(directory, "pnpm-lock.yaml"));
  const installWith = (selected: string[]) => {
    const result = runToolCommand(
      "tools/support/github/vp-install.ts",
      selected.flatMap((filter) => ["--filter", filter]),
      {
        cwd: directory,
      },
    );
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  };
  installWith(oldFilters);
  assert.equal(fs.existsSync(path.join(directory, "npm/ui/node_modules", alias)), false);
  assert.throws(() => providerDeclarations(baseUrl), { code: "MODULE_NOT_FOUND" });
  installWith(filters);
  const actual = providerDeclarations(baseUrl);
  const packageJson = actual.fromUi.resolve(`${alias}/package.json`);
  const providerRoot = fs.realpathSync(path.dirname(packageJson));
  const expected = JSON.parse(fs.readFileSync(packageJson, "utf8"));
  assert.equal(expected.name, "vue");
  assert.equal(expected.version, version);
  assert.equal(actual.compiler.version, version);
  assert.equal(actual.fromVue("./package.json").version, actual.compiler.version);
  assert.equal(
    fs.realpathSync(actual.runtime),
    path.join(providerRoot, "dist/vue.runtime-with-vapor.esm-browser.js"),
  );
  assert.equal(actual.compiler, actual.fromVue("@vue/compiler-sfc"));
  assert.ok(
    fs.statSync(actual.runtime).size > 0,
    "the real pinned browser runtime must be present",
  );
  assert.deepEqual(fs.readFileSync(path.join(directory, "pnpm-lock.yaml")), frozenLock);
});
