import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  ALL_FEATURES,
  exists,
  read,
  readAll,
  runInit,
  temporaryProject,
  write,
  writeManifest,
} from "./init-support.ts";

const VITE_PLUS_CONFIG = `import { defineConfig } from "vite-plus";

export default defineConfig({
  plugins: [],
});
`;

const EXPECTED_VITE_PLUS_CONFIG = `import { defineConfig } from "vite-plus";
import { createVizeLintConfig } from "oxlint-plugin-vize";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  lint: createVizeLintConfig({
    preset: "happy-path",
    settings: {
      helpLevel: "short",
    },
  }),
  plugins: [vize()],
});
`;

const EXPECTED_VIZE_CONFIG = `import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    templateSyntax: "standard",
  },
  linter: {
    enabled: true,
    preset: "happy-path",
  },
  formatter: {
    singleAttributePerLine: false,
    sortBlocks: true,
  },
  typeChecker: {
    enabled: true,
    strict: true,
    jsxTypecheck: true,
  },
  vite: {
    scanPatterns: ["src/**/*.vue"],
  },
});
`;

const EXPECTED_OXLINT_CONFIG = `import { defineConfig } from "oxlint";
import { configs } from "oxlint-plugin-vize";

export default defineConfig({
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      preset: "happy-path",
      helpLevel: "short",
    },
  },
  rules: configs.happyPath,
});
`;

const EXPECTED_EXTENSIONS = `{
  "recommendations": [
    "ubugeeei.vize"
  ]
}
`;

function vitePlusProject(name: string): string {
  const root = temporaryProject(name);
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { dev: "vp dev" },
    devDependencies: { "vite-plus": "^0.1.0" },
  });
  write(root, "vite.config.ts", VITE_PLUS_CONFIG);
  write(root, "tsconfig.json", "{}\n");
  write(root, "pnpm-lock.yaml", "lockfileVersion: '9.0'\n");
  return root;
}

test("a Vite+ project is configured through the vite.config lint block", async () => {
  const root = vitePlusProject("vite-plus");
  const result = await runInit(root, ALL_FEATURES);

  assert.deepEqual(result.commands, [
    {
      command: "vp",
      args: ["add", "-D", "@vizejs/vite-plugin", "oxlint", "oxlint-plugin-vize", "vize"],
      cwd: root,
    },
  ]);
  assert.deepEqual(result.written, [
    "vite.config.ts",
    "vize.config.ts",
    ".vscode/extensions.json",
    "package.json",
  ]);
  assert.deepEqual(readAll(root, ["vite.config.ts", "vize.config.ts", ".vscode/extensions.json"]), {
    "vite.config.ts": EXPECTED_VITE_PLUS_CONFIG,
    "vize.config.ts": EXPECTED_VIZE_CONFIG,
    ".vscode/extensions.json": EXPECTED_EXTENSIONS,
  });
  assert.equal(
    read(root, "package.json"),
    `${JSON.stringify(
      {
        name: "fixture",
        private: true,
        type: "module",
        scripts: {
          dev: "vp dev",
          "vize:lint": "vize lint --preset happy-path --max-warnings 0 src",
          "vize:fmt": "vize fmt --check src",
          "vize:fmt:fix": "vize fmt --write src",
          "vize:check": "vize check",
        },
        devDependencies: {
          "vite-plus": "^0.1.0",
          "@vizejs/vite-plugin": "^0.306.0",
          oxlint: "^0.306.0",
          "oxlint-plugin-vize": "^0.306.0",
          vize: "^0.306.0",
        },
      },
      null,
      2,
    )}\n`,
  );
  // The trap: a Vite+ project must not be handed an Oxlint config file, because
  // `vp lint` never reads one.
  assert.equal(exists(root, "oxlint.config.ts"), false);
  assert.equal(exists(root, ".oxlintrc.json"), false);
  assert.equal(result.plan?.lintTarget.kind, "vite-plus");
});

test("a second run of an already-configured project writes and runs nothing", async () => {
  const root = vitePlusProject("idempotent");
  const first = await runInit(root, ALL_FEATURES);
  const before = readAll(root, [
    "package.json",
    "vite.config.ts",
    "vize.config.ts",
    ".vscode/extensions.json",
  ]);

  const second = await runInit(root, ALL_FEATURES);

  assert.deepEqual(second.written, []);
  assert.deepEqual(second.commands, []);
  assert.deepEqual(second.plan?.createdFiles, []);
  assert.deepEqual(second.plan?.updatedFiles, []);
  assert.deepEqual(second.plan?.addedScripts, []);
  assert.deepEqual(
    readAll(root, ["package.json", "vite.config.ts", "vize.config.ts", ".vscode/extensions.json"]),
    before,
  );
  assert.deepEqual(
    second.plan?.features.map((feature) => [feature.id, feature.outcome]),
    [
      ["lint", "unchanged"],
      ["bundler", "unchanged"],
      ["fmt", "unchanged"],
      ["typecheck", "unchanged"],
      ["editor", "unchanged"],
    ],
  );
  assert.notEqual(first.written.length, 0);
});

test("a plain Vite project gets the Oxlint config its lint command reads", async () => {
  const root = temporaryProject("plain-vite");
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { dev: "vite", lint: "oxlint src" },
    devDependencies: { vite: "^7.0.0" },
  });
  write(
    root,
    "vite.config.ts",
    `import { defineConfig } from "vite";

export default defineConfig({
  plugins: [],
});
`,
  );
  write(root, "tsconfig.json", "{}\n");
  write(root, "package-lock.json", '{"lockfileVersion":3}\n');

  const result = await runInit(root, ALL_FEATURES);

  assert.deepEqual(result.commands, [
    {
      command: "npm",
      args: ["install", "-D", "@vizejs/vite-plugin", "oxlint", "oxlint-plugin-vize", "vize"],
      cwd: root,
    },
  ]);
  assert.deepEqual(result.written, [
    "oxlint.config.ts",
    "vite.config.ts",
    "vize.config.ts",
    ".vscode/extensions.json",
    "package.json",
  ]);
  assert.deepEqual(readAll(root, ["oxlint.config.ts", "vite.config.ts"]), {
    "oxlint.config.ts": EXPECTED_OXLINT_CONFIG,
    "vite.config.ts": `import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
`,
  });
  assert.equal(result.plan?.lintTarget.kind, "oxlint");
});

test("a Vite+ project that also runs oxlint gets both configs from one preset", async () => {
  const root = vitePlusProject("both-targets");
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { dev: "vp dev", "lint:ci": "oxlint-vize -f stylish src" },
    devDependencies: { "vite-plus": "^0.1.0" },
  });

  const result = await runInit(root, ALL_FEATURES);

  assert.equal(result.plan?.lintTarget.kind, "both");
  assert.deepEqual(readAll(root, ["oxlint.config.ts", "vite.config.ts"]), {
    "oxlint.config.ts": EXPECTED_OXLINT_CONFIG,
    "vite.config.ts": EXPECTED_VITE_PLUS_CONFIG,
  });
});

test("--dry-run reports the plan and writes nothing", async () => {
  const root = vitePlusProject("dry-run");
  const before = readAll(root, ["package.json", "vite.config.ts"]);

  const result = await runInit(root, [...ALL_FEATURES, "--dry-run"]);

  assert.deepEqual(result.written, []);
  assert.deepEqual(result.commands, []);
  assert.deepEqual(readAll(root, ["package.json", "vite.config.ts"]), before);
  assert.equal(exists(root, "vize.config.ts"), false);
  assert.deepEqual(result.plan?.createdFiles, ["vize.config.ts", ".vscode/extensions.json"]);
  assert.deepEqual(result.plan?.updatedFiles, ["vite.config.ts", "package.json"]);
  assert.match(
    result.output,
    /  editor    configured writes \.vscode\/extensions\.json recommending ubugeeei\.vize\n/,
  );
  assert.match(result.output, /\[vize init\] would create \.vscode\/extensions\.json\n/);
  assert.doesNotMatch(result.output, /\.vscode\\extensions\.json/);
});

test("a non-TTY stdin without --yes refuses to prompt instead of hanging", async () => {
  const root = vitePlusProject("non-tty");
  const before = readAll(root, ["package.json", "vite.config.ts"]);

  const result = await runInit(root, ["--lint", "--fmt"]);

  assert.equal(result.plan, null);
  assert.deepEqual(result.written, []);
  assert.deepEqual(result.commands, []);
  assert.deepEqual(readAll(root, ["package.json", "vite.config.ts"]), before);
  assert.equal(
    result.output,
    `[vize init] detected in ${root}:
  framework:       Vite+ (vite.config.ts)
  package manager: pnpm
  language:        TypeScript (tsconfig.json)
  lint command:    vp lint
  vize config:     none
  oxlint config:   none
[vize init] stdin is not a TTY, so init will not prompt.
[vize init] pass --yes with the features you want, for example:
[vize init]   vize init --yes --lint --vite --fmt --typecheck --editor
[vize init] or run with --dry-run to print the plan without writing.
`,
  );
});
