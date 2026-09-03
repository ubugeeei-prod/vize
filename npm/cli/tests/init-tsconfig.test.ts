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

const EXPECTED_JAVASCRIPT_TSCONFIG = `{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "jsx": "preserve",
    "allowJs": true,
    "checkJs": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": [
    "src/**/*"
  ]
}
`;

const EXPECTED_TYPESCRIPT_TSCONFIG = `{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "jsx": "preserve",
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": [
    "src/**/*"
  ]
}
`;

test("a JavaScript-only project scaffolds checkJs typechecking", async () => {
  const root = temporaryProject("javascript");
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { dev: "vite" },
    devDependencies: { vite: "^7.0.0" },
  });
  write(
    root,
    "vite.config.js",
    `import { defineConfig } from "vite";

export default defineConfig({});
`,
  );
  write(root, "bun.lock", "{}\n");

  const result = await runInit(root, ALL_FEATURES);

  assert.deepEqual(result.commands, [
    {
      command: "bun",
      args: ["add", "-D", "@vizejs/vite-plugin", "oxlint", "oxlint-plugin-vize", "vize"],
      cwd: root,
    },
  ]);
  assert.deepEqual(
    result.plan?.features.map((feature) => [feature.id, feature.outcome]),
    [
      ["lint", "configured"],
      ["bundler", "configured"],
      ["fmt", "configured"],
      ["typecheck", "configured"],
      ["editor", "configured"],
    ],
  );
  assert.deepEqual(result.plan?.addedScripts, [
    "vize:lint",
    "vize:fmt",
    "vize:fmt:fix",
    "vize:check",
  ]);
  assert.deepEqual(readAll(root, ["vite.config.js", "vize.config.ts", "tsconfig.json"]), {
    "vite.config.js": `import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
`,
    "vize.config.ts": `import { defineConfig } from "vize";

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
`,
    "tsconfig.json": EXPECTED_JAVASCRIPT_TSCONFIG,
  });

  const second = await runInit(root, ALL_FEATURES);
  assert.deepEqual(second.written, []);
  assert.deepEqual(second.commands, []);
});

test("a TypeScript project without a config gets a strict minimum scaffold", async () => {
  const root = temporaryProject("typescript-without-config");
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    devDependencies: { typescript: "^6.0.0" },
  });

  const args = [
    "--yes",
    "--no-lint",
    "--no-bundler",
    "--no-fmt",
    "--typecheck",
    "--no-editor",
    "--no-install",
  ] as const;

  const dryRun = await runInit(root, [...args, "--dry-run"]);
  assert.deepEqual(dryRun.written, []);
  assert.deepEqual(dryRun.plan?.createdFiles, ["tsconfig.json", "vize.config.ts"]);
  assert.deepEqual(dryRun.plan?.updatedFiles, ["package.json"]);
  assert.equal(exists(root, "tsconfig.json"), false);

  const result = await runInit(root, args);

  assert.deepEqual(result.written, ["tsconfig.json", "vize.config.ts", "package.json"]);
  assert.equal(read(root, "tsconfig.json"), EXPECTED_TYPESCRIPT_TSCONFIG);
  assert.equal(
    result.plan?.features.find((feature) => feature.id === "typecheck")?.outcome,
    "configured",
  );
  assert.deepEqual(result.plan?.addedScripts, ["vize:check"]);
});

test("a missing tsconfig is added without rewriting an existing Vize config", async () => {
  const root = temporaryProject("existing-vize-config-without-tsconfig");
  writeManifest(root, {
    name: "fixture",
    private: true,
    type: "module",
    scripts: { "vize:check": "vize check src" },
    devDependencies: { typescript: "^6.0.0", vize: "^0.306.0" },
  });
  const existingConfig = `export default { typeChecker: { strict: false } };\n`;
  write(root, "vize.config.ts", existingConfig);

  const result = await runInit(root, [
    "--yes",
    "--no-lint",
    "--no-bundler",
    "--no-fmt",
    "--typecheck",
    "--no-editor",
    "--no-install",
  ]);

  assert.deepEqual(result.written, ["tsconfig.json"]);
  assert.equal(read(root, "vize.config.ts"), existingConfig);
  assert.equal(read(root, "tsconfig.json"), EXPECTED_TYPESCRIPT_TSCONFIG);
  assert.deepEqual(
    result.plan?.features.find((feature) => feature.id === "typecheck"),
    {
      id: "typecheck",
      outcome: "configured",
      detail: "writes tsconfig.json; vize.config.ts already exists and was left unchanged",
      snippet: null,
    },
  );
});
