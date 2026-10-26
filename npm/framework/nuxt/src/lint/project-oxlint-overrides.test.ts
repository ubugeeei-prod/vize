import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, realpath, rm, symlink, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import type { TestContext } from "node:test";

import { runNuxtLintCheckerTask } from "./checker/worker.ts";
import { setupNuxtLintConfigGeneration } from "./generation.ts";
import { readProjectLintRuleSets } from "./project-rules.ts";

async function temporaryRoot(t: TestContext, prefix: string): Promise<string> {
  // Oxlint matches overrides against canonical paths; macOS tmpdir is a symlink.
  const root = await realpath(await mkdtemp(path.join(os.tmpdir(), prefix)));
  t.after(() => rm(root, { force: true, recursive: true }));
  return root;
}

function createNuxt(rootDir: string) {
  return {
    options: {
      rootDir,
      srcDir: path.join(rootDir, "app"),
      buildDir: path.join(rootDir, ".nuxt"),
      dir: {},
      _layers: [{ config: { srcDir: path.join(rootDir, "app") } }],
    },
    hook() {},
  };
}

function unexpectedPreset(): never {
  throw new Error("preset loader should not run");
}

const LOCAL_PLUGIN = `export default {
  meta: { name: "local" },
  rules: { "no-foo": { create(context) { return {
    Identifier(node) {
      if (node.name === "foo") context.report({ node, message: "Do not name things foo." });
    }
  }; } } }
};`;

void test("Vite+ rule options, severity aliases and overrides are read from the lint block", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-oxlint-overrides-");
  await writeFile(
    path.join(root, "vite.config.mjs"),
    `export default { lint: {
      rules: { "no-console": ["error", { allow: ["warn"] }], eqeqeq: 2, "no-alert": "deny", "no-var": ["bogus"] },
      overrides: [
        { files: ["app/legacy/**", "!app/legacy/keep.ts"], rules: { "local/no-foo": "off", "no-alert": 0 } },
        { files: ["app/**"] },
        { rules: { "no-var": "off" } }
      ]
    } };`,
  );

  assert.deepEqual(await readProjectLintRuleSets(root, unexpectedPreset), {
    oxlint: {
      "no-console": ["error", { allow: ["warn"] }],
      eqeqeq: "error",
      "no-alert": "error",
    },
    oxlintOverrides: [
      {
        files: [path.join(root, "app/legacy/**"), `!${path.join(root, "app/legacy/keep.ts")}`],
        rules: { "local/no-foo": "off", "no-alert": "off" },
      },
    ],
  });
});

void test("the dev checker honors Vite+ overrides and [severity, options] rules like vp lint", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-oxlint-parity-");
  await mkdir(path.join(root, "app", "legacy"), { recursive: true });
  await mkdir(path.join(root, "node_modules"));
  await symlink(
    path.dirname(createRequire(import.meta.url).resolve("oxlint/package.json")),
    path.join(root, "node_modules", "oxlint"),
    "junction",
  );
  await writeFile(path.join(root, "local-plugin.mjs"), LOCAL_PLUGIN);
  await writeFile(
    path.join(root, "vite.config.mjs"),
    `export default { lint: {
      jsPlugins: ["./local-plugin.mjs"],
      rules: { "local/no-foo": "error", "no-console": ["error", { allow: ["warn"] }] },
      overrides: [{ files: ["app/legacy/**"], rules: { "local/no-foo": "off" } }]
    } };`,
  );
  const current = path.join(root, "app", "current.ts");
  const legacy = path.join(root, "app", "legacy", "old.ts");
  await writeFile(current, 'const foo = 1;\nconsole.log(foo);\nconsole.warn("ok");\n');
  await writeFile(legacy, "const foo = 1;\nconsole.log(foo);\n");

  const generation = await setupNuxtLintConfigGeneration({ autoInit: false }, createNuxt(root), {
    resolvePluginSpecifier: () => "./vize-plugin.mjs",
  });
  const artifact = JSON.parse(await readFile(generation?.configFile ?? "", "utf8")) as {
    overrides: Array<{ files: string[]; rules?: Record<string, unknown> }>;
    rules: Record<string, unknown>;
  };
  assert.deepEqual(artifact.rules["no-console"], ["error", { allow: ["warn"] }]);
  assert.deepEqual(
    artifact.overrides.find(({ files }) => files.includes("app/legacy/**")),
    { files: ["app/legacy/**"], rules: { "local/no-foo": "off" } },
  );

  const vizeRules = [
    ...Object.keys(artifact.rules),
    ...artifact.overrides.flatMap(({ rules }) => Object.keys(rules ?? {})),
  ]
    .filter((rule, index, rules) => rule.startsWith("vize/") && rules.indexOf(rule) === index)
    .map((rule) => `${JSON.stringify(rule.slice("vize/".length))}: { create() { return {}; } }`);
  await writeFile(
    path.join(root, "vize-plugin.mjs"),
    `export default { meta: { name: "vize" }, rules: { ${vizeRules.join(",")} } };`,
  );

  const result = await runNuxtLintCheckerTask({
    configFile: generation?.configFile ?? "",
    cwd: root,
    emitError: true,
    emitWarning: true,
    exclude: [],
    fix: false,
    formatter: "json",
    targets: [current, legacy],
  });
  assert.deepEqual(
    result.diagnostics.map(({ code, filename }) => `${filename ?? ""} ${code ?? ""}`).sort(),
    [
      "app/current.ts eslint(no-console)",
      "app/current.ts local(no-foo)",
      "app/current.ts local(no-foo)",
      "app/legacy/old.ts eslint(no-console)",
    ],
  );
});
